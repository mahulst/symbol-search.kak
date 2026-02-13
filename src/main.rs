mod cache;
mod config;
mod ext;
mod fd;
mod fzf;
mod parser;
mod symbol;
mod text;
mod utils;
mod worker;

use std::{fs, path::PathBuf};

use anyhow::Context;
use clap::Parser;

use crate::{cache::Cache, config::Config, fd::Fd, worker::Worker};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
  /// A configuration TOML string.
  ///
  /// The default configuration will be applied if this argument is not provided
  /// or if it is set to the empty string.
  #[arg(short, long)]
  donfig: Option<String>,
  /// Directory to cache parsed symbols.
  ///
  /// Files are reparsed if their cached mtime differs from than their current mtime.
  /// The cache is only usable if the previously generated relative paths are still valid.
  /// This would normally only be the case when the binary is called from the same
  /// directory multiple times.
  ///
  /// This directory is created if it does not exist.
  #[arg(short, long)]
  cache_dir: Option<PathBuf>,
}

impl Args {
  /// Returns the parsed provided config or the default one.
  pub fn config(&self) -> Result<Config, anyhow::Error> {
    if let Some(config) = &self.donfig {
      if !config.is_empty() {
        return toml::from_str(config).context("from_str");
      }
    }

    Ok(Config::default())
  }

  /// Returns the provided cache or an empty one.
  pub fn cache(&self) -> Result<Cache, anyhow::Error> {
    if let Some(cache_dir) = &self.cache_dir {
      Cache::from_dir(cache_dir).context("from_str")
    } else {
      Ok(Cache::default())
    }
  }
}

fn main() -> Result<(), anyhow::Error> {
  let args = Args::parse();

  let config = Box::new(args.config().context("config")?);
  let config: &'static Config = Box::leak(config);

  let cache = args.cache().context("cache")?;

  let fd = Fd::new(config.extensions()).context("fd")?;
  let mut workers = vec![];

  for _ in 0..crate::utils::num_threads() {
    workers.push(Worker::new(config, &cache, fd.files()).run());
  }
  for worker in workers {
    worker.join().expect("Thread panicked");
  }

  let files_guard: Vec<_> = cache
    .files
    .read()
    .iter()
    .flat_map(|a| a.1.entries.iter().map(|entry| (a.0.clone(), entry.clone())))
    .collect();
  files_guard.iter().for_each(|(filename, entry)| {
    let absolute_path = fs::canonicalize(&filename).expect("");
    println!(
      "{:<10} {:<30} ({:}:{}:{})",
      format!("[{:?}]", entry.kind),
      entry.text,
      absolute_path.display(),
      entry.loc.line,
      entry.loc.column,
    );
  });
  // the cache is saved on drop
  drop(cache);

  Ok(())
}
