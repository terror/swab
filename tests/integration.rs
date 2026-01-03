use {
  Entry::*,
  anyhow::Error,
  executable_path::executable_path,
  indoc::indoc,
  pretty_assertions::assert_eq,
  std::{fs, iter::once, process::Command, str},
  tempfile::TempDir,
};

type Result<T = (), E = Error> = std::result::Result<T, E>;

#[allow(dead_code)]
#[derive(Debug, Clone)]
enum Entry<'a> {
  Directory(&'a str),
  File(&'a str, &'a str),
}

impl Entry<'_> {
  fn create(&self, tempdir: &TempDir) -> Result {
    match self {
      Self::Directory(path) => {
        fs::create_dir_all(tempdir.path().join(path))?;
        Ok(())
      }
      Self::File(path, content) => {
        let full_path = tempdir.path().join(path);

        if let Some(parent) = full_path.parent() {
          fs::create_dir_all(parent)?;
        }

        fs::write(&full_path, content)?;

        Ok(())
      }
    }
  }

  fn path(&self) -> &str {
    match self {
      Self::Directory(path) | Self::File(path, _) => path,
    }
  }
}

#[allow(dead_code)]
struct Test<'a> {
  arguments: Vec<String>,
  create: Vec<Entry<'a>>,
  exists: Vec<&'a str>,
  expected_status: i32,
  expected_stderr: String,
  expected_stdout: String,
  tempdir: TempDir,
}

#[allow(dead_code)]
impl<'a> Test<'a> {
  fn argument(self, argument: &str) -> Self {
    Self {
      arguments: self
        .arguments
        .into_iter()
        .chain(once(argument.to_owned()))
        .collect(),
      ..self
    }
  }

  fn command(&self) -> Result<Command> {
    let mut command = Command::new(executable_path(env!("CARGO_PKG_NAME")));

    command
      .env("NO_COLOR", "1")
      .current_dir(&self.tempdir)
      .arg(self.tempdir.path())
      .args(&self.arguments);

    Ok(command)
  }

  fn create(self, entries: &[Entry<'a>]) -> Self {
    Self {
      create: self
        .create
        .into_iter()
        .chain(entries.iter().cloned())
        .collect(),
      ..self
    }
  }

  fn exists(self, paths: &[&'a str]) -> Self {
    Self {
      exists: self
        .exists
        .into_iter()
        .chain(paths.iter().copied())
        .collect(),
      ..self
    }
  }

  fn expected_status(self, expected_status: i32) -> Self {
    Self {
      expected_status,
      ..self
    }
  }

  fn expected_stderr(self, expected_stderr: &str) -> Self {
    Self {
      expected_stderr: expected_stderr.to_owned(),
      ..self
    }
  }

  fn expected_stdout(self, expected_stdout: &str) -> Self {
    Self {
      expected_stdout: expected_stdout.to_owned(),
      ..self
    }
  }

  fn new() -> Result<Self> {
    Ok(Self {
      arguments: Vec::new(),
      create: Vec::new(),
      exists: Vec::new(),
      expected_status: 0,
      expected_stderr: String::new(),
      expected_stdout: String::new(),
      tempdir: TempDir::with_prefix("swab-test")?,
    })
  }

  fn run(self) -> Result {
    for entry in &self.create {
      entry.create(&self.tempdir)?;
    }

    let output = self.command()?.output()?;

    let stderr = str::from_utf8(&output.stderr)?;

    assert_eq!(
      output.status.code(),
      Some(self.expected_status),
      "unexpected exit status\nstderr: {stderr}"
    );

    if self.expected_stderr.is_empty() && !stderr.is_empty() {
      panic!("expected empty stderr: {stderr}");
    } else {
      assert_eq!(stderr, self.expected_stderr);
    }

    let stdout = str::from_utf8(&output.stdout)?
      .replace(&self.tempdir.path().display().to_string(), "[ROOT]");

    assert_eq!(stdout, self.expected_stdout);

    let created = self.create.iter().map(Entry::path).collect::<Vec<&str>>();

    for path in &created {
      assert_eq!(
        self.exists.contains(path),
        self.tempdir.path().join(path).exists(),
        "path `{path}` existence mismatch: expected exists={}, actual exists={}",
        self.exists.contains(path),
        self.tempdir.path().join(path).exists()
      );
    }

    self
      .exists
      .iter()
      .filter(|path| !created.contains(path))
      .for_each(|path| {
        assert!(
          self.tempdir.path().join(path).exists(),
          "expected path to exist: {path}"
        );
      });

    Ok(())
  }
}

#[test]
fn cargo_removes_target_directory() -> Result {
  Test::new()?
    .create(&[
      File("project/Cargo.toml", ""),
      File("project/target/debug/app", &"a".repeat(1000)),
      File("project/target/release/app", &"b".repeat(500)),
    ])
    .exists(&["project/Cargo.toml"])
    .expected_status(0)
    .expected_stdout(indoc! {
      "
      [ROOT]/project Cargo project (0 seconds ago)
        └─ target (1.46 KiB)
      Projects cleaned: 1, Bytes deleted: 1.46 KiB
      "
    })
    .run()
}
