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

        let metadata = if context.follow_symlinks {
          fs::metadata(&full_path)
        } else {
          fs::symlink_metadata(&full_path)
        };

        let metadata = match metadata {
          Ok(metadata) => metadata,
          Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Ok(());
          }
          Err(error) => return Err(error.into()),
        };

        if !context.follow_symlinks && metadata.file_type().is_symlink() {
          if let Err(error) = fs::remove_file(&full_path)
            && error.kind() != io::ErrorKind::NotFound
          {
            return Err(error.into());
          }

          return Ok(());
        }

        if metadata.is_dir() {
          if let Err(error) = fs::remove_dir_all(&full_path)
            && error.kind() != io::ErrorKind::NotFound
          {
            return Err(error.into());
          }
        } else if let Err(error) = fs::remove_file(&full_path)
          && error.kind() != io::ErrorKind::NotFound
        {
          return Err(error.into());
        }

        Ok(())
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use {super::*, temptree::temptree};

  #[test]
  fn remove_is_idempotent_for_missing_paths() {
    let tree = temptree! {
      "stale.log": "x",
      "dir": {
        "file.txt": "x",
      },
    };

    let root = tree.path();

    let context = Context::new(root.to_path_buf(), false).unwrap();

    fs::remove_file(root.join("stale.log")).unwrap();
    fs::remove_dir_all(root.join("dir")).unwrap();

    let file_task = Task::Remove {
      path: PathBuf::from("stale.log"),
      size: 0,
    };

    file_task.execute(&context).unwrap();

    let dir_task = Task::Remove {
      path: PathBuf::from("dir"),
      size: 0,
    };

    dir_task.execute(&context).unwrap();
  }
}
