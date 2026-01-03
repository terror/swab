use {
  anyhow::Error,
  executable_path::executable_path,
  indoc::indoc,
  pretty_assertions::assert_eq,
  std::{fs, iter::once, process::Command, str},
  tempfile::TempDir,
};

type Result<T = (), E = Error> = std::result::Result<T, E>;

#[allow(dead_code)]
struct Test<'a> {
  arguments: Vec<String>,
  exists: Vec<&'a str>,
  expected_status: i32,
  expected_stderr: String,
  expected_stdout: String,
  files: Vec<(&'a str, &'a str)>,
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

  fn file(self, path: &'a str, content: &'a str) -> Self {
    Self {
      files: self
        .files
        .into_iter()
        .chain(once((path, content)))
        .collect(),
      ..self
    }
  }

  fn files(self, files: &[(&'a str, &'a str)]) -> Self {
    Self {
      files: self
        .files
        .into_iter()
        .chain(files.iter().copied())
        .collect(),
      ..self
    }
  }

  fn new() -> Result<Self> {
    Ok(Self {
      arguments: Vec::new(),
      exists: Vec::new(),
      expected_status: 0,
      expected_stderr: String::new(),
      expected_stdout: String::new(),
      files: Vec::new(),
      tempdir: TempDir::with_prefix("swab-test")?,
    })
  }

  fn run(self) -> Result {
    for (path, content) in &self.files {
      let full_path = self.tempdir.path().join(path);

      if let Some(parent) = full_path.parent() {
        fs::create_dir_all(parent)?;
      }

      fs::write(&full_path, content)?;
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
      .replace(&self.tempdir.path().display().to_string(), "[ROOT]")
      .replace('\\', "/");

    assert_eq!(stdout, self.expected_stdout);

    let created = self.files.iter().map(|(path, _)| *path).collect::<Vec<_>>();

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
    .file("project/Cargo.toml", "")
    .file("project/target/debug/app", &"a".repeat(1000))
    .file("project/target/release/app", &"b".repeat(500))
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

#[test]
fn cargo_removes_nested_target_directories() -> Result {
  Test::new()?
    .file("workspace/Cargo.toml", "")
    .file("workspace/target/debug/main", &"a".repeat(1000))
    .file("workspace/crates/foo/Cargo.toml", "")
    .file("workspace/crates/foo/target/debug/foo", &"b".repeat(500))
    .file("workspace/crates/bar/Cargo.toml", "")
    .file("workspace/crates/bar/target/debug/bar", &"c".repeat(500))
    .exists(&[
      "workspace/Cargo.toml",
      "workspace/crates/foo/Cargo.toml",
      "workspace/crates/bar/Cargo.toml",
    ])
    .expected_status(0)
    .expected_stdout(indoc! {
      "
      [ROOT]/workspace Cargo project (0 seconds ago)
        ├─ crates/bar/target (500 bytes)
        ├─ crates/foo/target (500 bytes)
        └─ target (1000 bytes)
      Projects cleaned: 1, Bytes deleted: 1.95 KiB
      "
    })
    .run()
}

#[test]
fn dotnet_removes_bin_and_obj() -> Result {
  Test::new()?
    .file("project/App.csproj", "")
    .file("project/bin/Debug/net8.0/App.dll", &"a".repeat(1000))
    .file("project/obj/Debug/net8.0/App.dll", &"b".repeat(500))
    .exists(&["project/App.csproj"])
    .expected_status(0)
    .expected_stdout(indoc! {
      "
      [ROOT]/project .NET project (0 seconds ago)
        ├─ bin (1000 bytes)
        └─ obj (500 bytes)
      Projects cleaned: 1, Bytes deleted: 1.46 KiB
      "
    })
    .run()
}

#[test]
fn elixir_removes_build_directories() -> Result {
  Test::new()?
    .file("project/mix.exs", "")
    .file(
      "project/_build/dev/lib/app/ebin/app.beam",
      &"a".repeat(1000),
    )
    .file("project/.elixir_ls/build/dev/lib/app.ex", &"b".repeat(500))
    .exists(&["project/mix.exs"])
    .expected_status(0)
    .expected_stdout(indoc! {
      "
      [ROOT]/project Elixir project (0 seconds ago)
        ├─ .elixir_ls (500 bytes)
        └─ _build (1000 bytes)
      Projects cleaned: 1, Bytes deleted: 1.46 KiB
      "
    })
    .run()
}

#[test]
fn gradle_removes_build_directories() -> Result {
  Test::new()?
    .file("project/build.gradle", "")
    .file("project/build/classes/main/App.class", &"a".repeat(1000))
    .file(
      "project/.gradle/8.0/checksums/checksums.lock",
      &"b".repeat(500),
    )
    .exists(&["project/build.gradle"])
    .expected_status(0)
    .expected_stdout(indoc! {
      "
      [ROOT]/project Gradle project (0 seconds ago)
        ├─ .gradle (500 bytes)
        └─ build (1000 bytes)
      Projects cleaned: 1, Bytes deleted: 1.46 KiB
      "
    })
    .run()
}

#[test]
fn gradle_kotlin_dsl() -> Result {
  Test::new()?
    .file("project/build.gradle.kts", "")
    .file("project/build/classes/main/App.class", &"a".repeat(1000))
    .exists(&["project/build.gradle.kts"])
    .expected_status(0)
    .expected_stdout(indoc! {
      "
      [ROOT]/project Gradle project (0 seconds ago)
        └─ build (1000 bytes)
      Projects cleaned: 1, Bytes deleted: 1000 bytes
      "
    })
    .run()
}

#[test]
fn maven_removes_target() -> Result {
  Test::new()?
    .file("project/pom.xml", "")
    .file(
      "project/target/classes/com/example/App.class",
      &"a".repeat(1000),
    )
    .exists(&["project/pom.xml"])
    .expected_status(0)
    .expected_stdout(indoc! {
      "
      [ROOT]/project Maven project (0 seconds ago)
        └─ target (1000 bytes)
      Projects cleaned: 1, Bytes deleted: 1000 bytes
      "
    })
    .run()
}

#[test]
fn node_removes_node_modules() -> Result {
  Test::new()?
    .file("project/package.json", "")
    .file("project/node_modules/lodash/index.js", &"a".repeat(1000))
    .file("project/node_modules/express/index.js", &"b".repeat(500))
    .exists(&["project/package.json"])
    .expected_status(0)
    .expected_stdout(indoc! {
      "
      [ROOT]/project Node project (0 seconds ago)
        └─ node_modules (1.46 KiB)
      Projects cleaned: 1, Bytes deleted: 1.46 KiB
      "
    })
    .run()
}

#[test]
fn node_removes_angular_cache() -> Result {
  Test::new()?
    .file("project/package.json", "")
    .file("project/.angular/cache/data.json", &"a".repeat(1000))
    .exists(&["project/package.json"])
    .expected_status(0)
    .expected_stdout(indoc! {
      "
      [ROOT]/project Node project (0 seconds ago)
        └─ .angular (1000 bytes)
      Projects cleaned: 1, Bytes deleted: 1000 bytes
      "
    })
    .run()
}

#[test]
fn python_removes_cache_directories() -> Result {
  Test::new()?
    .file("project/pyproject.toml", "")
    .file(
      "project/.venv/lib/python3.12/site-packages/pip.py",
      &"a".repeat(1000),
    )
    .file("project/__pycache__/main.cpython-312.pyc", &"b".repeat(500))
    .file("project/.pytest_cache/v/cache/data", &"c".repeat(200))
    .file("project/.mypy_cache/3.12/main.meta.json", &"d".repeat(100))
    .file("project/.ruff_cache/0.1.0/data", &"e".repeat(100))
    .exists(&["project/pyproject.toml"])
    .expected_status(0)
    .expected_stdout(indoc! {
      "
      [ROOT]/project Python project (0 seconds ago)
        ├─ .mypy_cache (100 bytes)
        ├─ .pytest_cache (200 bytes)
        ├─ .ruff_cache (100 bytes)
        ├─ .venv (1000 bytes)
        └─ __pycache__ (500 bytes)
      Projects cleaned: 1, Bytes deleted: 1.86 KiB
      "
    })
    .run()
}

#[test]
fn swift_removes_build_directories() -> Result {
  Test::new()?
    .file("project/Package.swift", "")
    .file("project/.build/debug/App", &"a".repeat(1000))
    .file("project/.swiftpm/xcode/xcshareddata/data", &"b".repeat(500))
    .exists(&["project/Package.swift"])
    .expected_status(0)
    .expected_stdout(indoc! {
      "
      [ROOT]/project Swift project (0 seconds ago)
        ├─ .build (1000 bytes)
        └─ .swiftpm (500 bytes)
      Projects cleaned: 1, Bytes deleted: 1.46 KiB
      "
    })
    .run()
}

#[test]
fn zig_removes_cache_directories() -> Result {
  Test::new()?
    .file("project/build.zig", "")
    .file("project/zig-cache/o/data", &"a".repeat(1000))
    .file("project/zig-out/bin/app", &"b".repeat(500))
    .exists(&["project/build.zig"])
    .expected_status(0)
    .expected_stdout(indoc! {
      "
      [ROOT]/project Zig project (0 seconds ago)
        ├─ zig-cache (1000 bytes)
        └─ zig-out (500 bytes)
      Projects cleaned: 1, Bytes deleted: 1.46 KiB
      "
    })
    .run()
}

#[test]
fn cabal_removes_dist_newstyle() -> Result {
  Test::new()?
    .file("project/cabal.project", "")
    .file(
      "project/dist-newstyle/build/x86_64-linux/ghc-9.4.7/app-0.1.0.0/build/app/app",
      &"a".repeat(1000),
    )
    .exists(&["project/cabal.project"])
    .expected_status(0)
    .expected_stdout(indoc! {
      "
      [ROOT]/project Cabal (Haskell) project (0 seconds ago)
        └─ dist-newstyle (1000 bytes)
      Projects cleaned: 1, Bytes deleted: 1000 bytes
      "
    })
    .run()
}

#[test]
fn cmake_removes_build_directories() -> Result {
  Test::new()?
    .file("project/CMakeLists.txt", "")
    .file("project/build/CMakeCache.txt", &"a".repeat(1000))
    .file("project/cmake-build-debug/app", &"b".repeat(500))
    .file("project/cmake-build-release/app", &"c".repeat(500))
    .exists(&["project/CMakeLists.txt"])
    .expected_status(0)
    .expected_stdout(indoc! {
      "
      [ROOT]/project CMake project (0 seconds ago)
        ├─ build (1000 bytes)
        ├─ cmake-build-debug (500 bytes)
        └─ cmake-build-release (500 bytes)
      Projects cleaned: 1, Bytes deleted: 1.95 KiB
      "
    })
    .run()
}

#[test]
fn composer_removes_vendor() -> Result {
  Test::new()?
    .file("project/composer.json", "")
    .file("project/vendor/autoload.php", &"a".repeat(1000))
    .file("project/vendor/composer/installed.json", &"b".repeat(500))
    .exists(&["project/composer.json"])
    .expected_status(0)
    .expected_stdout(indoc! {
      "
      [ROOT]/project Composer (PHP) project (0 seconds ago)
        └─ vendor (1.46 KiB)
      Projects cleaned: 1, Bytes deleted: 1.46 KiB
      "
    })
    .run()
}

#[test]
fn godot_removes_godot_directory() -> Result {
  Test::new()?
    .file("project/project.godot", "")
    .file("project/.godot/imported/icon.png", &"a".repeat(1000))
    .exists(&["project/project.godot"])
    .expected_status(0)
    .expected_stdout(indoc! {
      "
      [ROOT]/project Godot 4 project (0 seconds ago)
        └─ .godot (1000 bytes)
      Projects cleaned: 1, Bytes deleted: 1000 bytes
      "
    })
    .run()
}

#[test]
fn jupyter_removes_checkpoints() -> Result {
  Test::new()?
    .file("project/notebook.ipynb", "")
    .file(
      "project/.ipynb_checkpoints/notebook-checkpoint.ipynb",
      &"a".repeat(1000),
    )
    .exists(&["project/notebook.ipynb"])
    .expected_status(0)
    .expected_stdout(indoc! {
      "
      [ROOT]/project Jupyter project (0 seconds ago)
        └─ .ipynb_checkpoints (1000 bytes)
      Projects cleaned: 1, Bytes deleted: 1000 bytes
      "
    })
    .run()
}

#[test]
fn pixi_removes_pixi_directory() -> Result {
  Test::new()?
    .file("project/pixi.toml", "")
    .file("project/.pixi/envs/default/bin/python", &"a".repeat(1000))
    .exists(&["project/pixi.toml"])
    .expected_status(0)
    .expected_stdout(indoc! {
      "
      [ROOT]/project Pixi project (0 seconds ago)
        └─ .pixi (1000 bytes)
      Projects cleaned: 1, Bytes deleted: 1000 bytes
      "
    })
    .run()
}

#[test]
fn pub_removes_build_directories() -> Result {
  Test::new()?
    .file("project/pubspec.yaml", "")
    .file("project/build/app.dill", &"a".repeat(1000))
    .file("project/.dart_tool/package_config.json", &"b".repeat(500))
    .file(
      "project/linux/flutter/ephemeral/libflutter.so",
      &"c".repeat(300),
    )
    .file(
      "project/windows/flutter/ephemeral/flutter.dll",
      &"d".repeat(200),
    )
    .exists(&["project/pubspec.yaml"])
    .expected_status(0)
    .expected_stdout(indoc! {
      "
      [ROOT]/project Pub (Dart/Flutter) project (0 seconds ago)
        ├─ .dart_tool (500 bytes)
        ├─ build (1000 bytes)
        ├─ linux/flutter/ephemeral (300 bytes)
        └─ windows/flutter/ephemeral (200 bytes)
      Projects cleaned: 1, Bytes deleted: 1.95 KiB
      "
    })
    .run()
}

#[test]
fn sbt_removes_target_directories() -> Result {
  Test::new()?
    .file("project/build.sbt", "")
    .file(
      "project/target/scala-3.3.1/classes/Main.class",
      &"a".repeat(1000),
    )
    .file(
      "project/project/target/scala-2.12/sbt-1.0/classes/Build.class",
      &"b".repeat(500),
    )
    .exists(&["project/build.sbt"])
    .expected_status(0)
    .expected_stdout(indoc! {
      "
      [ROOT]/project SBT (Scala) project (0 seconds ago)
        ├─ project/target (500 bytes)
        └─ target (1000 bytes)
      Projects cleaned: 1, Bytes deleted: 1.46 KiB
      "
    })
    .run()
}

#[test]
fn stack_removes_stack_work() -> Result {
  Test::new()?
    .file("project/stack.yaml", "")
    .file(
      "project/.stack-work/install/x86_64-linux/lts-21.0/9.4.7/bin/app",
      &"a".repeat(1000),
    )
    .exists(&["project/stack.yaml"])
    .expected_status(0)
    .expected_stdout(indoc! {
      "
      [ROOT]/project Stack (Haskell) project (0 seconds ago)
        └─ .stack-work (1000 bytes)
      Projects cleaned: 1, Bytes deleted: 1000 bytes
      "
    })
    .run()
}

#[test]
fn turborepo_removes_turbo_directory() -> Result {
  Test::new()?
    .file("project/turbo.json", "")
    .file("project/.turbo/cache/data", &"a".repeat(1000))
    .exists(&["project/turbo.json"])
    .expected_status(0)
    .expected_stdout(indoc! {
      "
      [ROOT]/project Turborepo project (0 seconds ago)
        └─ .turbo (1000 bytes)
      Projects cleaned: 1, Bytes deleted: 1000 bytes
      "
    })
    .run()
}

#[test]
fn unity_removes_build_directories() -> Result {
  Test::new()?
    .file("project/Assembly-CSharp.csproj", "")
    .file(
      "project/Library/ScriptAssemblies/Assembly-CSharp.dll",
      &"a".repeat(1000),
    )
    .file("project/Temp/UnityLockfile", &"b".repeat(500))
    .file("project/Obj/Debug/Assembly-CSharp.dll", &"c".repeat(300))
    .file("project/Logs/AssetImportWorker0.log", &"d".repeat(200))
    .file("project/MemoryCaptures/capture.raw", &"e".repeat(100))
    .file("project/Build/game.exe", &"f".repeat(100))
    .file("project/Builds/game.exe", &"g".repeat(100))
    .exists(&["project/Assembly-CSharp.csproj"])
    .expected_status(0)
    .expected_stdout(indoc! {
      "
      [ROOT]/project Unity project (0 seconds ago)
        ├─ Build (100 bytes)
        ├─ Builds (100 bytes)
        ├─ Library (1000 bytes)
        ├─ Logs (200 bytes)
        ├─ MemoryCaptures (100 bytes)
        ├─ Obj (300 bytes)
        └─ Temp (500 bytes)
      Projects cleaned: 1, Bytes deleted: 2.25 KiB
      "
    })
    .run()
}

#[test]
fn unreal_removes_build_directories() -> Result {
  Test::new()?
    .file("project/MyGame.uproject", "")
    .file("project/Binaries/Win64/MyGame.exe", &"a".repeat(1000))
    .file("project/Build/WindowsNoEditor/MyGame.pak", &"b".repeat(500))
    .file("project/Saved/Logs/MyGame.log", &"c".repeat(300))
    .file("project/DerivedDataCache/DDC.bin", &"d".repeat(200))
    .file(
      "project/Intermediate/Build/Win64/MyGame.obj",
      &"e".repeat(100),
    )
    .exists(&["project/MyGame.uproject"])
    .expected_status(0)
    .expected_stdout(indoc! {
      "
      [ROOT]/project Unreal Engine project (0 seconds ago)
        ├─ Binaries (1000 bytes)
        ├─ Build (500 bytes)
        ├─ DerivedDataCache (200 bytes)
        ├─ Intermediate (100 bytes)
        └─ Saved (300 bytes)
      Projects cleaned: 1, Bytes deleted: 2.05 KiB
      "
    })
    .run()
}
