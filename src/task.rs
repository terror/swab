use super::*;

#[derive(Debug)]
pub(crate) enum Task {
  Command(&'static str),
  Remove { path: PathBuf, size: u64 },
}

impl Task {
  fn command(command: &str, context: &Context) -> Result {
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

    let status = command.current_dir(&context.root).status()?;

    ensure!(
      status.success(),
      "command `{}` failed in `{}`",
      command_text,
      context.root.display()
    );

    Ok(())
  }

  pub(crate) fn execute(&self, context: &Context) -> Result {
    match self {
      Task::Command(command) => Self::command(command, context),
      Task::Remove { path, .. } => Self::remove(context, path),
    }
  }

  fn read_metadata(
    context: &Context,
    path: &Path,
  ) -> io::Result<Option<fs::Metadata>> {
    let result = if context.follow_symlinks {
      fs::metadata(path)
    } else {
      fs::symlink_metadata(path)
    };

    match result {
      Ok(meta) => Ok(Some(meta)),
      Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
      Err(e) => Err(e),
    }
  }

  fn remove(context: &Context, relative: &Path) -> Result {
    let path = context.root.join(relative);

    let Some(metadata) = Self::read_metadata(context, &path)? else {
      return Ok(());
    };

    if !context.follow_symlinks && metadata.file_type().is_symlink() {
      return Self::remove_file(&path);
    }

    if metadata.is_dir() {
      Self::remove_directory(&path)
    } else {
      Self::remove_file(&path)
    }
  }

  fn remove_directory(path: &Path) -> Result {
    match fs::remove_dir_all(path) {
      Ok(()) => Ok(()),
      Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
      Err(e) => Err(e.into()),
    }
  }

  fn remove_file(path: &Path) -> Result {
    match fs::remove_file(path) {
      Ok(()) => Ok(()),
      Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
      Err(e) => Err(e.into()),
    }
  }
}

#[cfg(test)]
mod tests {
  use {super::*, tempfile::tempdir};

  #[test]
  fn remove_is_idempotent_for_missing_paths() {
    let tempdir = tempdir().unwrap();

    let context = Context::new(tempdir.path().to_path_buf(), false).unwrap();

    let file_task = Task::Remove {
      path: PathBuf::from("stale.log"),
      size: 0,
    };

    file_task.execute(&context).unwrap();

    let directory_task = Task::Remove {
      path: PathBuf::from("dir"),
      size: 0,
    };

    directory_task.execute(&context).unwrap();
  }
}
