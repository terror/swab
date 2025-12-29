use super::*;

static RULES: &[&dyn Rule] = &[
  &Cabal, &Cargo, &Cmake, &Composer, &Elixir, &Godot, &Gradle, &Maven, &Node,
  &Pixi, &Pub, &Sbt, &Stack, &Swift, &Turborepo, &Unity, &Zig,
];

#[derive(Debug, Parser)]
pub(crate) struct Arguments {
  directories: Vec<PathBuf>,
  #[clap(long, help = "Enable dry run mode")]
  dry_run: bool,
}

impl Arguments {
  fn command(command: &'static str, root: &Path) -> Result {
    let mut parts = command.split_whitespace();

    let program = parts
      .next()
      .ok_or_else(|| anyhow!("command action cannot be empty"))?;

    let status = Command::new(program)
      .args(parts)
      .current_dir(root)
      .status()?;

    ensure!(
      status.success(),
      "command `{}` failed in `{}`",
      command,
      root.display()
    );

    Ok(())
  }

  pub(crate) fn run(self) -> Result {
    let style = Style::stdout();

    let mut reports = Vec::new();

    for root in self.directories {
      ensure!(
        root.is_dir(),
        "The path '{}' is not a valid directory.",
        root.display()
      );

      for directory in root.directories()? {
        let context = Context::try_from(directory)?;

        for rule in RULES {
          if !rule.applies(&context) {
            continue;
          }

          let report = Report::try_from((&context, *rule))?;

          if report.items.is_empty() && report.commands.is_empty() {
            continue;
          }

          if !self.dry_run {
            for command in &report.commands {
              Self::command(command, &context.root)?;
            }

            for item in &report.items {
              let full_path = context.root.join(&item.1);

              if !full_path.exists() {
                continue;
              }

              if full_path.is_dir() {
                fs::remove_dir_all(&full_path)?;
              } else {
                fs::remove_file(&full_path)?;
              }
            }
          }

          reports.push(report);
        }
      }
    }

    for report in &reports {
      print!("{report}");
    }

    let total_bytes = reports.iter().map(|report| report.bytes).sum();

    let total_projects = reports.len();

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

    Ok(())
  }
}
