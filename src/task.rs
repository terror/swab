use super::*;

#[derive(Debug)]
pub(crate) enum Task {
  Command(&'static str),
  Remove { path: PathBuf, size: u64 },
}

impl Task {
  pub(crate) fn execute(&self, context: &Context) -> Result {
    match self {
      Task::Command(command) => {
        let command_text = command.trim();

        ensure!(!command_text.is_empty(), "command action cannot be empty");

        let mut command = if cfg!(windows) {
          let mut command = Command::new("cmd");
          command.arg("/C").arg(command_text);
          command
        } else {
          let mut command = Command::new("sh");
          command.arg("-c").arg(command_text);
          command
        };

        let status = command.current_dir(context.root.clone()).status()?;

        ensure!(
          status.success(),
          "command `{}` failed in `{}`",
          command_text,
          context.root.display()
        );

        Ok(())
      }
      Task::Remove { path, .. } => {
        let full_path = context.root.join(path);

        ensure!(
          full_path.exists(),
          "the path `{}` does not exist",
          full_path.display()
        );

        if full_path.is_dir() {
          fs::remove_dir_all(&full_path)?;
        } else {
          fs::remove_file(&full_path)?;
        }

        Ok(())
      }
    }
  }
}
