use super::*;

#[derive(Debug, Parser)]
#[command(
  name = "swab",
  version,
  author,
  about = "A configurable project cleaning tool"
)]
pub(crate) struct Arguments {
  #[arg(help = "Directories to scan for projects to clean")]
  directories: Vec<PathBuf>,
  #[clap(long, help = "Enable dry run mode")]
  dry_run: bool,
  #[clap(long, help = "Follow symlinks during traversal")]
  follow_symlinks: bool,
  #[clap(
    short,
    long,
    help = "Prompt before each task",
    conflicts_with = "quiet"
  )]
  interactive: bool,
  #[clap(
    short,
    long,
    help = "Suppress all output",
    conflicts_with = "interactive"
  )]
  quiet: bool,
  #[clap(subcommand)]
  subcommand: Option<Subcommand>,
}

impl Arguments {
  fn print_summary(&self, total_projects: u64, total_bytes: u64) {
    if self.quiet {
      return;
    }

    let (projects_label, bytes_label) = if self.dry_run {
      ("Projects matched", "Bytes matched")
    } else {
      ("Projects cleaned", "Bytes deleted")
    };

    let style = Style::stdout();

    println!(
      "{}: {}, {}: {}",
      style.apply(BOLD, projects_label),
      style.apply(CYAN, total_projects),
      style.apply(BOLD, bytes_label),
      style.apply(GREEN, Bytes(total_bytes)),
    );
  }

  fn process_context(
    &self,
    context: &Context,
    rules: &[Box<dyn Rule>],
  ) -> Result<(u64, bool)> {
    let mut seen_removals = HashSet::new();

    let reports = rules
      .iter()
      .filter(|rule| rule.detection().matches(context))
      .map(|rule| context.report(rule.as_ref()))
      .collect::<Result<Vec<_>>>()?;

    let reports = reports
      .into_iter()
      .filter(|report| !report.tasks.is_empty())
      .collect::<Vec<Report>>();

    let has_matches = !reports.is_empty();

    let (bytes, executed) = reports.iter().try_fold(
      (0u64, false),
      |(bytes, executed), report| -> Result<_> {
        if !self.quiet {
          print!("{report}");
          io::stdout().flush()?;
        }

        report.tasks.iter().try_fold(
          (bytes, executed),
          |(bytes, executed), task| -> Result<_> {
            let (task_bytes, task_executed) =
              self.process_task(task, context, &mut seen_removals)?;

            Ok((bytes + task_bytes, executed || task_executed))
          },
        )
      },
    )?;

    let should_count = if self.dry_run { has_matches } else { executed };

    Ok((bytes, should_count))
  }

  fn process_task(
    &self,
    task: &Task,
    context: &Context,
    seen_removals: &mut HashSet<PathBuf>,
  ) -> Result<(u64, bool)> {
    let (style, theme) = (Style::stdout(), ColorfulTheme::default());

    match task {
      Task::Remove { path, size } => {
        if !seen_removals.insert(path.clone()) {
          return Ok((0, false));
        }

        if self.dry_run {
          return Ok((*size, false));
        }

        let confirmation = Confirm::with_theme(&theme)
          .with_prompt(format!(
            "Remove {} ({}) in {}?",
            style.apply(CYAN, path.display()),
            style.apply(GREEN, Bytes(*size)),
            style.apply(DIM, context.root.display())
          ))
          .default(true);

        if self.interactive && !confirmation.interact()? {
          return Ok((0, false));
        }

        task.execute(context)?;

        Ok((*size, true))
      }
      Task::Command(command) => {
        if self.dry_run {
          return Ok((0, false));
        }

        let confirmation = Confirm::with_theme(&theme)
          .with_prompt(format!(
            "Run {} in {}?",
            style.apply(YELLOW, command),
            style.apply(CYAN, context.root.display())
          ))
          .default(true);

        if self.interactive && !confirmation.interact()? {
          return Ok((0, false));
        }

        task.execute(context)?;

        Ok((0, true))
      }
    }
  }

  pub(crate) fn quiet(&self) -> bool {
    self.quiet
  }

  pub(crate) fn run(self) -> Result {
    if let Some(subcommand) = self.subcommand {
      return subcommand.run();
    }

    let rules: Vec<Box<dyn Rule>> = Config::load()?.try_into()?;

    self.directories.iter().try_for_each(|root| {
      ensure!(
        root.is_dir(),
        "the path `{}` is not a valid directory",
        root.display()
      );

      Ok(())
    })?;

    let directories = self.directories.iter().try_fold(
      Vec::new(),
      |mut acc: Vec<PathBuf>, root| -> Result<Vec<PathBuf>> {
        acc.extend(root.directories(self.follow_symlinks)?);
        Ok(acc)
      },
    )?;

    let contexts = directories
      .into_iter()
      .map(|directory| Context::new(directory, self.follow_symlinks))
      .collect::<Result<Vec<_>>>()?;

    let (total_bytes, total_projects) = contexts.into_iter().try_fold(
      (0u64, 0u64),
      |totals @ (total_bytes, total_projects), context| {
        self
          .process_context(&context, &rules)
          .map(|(bytes, should_count)| {
            if should_count {
              (total_bytes + bytes, total_projects + 1)
            } else {
              totals
            }
          })
      },
    )?;

    self.print_summary(total_projects, total_bytes);

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use {
    super::*,
    clap::{CommandFactory, error::ErrorKind},
  };

  #[test]
  fn interactive_and_quiet_conflict() {
    let result = Arguments::command().try_get_matches_from([
      "swab",
      "--interactive",
      "--quiet",
    ]);

    assert!(matches!(
      result,
      Err(error) if error.kind() == ErrorKind::ArgumentConflict
    ));
  }
}
