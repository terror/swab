use super::*;

#[derive(Debug)]
pub(crate) struct Report {
  pub(crate) modified: SystemTime,
  pub(crate) root: PathBuf,
  pub(crate) rule_name: String,
  pub(crate) tasks: Vec<Task>,
}

impl Display for Report {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let style = Style::stdout();

    let age = self.modified.format();

    writeln!(
      f,
      "{} {} project ({})",
      style.apply(CYAN, self.root.display()),
      style.apply(BOLD, self.rule_name.as_str()),
      style.apply(DIM, age),
    )?;

    let total_entries = self.tasks.len();

    for (index, task) in self.tasks.iter().enumerate() {
      let branch = if index + 1 == total_entries {
        "└─"
      } else {
        "├─"
      };

      match task {
        Task::Command(command) => {
          writeln!(
            f,
            "  {} {} {}",
            style.apply(DIM, branch),
            style.apply(DIM, "run"),
            style.apply(YELLOW, command),
          )?;
        }
        Task::Remove { path, size } => {
          writeln!(
            f,
            "  {} {} {}",
            style.apply(DIM, branch),
            path.display(),
            style.apply(GREEN, format_args!("({})", Bytes(*size))),
          )?;
        }
      }
    }

    Ok(())
  }
}
