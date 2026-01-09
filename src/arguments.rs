use super::*;

#[derive(Debug, Parser)]
#[command(
  name = "swab",
  version,
  author,
  about = "A configurable project cleaning tool"
)]
pub(crate) struct Arguments {
  #[clap(
    long,
    value_name = "EXPR",
    value_parser = parse_age,
    help = "Only process projects inactive for at least this age (e.g. 30d, 12h, 2w)"
  )]
  age: Option<Duration>,
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

fn parse_age(value: &str) -> Result<Duration> {
  let trimmed = value.trim();

  let core = if let Some(prefix) = trimmed.strip_suffix(" ago") {
    prefix
  } else if trimmed.ends_with("ago") {
    bail!("invalid age: expected a space before `ago`");
  } else {
    trimmed
  };

  ensure!(
    !core.is_empty() && !core.contains(char::is_whitespace),
    "invalid age: expected <amount><unit>[ ago] with no interior spaces"
  );

  let (amount_part, unit_part) = core
    .chars()
    .enumerate()
    .find(|(_, ch)| !ch.is_ascii_digit())
    .map(|(idx, _)| core.split_at(idx))
    .ok_or_else(|| anyhow!("invalid age: missing unit"))?;

  ensure!(!amount_part.is_empty(), "invalid age: missing amount");

  let amount = amount_part
    .parse::<u64>()
    .map_err(|_| anyhow!("invalid age: amount must be an integer"))?;

  let base_seconds = match unit_part {
    "s" => 1,
    "m" => 60,
    "h" => 60 * 60,
    "d" => 60 * 60 * 24,
    "w" => 60 * 60 * 24 * 7,
    "mo" => 60 * 60 * 24 * 30,
    "y" => 60 * 60 * 24 * 365,
    _ => bail!("invalid age: unit must be one of s, m, h, d, w, mo, y"),
  };

  let seconds = amount
    .checked_mul(base_seconds)
    .ok_or_else(|| anyhow!("invalid age: value is too large"))?;

  Ok(Duration::from_secs(seconds))
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

    let directories = if self.directories.is_empty() {
      vec![env::current_dir()?]
    } else {
      self.directories.clone()
    };

    directories.iter().try_for_each(|root| {
      ensure!(
        root.is_dir(),
        "the path `{}` is not a valid directory",
        root.display()
      );

      Ok(())
    })?;

    let directories = directories.iter().try_fold(
      Vec::new(),
      |mut acc: Vec<PathBuf>, root| -> Result<Vec<PathBuf>> {
        acc.push(root.clone());
        acc.extend(root.directories(self.follow_symlinks)?);
        Ok(acc)
      },
    )?;

    let contexts = directories
      .into_iter()
      .map(|directory| Context::new(directory, self.follow_symlinks))
      .collect::<Result<Vec<_>>>()?;

    let age_cutoff = self
      .age
      .and_then(|age| SystemTime::now().checked_sub(age))
      .unwrap_or(SystemTime::UNIX_EPOCH);

    let (total_bytes, total_projects) = contexts.into_iter().try_fold(
      (0u64, 0u64),
      |totals @ (total_bytes, total_projects), context| {
        if self.age.is_some() {
          let modified = context.modified_time()?;

          if modified > age_cutoff {
            return Ok(totals);
          }
        }

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

  #[test]
  fn parse_age_accepts_valid_inputs() {
    assert_eq!(
      parse_age("30d").unwrap(),
      Duration::from_secs(30 * 24 * 60 * 60)
    );
    assert_eq!(
      parse_age("12h ago").unwrap(),
      Duration::from_secs(12 * 60 * 60)
    );
    assert_eq!(
      parse_age("2w").unwrap(),
      Duration::from_secs(2 * 7 * 24 * 60 * 60)
    );
    assert_eq!(
      parse_age("1y").unwrap(),
      Duration::from_secs(365 * 24 * 60 * 60)
    );
  }

  #[test]
  fn parse_age_rejects_invalid_inputs() {
    assert!(parse_age("3x").is_err());
    assert!(parse_age("ago").is_err());
    assert!(parse_age("30 d").is_err());
    assert!(parse_age("30dago").is_err());
  }
}
