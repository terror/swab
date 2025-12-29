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
  #[clap(short, long, help = "Prompt before each task")]
  interactive: bool,
  #[clap(short, long, help = "Suppress all output")]
  quiet: bool,
}

impl Arguments {
  pub(crate) fn quiet(&self) -> bool {
    self.quiet
  }

  pub(crate) fn run(self) -> Result {
    let rules: Vec<Box<dyn Rule>> = Config::load()?.try_into()?;

    let (style, theme) = (Style::stdout(), ColorfulTheme::default());

    let (mut total_bytes, mut total_projects) = (0, 0);

    for root in self.directories {
      ensure!(
        root.is_dir(),
        "the path `{}` is not a valid directory",
        root.display()
      );

      for directory in root.directories()? {
        let context = Context::new(directory, self.follow_symlinks)?;

        let mut project_matched = false;
        let mut project_executed = false;
        let mut project_bytes = 0;
        let mut seen_removals: HashSet<PathBuf> = HashSet::new();

        for rule in &rules {
          let rule = rule.as_ref();

          if !rule.detection().matches(&context) {
            continue;
          }

          let report = context.report(rule)?;

          if report.tasks.is_empty() {
            continue;
          }

          project_matched = true;

          if !self.quiet {
            print!("{report}");
            io::stdout().flush()?;
          }

          if self.dry_run {
            for task in &report.tasks {
              if let Task::Remove { path, size } = task
                && seen_removals.insert(path.clone())
              {
                project_bytes += *size;
              }
            }
          } else {
            for task in &report.tasks {
              if let Task::Remove { path, size } = task {
                if seen_removals.contains(path) {
                  continue;
                }

                if self.interactive && !self.quiet {
                  let prompt = format!(
                    "Remove {} ({}) in {}?",
                    style.apply(CYAN, path.display()),
                    style.apply(GREEN, Bytes(*size)),
                    style.apply(DIM, context.root.display())
                  );

                  let confirmation = Confirm::with_theme(&theme)
                    .with_prompt(prompt)
                    .default(true)
                    .interact()?;

                  if !confirmation {
                    seen_removals.insert(path.clone());
                    continue;
                  }
                }

                task.execute(&context)?;
                project_executed = true;
                project_bytes += *size;
                seen_removals.insert(path.clone());
              } else {
                if self.interactive && !self.quiet {
                  let Task::Command(command) = task else {
                    continue;
                  };

                  let prompt = format!(
                    "Run {} in {}?",
                    style.apply(YELLOW, command),
                    style.apply(CYAN, context.root.display())
                  );

                  let confirmation = Confirm::with_theme(&theme)
                    .with_prompt(prompt)
                    .default(true)
                    .interact()?;

                  if !confirmation {
                    continue;
                  }
                }

                task.execute(&context)?;
                project_executed = true;
              }
            }
          }
        }

        if self.dry_run {
          if project_matched {
            total_bytes += project_bytes;
            total_projects += 1;
          }
        } else if project_executed {
          total_bytes += project_bytes;
          total_projects += 1;
        }
      }
    }

    if !self.quiet {
      if self.dry_run {
        println!(
          "{}: {}, {}: {}",
          style.apply(BOLD, "Projects matched"),
          style.apply(CYAN, total_projects),
          style.apply(BOLD, "Bytes matched"),
          style.apply(GREEN, Bytes(total_bytes)),
        );
      } else {
        println!(
          "{}: {}, {}: {}",
          style.apply(BOLD, "Projects cleaned"),
          style.apply(CYAN, total_projects),
          style.apply(BOLD, "Bytes deleted"),
          style.apply(GREEN, Bytes(total_bytes)),
        );
      }
    }

    Ok(())
  }
}
