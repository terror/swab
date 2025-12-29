use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Arguments {
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

        for rule in &rules {
          let rule = rule.as_ref();

          if !rule.detection().matches(&context) {
            continue;
          }

          let report = context.report(rule)?;

          if report.tasks.is_empty() {
            continue;
          }

          if !self.quiet {
            print!("{report}");
            io::stdout().flush()?;
          }

          if self.dry_run {
            total_bytes += report.bytes;
            total_projects += 1;
          } else {
            let mut project_bytes = 0;
            let mut project_executed = false;

            for task in &report.tasks {
              if self.interactive && !self.quiet {
                let prompt = match task {
                  Task::Command(command) => format!(
                    "Run {} in {}?",
                    style.apply(YELLOW, command),
                    style.apply(CYAN, context.root.display())
                  ),
                  Task::Remove { path, size } => format!(
                    "Remove {} ({}) in {}?",
                    style.apply(CYAN, path.display()),
                    style.apply(GREEN, Bytes(*size)),
                    style.apply(DIM, context.root.display())
                  ),
                };

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

              if let Task::Remove { size, .. } = task {
                project_bytes += *size;
              }
            }

            if project_executed {
              total_bytes += project_bytes;
              total_projects += 1;
            }
          }
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
