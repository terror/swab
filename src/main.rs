use {
  action::Action,
  anyhow::{Error, bail, ensure},
  arguments::Arguments,
  bytes::Bytes,
  clap::Parser,
  context::Context,
  globset::Glob,
  path_ext::PathExt,
  report::Report,
  rule::*,
  std::{
    backtrace::BacktraceStatus,
    collections::HashSet,
    fmt::{self, Display, Formatter},
    fs,
    path::{Path, PathBuf},
    process,
    time::{Duration, SystemTime},
  },
  system_time_ext::SystemTimeExt,
  walkdir::WalkDir,
};

mod action;
mod arguments;
mod bytes;
mod context;
mod path_ext;
mod report;
mod rule;
mod system_time_ext;

type Result<T = (), E = Error> = std::result::Result<T, E>;

fn main() {
  if let Err(error) = Arguments::parse().run() {
    eprintln!("error: {error}");

    for (i, error) in error.chain().skip(1).enumerate() {
      if i == 0 {
        eprintln!();
        eprintln!("because:");
      }

      eprintln!("- {error}");
    }

    let backtrace = error.backtrace();

    if backtrace.status() == BacktraceStatus::Captured {
      eprintln!("backtrace:");
      eprintln!("{backtrace}");
    }

    process::exit(1);
  }
}
