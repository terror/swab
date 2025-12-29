use {
  action::Action,
  anyhow::{Error, anyhow, bail, ensure},
  arguments::Arguments,
  bytes::Bytes,
  clap::Parser,
  context::Context,
  detection::Detection,
  dialoguer::{Confirm, theme::ColorfulTheme},
  globset::Glob,
  path_ext::PathExt,
  report::Report,
  rule::*,
  std::{
    backtrace::BacktraceStatus,
    collections::HashSet,
    env,
    fmt::{self, Display, Formatter},
    fs,
    io::{self, IsTerminal, Write},
    path::{Path, PathBuf},
    process::{self, Command},
    str::FromStr,
    time::{Duration, SystemTime},
  },
  style::{BOLD, CYAN, DIM, GREEN, Style, YELLOW},
  system_time_ext::SystemTimeExt,
  task::Task,
  walkdir::WalkDir,
};

mod action;
mod arguments;
mod bytes;
mod context;
mod detection;
mod path_ext;
mod report;
mod rule;
mod style;
mod system_time_ext;
mod task;

type Result<T = (), E = Error> = std::result::Result<T, E>;

fn main() {
  let arguments = Arguments::parse();

  let quiet = arguments.quiet();

  if let Err(error) = arguments.run() {
    if !quiet {
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
    }

    process::exit(1);
  }
}
