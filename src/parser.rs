use std::{
  collections::HashSet,
  path::{Path, PathBuf},
};

use anyhow::Context;
use streaming_iterator::StreamingIterator;
use tree_sitter::{Parser as TreeSitterParser, QueryCursor};

use crate::{
  config::{Config, Language, LanguageConfig},
  symbol::{Kind, Symbol},
  text::{Loc, Span},
};

pub struct Parser<'a> {
  path: PathBuf,
  language: Language,
  language_config: &'a LanguageConfig,
}

impl<'a> Parser<'a> {
  pub fn from_path<P: AsRef<Path>>(config: &'a Config, path: P) -> Option<Self> {
    let path = path.as_ref();
    let extension = path.extension()?.to_str()?;
    let language = Language::from_extension(extension)?;
    let language_config = config.languages.get(&language)?;

    Some(Self {
      path: path.to_path_buf(),
      language,
      language_config,
    })
  }

  /// For Vue files, extracts the content of the `<script>` block and prepends
  /// empty lines so that tree-sitter reports correct absolute line numbers.
  /// Returns `None` if no `<script>` block is found.
  fn extract_vue_script(content: &str) -> Option<String> {
    let mut script_start_line = None;
    let mut script_end_line = None;

    for (i, line) in content.lines().enumerate() {
      let trimmed = line.trim();
      if script_start_line.is_none() && trimmed.starts_with("<script") && (trimmed.ends_with('>') || trimmed.contains('>')) {
        script_start_line = Some(i);
      } else if script_start_line.is_some() && script_end_line.is_none() && trimmed.starts_with("</script") {
        script_end_line = Some(i);
        break;
      }
    }

    let start = script_start_line?;
    let end = script_end_line?;

    let lines: Vec<&str> = content.lines().collect();
    // Prepend empty lines to preserve correct line numbers
    let padding = "\n".repeat(start + 1);
    let script_content: String = lines[start + 1..end].join("\n");

    Some(format!("{}{}", padding, script_content))
  }

  /// Derives the Vue component name from the file path.
  /// e.g. `/path/to/App.vue` -> `"App"`
  fn vue_component_name(path: &Path) -> Option<String> {
    let stem = path.file_stem()?.to_str()?;
    Some(stem.to_string())
  }

  pub fn on_symbol(&self, callback: impl Fn(Symbol) -> Result<(), anyhow::Error>) -> Result<(), anyhow::Error> {
    let mut parser = TreeSitterParser::new();
    parser.set_language(&self.language.to_tree_sitter()).context("set_language")?;

    let raw_content = std::fs::read_to_string(&self.path).context("read")?;

    // For Vue files, emit the component name and extract the <script> block
    let content = if self.language == Language::Vue {
      // Emit the component name as a Class symbol at line 1, column 1
      if let Some(component_name) = Self::vue_component_name(&self.path) {
        let span = Span::new(Loc::new(1, 1), Loc::new(1, 1));
        callback(Symbol {
          span,
          text: &component_name,
          kind: Kind::Class,
        })
        .context("callback")?;
      }

      // Extract <script> block; if none found, nothing more to parse
      match Self::extract_vue_script(&raw_content) {
        Some(script) => script,
        None => return Ok(()),
      }
    } else {
      raw_content
    };

    let tree = parser.parse(content.as_bytes(), None).context("parse")?;
    let mut positions = HashSet::new();

    for (kind, queries) in &self.language_config.symbol_queries {
      for query in queries {
        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(query, tree.root_node(), content.as_bytes());

        while let Some(m) = matches.next() {
          for capture in m.captures {
            let node = capture.node;
            let start_pos = node.start_position();

            if positions.contains(&start_pos) {
              continue;
            } else {
              positions.insert(start_pos);
            }

            let end_pos = node.start_position();

            let start_byte = node.start_byte();
            let end_byte = node.end_byte();
            let text = &content[start_byte..end_byte];

            let span = Span::new(
              Loc::new(start_pos.row + 1, start_pos.column + 1),
              Loc::new(end_pos.row + 1, end_pos.column + 1),
            );

            callback(Symbol { span, text, kind: *kind }).context("callback")?;
          }
        }
      }
    }

    Ok(())
  }
}
