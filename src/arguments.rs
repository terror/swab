use super::*;

static RULES: &[&dyn Rule] = &[&Cargo];

#[derive(Debug, Parser)]
pub(crate) struct Arguments {
  directories: Vec<PathBuf>,
  #[clap(long, help = "Enable dry run mode")]
  dry_run: bool,
}

impl Arguments {
  pub(crate) fn run(self) -> Result {
    let mut reports = Vec::new();

    for directory in self.directories {
      ensure!(
        directory.is_dir(),
        "The path '{}' is not a valid directory.",
        directory.display()
      );

      let context = Context::try_from(directory)?;

      for rule in RULES {
        if !rule.applies(&context) {
          continue;
        }

        let report = Report::try_from((&context, *rule))?;

        if report.items.is_empty() {
          continue;
        }

        if !self.dry_run {
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

    for report in &reports {
      print!("{report}");
    }

    let total_bytes = reports.iter().map(|report| report.bytes).sum();

    let total_projects = reports.len();

    if self.dry_run {
      println!(
        "Projects matched: {total_projects}, Bytes matched: {}",
        Bytes(total_bytes)
      );
    } else {
      println!(
        "Projects cleaned: {total_projects}, Bytes deleted: {}",
        Bytes(total_bytes)
      );
    }

    Ok(())
  }
}
