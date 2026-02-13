use std::{fmt::Display, path::Path};

use serde::{Deserialize, Serialize};

const SPACE: char = '\u{2008}';

use crate::{symbol::Kind, text::Loc};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Entry<P, S> {
  pub path: P,
  pub loc: Loc,
  pub text: S,
  pub kind: Kind,
}

impl<P, S> Entry<P, S> {
  pub fn new(path: P, loc: Loc, text: S, kind: Kind) -> Self {
    Self { path, loc, text, kind }
  }
}

impl<P: AsRef<Path>, S: Display> Display for Entry<P, S> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{path}{SPACE}{line}{SPACE}{column}{SPACE}{text}{SPACE}{kind}{SPACE}",
      path = self.path.as_ref().to_string_lossy(),
      line = self.loc.line,
      column = self.loc.column,
      text = self.text,
      kind = self.kind.colored_abbreviation(),
    )
  }
}
