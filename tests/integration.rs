use {
  anyhow::Error,
  filetime::{self, FileTime},
  indoc::indoc,
  pretty_assertions::assert_eq,
  std::{
    fs,
    process::Command,
    str,
    time::{Duration, SystemTime},
  },
  tempfile::TempDir,
};

type Result<T = (), E = Error> = std::result::Result<T, E>;

#[derive(Debug)]
struct Test<'a> {
  age: Option<Duration>,
  arguments: Vec<String>,
  directory: Option<String>,
  exists: Vec<&'a str>,
  expected_status: i32,
  expected_stderr: String,
  expected_stdout: String,
  files: Vec<(&'a str, &'a str)>,
  tempdir: TempDir,
}

impl<'a> Test<'a> {
  fn age(mut self, age: Duration) -> Self {
    self.age = Some(age);

    self
  }

  fn argument(mut self, argument: &str) -> Self {
    self.arguments.push(argument.to_owned());

    self
  }

  fn command(&self) -> Result<Command> {
    let mut command = Command::new(env!("CARGO_BIN_EXE_swab"));

    command
      .env("NO_COLOR", "1")
      .env("RUST_BACKTRACE", "0")
      .env("APPDATA", self.tempdir.path())
      .env("HOME", self.tempdir.path())
      .env("LOCALAPPDATA", self.tempdir.path())
      .env("XDG_CONFIG_HOME", self.tempdir.path())
      .current_dir(&self.tempdir);

    if let Some(dir) = &self.directory {
      command.arg(self.tempdir.path().join(dir));
    } else {
      command.arg(self.tempdir.path());
    }

    command.args(&self.arguments);

    Ok(command)
  }

  fn directory(mut self, directory: &str) -> Self {
    self.directory = Some(directory.to_owned());

    self
  }

  fn exists(mut self, paths: &[&'a str]) -> Self {
    self.exists.extend_from_slice(paths);

    self
  }

  fn expected_status(mut self, expected_status: i32) -> Self {
    self.expected_status = expected_status;

    self
  }

  fn expected_stderr(mut self, expected_stderr: &str) -> Self {
    expected_stderr.clone_into(&mut self.expected_stderr);

    self
  }

  fn expected_stdout(mut self, expected_stdout: &str) -> Self {
    expected_stdout.clone_into(&mut self.expected_stdout);

    self
  }

  fn file(mut self, path: &'a str, content: &'a str) -> Self {
    self.files.push((path, content));

    self
  }

  fn new() -> Result<Self> {
    Ok(Self {
      age: None,
      arguments: Vec::new(),
      directory: None,
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

    if let Some(age) = self.age {
      let mtime = FileTime::from_system_time(SystemTime::now() - age);

      for (path, _) in &self.files {
        let full_path = self.tempdir.path().join(path);

        filetime::set_file_mtime(&full_path, mtime)?;

        if let Some(parent) = full_path.parent() {
          filetime::set_file_mtime(parent, mtime)?;
        }
      }

      filetime::set_file_mtime(self.tempdir.path(), mtime)?;
    }

    let output = self.command()?.output()?;

    let stderr = str::from_utf8(&output.stderr)?
      .replace(&self.tempdir.path().display().to_string(), "[ROOT]")
      .replace('\\', "/");

    assert_eq!(
      output.status.code(),
      Some(self.expected_status),
      "unexpected exit status\nstderr: {stderr}"
    );

    assert_eq!(stderr, self.expected_stderr);

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
fn buck2_removes_buck_out() -> Result {
  Test::new()?
    .file("project/.buckconfig", "")
    .file("project/buck-out/v2/gen/app", &"a".repeat(1000))
    .exists(&["project/.buckconfig"])
    .expected_stdout(indoc! {
      "
      [ROOT]/project Buck2 project (0 seconds ago)
        └─ buck-out (1000 bytes)
      Projects cleaned: 1, Bytes deleted: 1000 bytes
      "
    })
    .run()
}

#[test]
fn cargo_removes_target_directory() -> Result {
  Test::new()?
    .file("project/Cargo.toml", "")
    .file("project/target/debug/app", &"a".repeat(1000))
    .file("project/target/release/app", &"b".repeat(500))
    .exists(&["project/Cargo.toml"])
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
fn cargo_removes_target_directory_at_root() -> Result {
  Test::new()?
    .file("Cargo.toml", "")
    .file("target/debug/app", &"a".repeat(1000))
    .file("target/release/app", &"b".repeat(500))
    .exists(&["Cargo.toml"])
    .expected_stdout(indoc! {
      "
      [ROOT] Cargo project (0 seconds ago)
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
    .directory("project")
    .file("project/App.csproj", "")
    .file("project/bin/Debug/net8.0/App.dll", &"a".repeat(1000))
    .file("project/obj/Debug/net8.0/App.dll", &"b".repeat(500))
    .exists(&["project/App.csproj"])
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
fn dune_removes_build_directory() -> Result {
  Test::new()?
    .file("project/dune-project", "")
    .file("project/_build/default/bin/main.exe", &"a".repeat(1000))
    .file("workspace/dune-workspace", "")
    .file("workspace/_build/default/lib/foo.cma", &"b".repeat(500))
    .exists(&["project/dune-project", "workspace/dune-workspace"])
    .expected_stdout(indoc! {
      "
      [ROOT]/project Dune (OCaml) project (0 seconds ago)
        └─ _build (1000 bytes)
      [ROOT]/workspace Dune (OCaml) project (0 seconds ago)
        └─ _build (500 bytes)
      Projects cleaned: 2, Bytes deleted: 1.46 KiB
      "
    })
    .run()
}

#[test]
fn dotnet_detects_visual_basic_projects() -> Result {
  Test::new()?
    .directory("project")
    .file("project/App.vbproj", "")
    .file("project/bin/Debug/net8.0/App.dll", &"a".repeat(1000))
    .exists(&["project/App.vbproj"])
    .expected_stdout(indoc! {
      "
      [ROOT]/project .NET project (0 seconds ago)
        └─ bin (1000 bytes)
      Projects cleaned: 1, Bytes deleted: 1000 bytes
      "
    })
    .run()
}

#[test]
fn dotnet_removes_nested_project_outputs() -> Result {
  Test::new()?
    .directory("solution")
    .file("solution/App.sln", "")
    .file("solution/src/App/App.csproj", "")
    .file(
      "solution/src/App/bin/Debug/net8.0/App.dll",
      &"a".repeat(1000),
    )
    .file("solution/tests/App.Tests/App.Tests.fsproj", "")
    .file(
      "solution/tests/App.Tests/obj/Debug/net8.0/App.Tests.dll",
      &"b".repeat(500),
    )
    .exists(&[
      "solution/App.sln",
      "solution/src/App/App.csproj",
      "solution/tests/App.Tests/App.Tests.fsproj",
    ])
    .expected_stdout(indoc! {
      "
      [ROOT]/solution/src/App .NET project (0 seconds ago)
        └─ bin (1000 bytes)
      [ROOT]/solution/tests/App.Tests .NET project (0 seconds ago)
        └─ obj (500 bytes)
      Projects cleaned: 2, Bytes deleted: 1.46 KiB
      "
    })
    .run()
}

#[test]
fn dotnet_does_not_remove_outputs_from_sibling_directories() -> Result {
  Test::new()?
    .file("a/App.csproj", "")
    .file("a/bin/App.dll", &"a".repeat(1000))
    .file("b/bin/Other.dll", &"b".repeat(500))
    .exists(&["a/App.csproj", "b/bin/Other.dll"])
    .expected_stdout(indoc! {
      "
      [ROOT]/a .NET project (0 seconds ago)
        └─ bin (1000 bytes)
      Projects cleaned: 1, Bytes deleted: 1000 bytes
      "
    })
    .run()
}

#[test]
fn elixir_removes_build_and_dependency_directories() -> Result {
  Test::new()?
    .file("project/mix.exs", "")
    .file(
      "project/_build/dev/lib/app/ebin/app.beam",
      &"a".repeat(1000),
    )
    .file("project/.elixir_ls/build/dev/lib/app.ex", &"b".repeat(500))
    .file("project/deps/foo/ebin/foo.beam", &"c".repeat(300))
    .exists(&["project/mix.exs"])
    .expected_stdout(indoc! {
      "
      [ROOT]/project Elixir project (0 seconds ago)
        ├─ .elixir_ls (500 bytes)
        ├─ _build (1000 bytes)
        └─ deps (300 bytes)
      Projects cleaned: 1, Bytes deleted: 1.76 KiB
      "
    })
    .run()
}

#[test]
fn elm_removes_elm_stuff_directory() -> Result {
  Test::new()?
    .file("foo/elm.json", "")
    .file("foo/elm-stuff/bar", "baz")
    .file("foo/src/bar.elm", "baz")
    .file("bar/elm-stuff/foo", "baz")
    .exists(&["foo/elm.json", "foo/src/bar.elm", "bar/elm-stuff/foo"])
    .expected_stdout(indoc! {
      "
      [ROOT]/foo Elm project (0 seconds ago)
        └─ elm-stuff (3 bytes)
      Projects cleaned: 1, Bytes deleted: 3 bytes
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
fn gradle_multi_project_builds() -> Result {
  Test::new()?
    .file("groovy/settings.gradle", "")
    .file("groovy/app/build/classes/main/App.class", &"a".repeat(1000))
    .file("kotlin/settings.gradle.kts", "")
    .file("kotlin/lib/build/classes/main/Lib.class", &"b".repeat(500))
    .exists(&["groovy/settings.gradle", "kotlin/settings.gradle.kts"])
    .expected_stdout(indoc! {
      "
      [ROOT]/groovy Gradle project (0 seconds ago)
        └─ app/build (1000 bytes)
      [ROOT]/kotlin Gradle project (0 seconds ago)
        └─ lib/build (500 bytes)
      Projects cleaned: 2, Bytes deleted: 1.46 KiB
      "
    })
    .run()
}

#[test]
fn maven_removes_target() -> Result {
  Test::new()?
    .file("project/pom.xml", "")
    .file("project/module/pom.xml", "")
    .file(
      "project/target/classes/com/example/App.class",
      &"a".repeat(1000),
    )
    .file(
      "project/module/target/classes/com/example/Module.class",
      &"b".repeat(500),
    )
    .exists(&["project/pom.xml", "project/module/pom.xml"])
    .expected_stdout(indoc! {
      "
      [ROOT]/project Maven project (0 seconds ago)
        ├─ module/target (500 bytes)
        └─ target (1000 bytes)
      Projects cleaned: 1, Bytes deleted: 1.46 KiB
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
    .file("project/.angular/config.json", "bar")
    .exists(&["project/package.json", "project/.angular/config.json"])
    .expected_stdout(indoc! {
      "
      [ROOT]/project Node project (0 seconds ago)
        └─ .angular/cache (1000 bytes)
      Projects cleaned: 1, Bytes deleted: 1000 bytes
      "
    })
    .run()
}

#[test]
fn nextjs_removes_next_directory() -> Result {
  Test::new()?
    .file("project/package.json", "{}")
    .file("project/next.config.ts", "export default {}")
    .file("project/.next/cache/data", &"a".repeat(1000))
    .file("project/out/index.html", "bar")
    .exists(&[
      "project/package.json",
      "project/next.config.ts",
      "project/out/index.html",
    ])
    .expected_stdout(indoc! {
      "
      [ROOT]/project Next.js project (0 seconds ago)
        └─ .next (1000 bytes)
      Projects cleaned: 1, Bytes deleted: 1000 bytes
      "
    })
    .run()
}

#[test]
fn nuxt_removes_generated_directories() -> Result {
  Test::new()?
    .file("foo/package.json", "")
    .file("foo/nuxt.config.ts", "")
    .file("foo/.nuxt/bar", "baz")
    .file("foo/.output/bar", "baz")
    .file("foo/pages/bar.vue", "baz")
    .file("bar/package.json", "")
    .file("bar/.nuxt/foo", "baz")
    .file("bar/.output/foo", "baz")
    .file("baz/nuxt.config.ts", "")
    .file("baz/.nuxt/foo", "bar")
    .file("baz/.output/foo", "bar")
    .exists(&[
      "foo/package.json",
      "foo/nuxt.config.ts",
      "foo/pages/bar.vue",
      "bar/package.json",
      "bar/.nuxt/foo",
      "bar/.output/foo",
      "baz/nuxt.config.ts",
      "baz/.nuxt/foo",
      "baz/.output/foo",
    ])
    .expected_stdout(indoc! {
      "
      [ROOT]/foo Nuxt project (0 seconds ago)
        ├─ .nuxt (3 bytes)
        └─ .output (3 bytes)
      Projects cleaned: 1, Bytes deleted: 6 bytes
      "
    })
    .run()
}

#[test]
fn nx_removes_cache_and_workspace_data() -> Result {
  Test::new()?
    .file("project/nx.json", "")
    .file("project/.nx/cache/foo", &"a".repeat(1000))
    .file("project/.nx/workspace-data/bar", &"b".repeat(500))
    .file("project/.nx/foo/bar", "baz")
    .exists(&["project/nx.json", "project/.nx/foo/bar"])
    .expected_stdout(indoc! {
      "
      [ROOT]/project Nx project (0 seconds ago)
        ├─ .nx/cache (1000 bytes)
        └─ .nx/workspace-data (500 bytes)
      Projects cleaned: 1, Bytes deleted: 1.46 KiB
      "
    })
    .run()
}

#[test]
fn parcel_removes_cache_directory() -> Result {
  Test::new()?
    .file("foo/package.json", "")
    .file("foo/.parcel-cache/bar", "baz")
    .file("foo/dist/bar", "baz")
    .file("bar/.parcel-cache/foo", "baz")
    .exists(&["foo/package.json", "foo/dist/bar", "bar/.parcel-cache/foo"])
    .expected_stdout(indoc! {
      "
      [ROOT]/foo Parcel project (0 seconds ago)
        └─ .parcel-cache (3 bytes)
      Projects cleaned: 1, Bytes deleted: 3 bytes
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
    .file(
      "project/src/foo/__pycache__/main.cpython-312.pyc",
      &"b".repeat(500),
    )
    .file("project/.pytest_cache/v/cache/data", &"c".repeat(200))
    .file("project/.mypy_cache/3.12/main.meta.json", &"d".repeat(100))
    .file("project/.ruff_cache/0.1.0/data", &"e".repeat(100))
    .exists(&["project/pyproject.toml"])
    .expected_stdout(indoc! {
      "
      [ROOT]/project Python project (0 seconds ago)
        ├─ .mypy_cache (100 bytes)
        ├─ .pytest_cache (200 bytes)
        ├─ .ruff_cache (100 bytes)
        ├─ .venv (1000 bytes)
        └─ src/foo/__pycache__ (500 bytes)
      Projects cleaned: 1, Bytes deleted: 1.86 KiB
      "
    })
    .run()
}

#[test]
fn python_detects_setup_project_files() -> Result {
  Test::new()?
    .file("foo/setup.py", "")
    .file("foo/__pycache__/foo.pyc", &"a".repeat(500))
    .file("bar/setup.cfg", "")
    .file("bar/__pycache__/bar.pyc", &"b".repeat(300))
    .exists(&["foo/setup.py", "bar/setup.cfg"])
    .expected_stdout(indoc! {
      "
      [ROOT]/bar Python project (0 seconds ago)
        └─ __pycache__ (300 bytes)
      [ROOT]/foo Python project (0 seconds ago)
        └─ __pycache__ (500 bytes)
      Projects cleaned: 2, Bytes deleted: 800 bytes
      "
    })
    .run()
}

#[test]
fn sveltekit_removes_generated_directory() -> Result {
  Test::new()?
    .file("foo/package.json", "")
    .file("foo/svelte.config.js", "")
    .file("foo/.svelte-kit/bar", "baz")
    .file("foo/src/bar.svelte", "baz")
    .file("bar/package.json", "")
    .file("bar/.svelte-kit/foo", "baz")
    .file("baz/svelte.config.js", "")
    .file("baz/.svelte-kit/foo", "bar")
    .exists(&[
      "foo/package.json",
      "foo/svelte.config.js",
      "foo/src/bar.svelte",
      "bar/package.json",
      "bar/.svelte-kit/foo",
      "baz/svelte.config.js",
      "baz/.svelte-kit/foo",
    ])
    .expected_stdout(indoc! {
      "
      [ROOT]/foo SvelteKit project (0 seconds ago)
        └─ .svelte-kit (3 bytes)
      Projects cleaned: 1, Bytes deleted: 3 bytes
      "
    })
    .run()
}

#[test]
fn swift_removes_build_directory_and_preserves_configuration() -> Result {
  Test::new()?
    .file("project/Package.swift", "")
    .file("project/.build/debug/foo", "bar")
    .file("project/.swiftpm/configuration/mirrors.json", "foo")
    .file("project/.swiftpm/xcode/xcshareddata/foo", "bar")
    .exists(&[
      "project/Package.swift",
      "project/.swiftpm/configuration/mirrors.json",
      "project/.swiftpm/xcode/xcshareddata/foo",
    ])
    .expected_stdout(indoc! {
      "
      [ROOT]/project Swift project (0 seconds ago)
        └─ .build (3 bytes)
      Projects cleaned: 1, Bytes deleted: 3 bytes
      "
    })
    .run()
}

#[test]
fn terraform_removes_generated_directory() -> Result {
  Test::new()?
    .file("lock-project/.terraform.lock.hcl", "foo")
    .file("lock-project/.terraform/providers/provider", "foo")
    .file("lock-project/terraform.tfstate", "bar")
    .file("lock-project/terraform.tfstate.backup", "baz")
    .file("lock-project/saved.tfplan", "qux")
    .file("tf-project/main.tf", "foo")
    .file("tf-project/.terraform/modules/modules.json", "bar")
    .exists(&[
      "lock-project/.terraform.lock.hcl",
      "lock-project/terraform.tfstate",
      "lock-project/terraform.tfstate.backup",
      "lock-project/saved.tfplan",
      "tf-project/main.tf",
    ])
    .expected_stdout(indoc! {
      "
      [ROOT]/lock-project Terraform project (0 seconds ago)
        └─ .terraform (3 bytes)
      [ROOT]/tf-project Terraform project (0 seconds ago)
        └─ .terraform (3 bytes)
      Projects cleaned: 2, Bytes deleted: 6 bytes
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
    .file("standalone/foo.cabal", "")
    .file("standalone/dist-newstyle/build/foo", &"b".repeat(500))
    .exists(&["project/cabal.project", "standalone/foo.cabal"])
    .expected_stdout(indoc! {
      "
      [ROOT]/project Cabal (Haskell) project (0 seconds ago)
        └─ dist-newstyle (1000 bytes)
      [ROOT]/standalone Cabal (Haskell) project (0 seconds ago)
        └─ dist-newstyle (500 bytes)
      Projects cleaned: 2, Bytes deleted: 1.46 KiB
      "
    })
    .run()
}

#[test]
fn cabal_detection_does_not_cross_directories() -> Result {
  Test::new()?
    .file("nested/foo.cabal", "")
    .file("nested/dist-newstyle/app", &"a".repeat(1000))
    .file("dist-newstyle/unrelated", &"b".repeat(500))
    .exists(&["nested/foo.cabal", "dist-newstyle/unrelated"])
    .expected_stdout(indoc! {
      "
      [ROOT]/nested Cabal (Haskell) project (0 seconds ago)
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
fn gleam_removes_root_build_directory() -> Result {
  Test::new()?
    .file("foo/gleam.toml", "")
    .file("foo/manifest.toml", "")
    .file("foo/build/bar", "baz")
    .file("foo/src/bar.gleam", "baz")
    .file("foo/src/build/bar", "baz")
    .file("bar/build/foo", "baz")
    .exists(&[
      "foo/gleam.toml",
      "foo/manifest.toml",
      "foo/src/bar.gleam",
      "foo/src/build/bar",
      "bar/build/foo",
    ])
    .expected_stdout(indoc! {
      "
      [ROOT]/foo Gleam project (0 seconds ago)
        └─ build (3 bytes)
      Projects cleaned: 1, Bytes deleted: 3 bytes
      "
    })
    .run()
}

#[test]
fn godot_removes_godot_directory() -> Result {
  Test::new()?
    .file("project/project.godot", "")
    .file("project/App.csproj", "")
    .file("project/.godot/imported/icon.png", &"a".repeat(1000))
    .file("project/bin/Debug/net8.0/App.dll", "bar")
    .exists(&[
      "project/project.godot",
      "project/App.csproj",
      "project/bin/Debug/net8.0/App.dll",
    ])
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
fn pixi_removes_environments_and_preserves_configuration() -> Result {
  #[track_caller]
  fn case(manifest: &'static str) -> Result {
    Test::new()?
      .file(manifest, "")
      .file("project/.pixi/config.toml", "foo")
      .file("project/.pixi/envs/foo/bin/bar", "baz")
      .exists(&[manifest, "project/.pixi/config.toml"])
      .expected_stdout(indoc! {
        "
        [ROOT]/project Pixi project (0 seconds ago)
          └─ .pixi/envs (3 bytes)
        Projects cleaned: 1, Bytes deleted: 3 bytes
        "
      })
      .run()
  }

  case("project/pixi.toml")?;
  case("project/pyproject.toml")?;

  Ok(())
}

#[test]
fn pub_removes_build_directories() -> Result {
  Test::new()?
    .file("project/pubspec.yaml", "")
    .file("project/build/app.dill", &"a".repeat(1000))
    .file("project/.dart_tool/package_config.json", &"b".repeat(500))
    .file("project/.android/app/build.gradle", &"c".repeat(100))
    .file("project/.flutter-plugins-dependencies", &"d".repeat(100))
    .file(
      "project/.ios/Runner.xcodeproj/project.pbxproj",
      &"e".repeat(100),
    )
    .file("project/ios/Flutter/App.framework/App", &"f".repeat(100))
    .file(
      "project/ios/Flutter/Flutter.framework/Flutter",
      &"g".repeat(100),
    )
    .file("project/ios/Flutter/Flutter.podspec", &"h".repeat(100))
    .file("project/ios/Flutter/Generated.xcconfig", &"i".repeat(100))
    .file("project/ios/Flutter/ephemeral/foo", &"j".repeat(100))
    .file(
      "project/ios/Flutter/flutter_export_environment.sh",
      &"k".repeat(100),
    )
    .file(
      "project/linux/flutter/ephemeral/libflutter.so",
      &"l".repeat(300),
    )
    .file("project/macos/Flutter/ephemeral/foo", &"m".repeat(100))
    .file(
      "project/windows/flutter/ephemeral/flutter.dll",
      &"n".repeat(200),
    )
    .file("project/android/app/build.gradle", "")
    .file("project/ios/Runner/AppDelegate.swift", "")
    .file("project/macos/Runner/AppDelegate.swift", "")
    .exists(&[
      "project/pubspec.yaml",
      "project/android/app/build.gradle",
      "project/ios/Runner/AppDelegate.swift",
      "project/macos/Runner/AppDelegate.swift",
    ])
    .expected_stdout(indoc! {
      "
      [ROOT]/project Pub (Dart/Flutter) project (0 seconds ago)
        ├─ .android (100 bytes)
        ├─ .dart_tool (500 bytes)
        ├─ .flutter-plugins-dependencies (100 bytes)
        ├─ .ios (100 bytes)
        ├─ build (1000 bytes)
        ├─ ios/Flutter/App.framework (100 bytes)
        ├─ ios/Flutter/Flutter.framework (100 bytes)
        ├─ ios/Flutter/Flutter.podspec (100 bytes)
        ├─ ios/Flutter/Generated.xcconfig (100 bytes)
        ├─ ios/Flutter/ephemeral (100 bytes)
        ├─ ios/Flutter/flutter_export_environment.sh (100 bytes)
        ├─ linux/flutter/ephemeral (300 bytes)
        ├─ macos/Flutter/ephemeral (100 bytes)
        └─ windows/flutter/ephemeral (200 bytes)
      Projects cleaned: 1, Bytes deleted: 2.93 KiB
      "
    })
    .run()
}

#[test]
fn rebar3_removes_build_directory() -> Result {
  Test::new()?
    .file("project/rebar.config", "")
    .file(
      "project/_build/default/lib/foo/ebin/foo.beam",
      &"a".repeat(1000),
    )
    .exists(&["project/rebar.config"])
    .expected_stdout(indoc! {
      "
      [ROOT]/project Rebar3 (Erlang) project (0 seconds ago)
        └─ _build (1000 bytes)
      Projects cleaned: 1, Bytes deleted: 1000 bytes
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
    .file(
      "project/module/target/scala-3.3.1/classes/Module.class",
      &"c".repeat(300),
    )
    .exists(&["project/build.sbt"])
    .expected_stdout(indoc! {
      "
      [ROOT]/project sbt (Scala) project (0 seconds ago)
        ├─ module/target (300 bytes)
        ├─ project/target (500 bytes)
        └─ target (1000 bytes)
      Projects cleaned: 1, Bytes deleted: 1.76 KiB
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
fn terragrunt_removes_nested_caches() -> Result {
  Test::new()?
    .file("project/terragrunt.hcl", "")
    .file("project/.terragrunt-cache/foo/data", &"a".repeat(1000))
    .file("project/foo/terragrunt.hcl", "")
    .file("project/foo/.terragrunt-cache/bar/data", &"b".repeat(500))
    .file(
      "project/foo/bar/.terragrunt-cache/baz/data",
      &"c".repeat(300),
    )
    .exists(&["project/terragrunt.hcl", "project/foo/terragrunt.hcl"])
    .expected_stdout(indoc! {
      "
      [ROOT]/project Terragrunt project (0 seconds ago)
        ├─ .terragrunt-cache (1000 bytes)
        ├─ foo/.terragrunt-cache (500 bytes)
        └─ foo/bar/.terragrunt-cache (300 bytes)
      Projects cleaned: 1, Bytes deleted: 1.76 KiB
      "
    })
    .run()
}

#[test]
fn turborepo_removes_cache_and_preserves_configuration() -> Result {
  #[track_caller]
  fn case(manifest: &'static str) -> Result {
    Test::new()?
      .file(manifest, "")
      .file("project/.turbo/cache/foo", "bar")
      .file("project/.turbo/config.json", "foo")
      .exists(&[manifest, "project/.turbo/config.json"])
      .expected_stdout(indoc! {
        "
        [ROOT]/project Turborepo project (0 seconds ago)
          └─ .turbo/cache (3 bytes)
        Projects cleaned: 1, Bytes deleted: 3 bytes
        "
      })
      .run()
  }

  case("project/turbo.json")?;
  case("project/turbo.jsonc")?;

  Ok(())
}

#[test]
fn unity_removes_build_directories() -> Result {
  Test::new()?
    .file("project/Assembly-CSharp.csproj", "")
    .file("project/bin/Debug/Assembly-CSharp.dll", "bar")
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
    .exists(&[
      "project/Assembly-CSharp.csproj",
      "project/bin/Debug/Assembly-CSharp.dll",
    ])
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
fn unreal_removes_generated_directories_and_preserves_project_data() -> Result {
  Test::new()?
    .file("project/foo.uproject", "")
    .file("project/Binaries/Win64/foo.exe", "bar")
    .file("project/Build/Android/foo.keystore", "bar")
    .file("project/Build/Windows/foo.ico", "bar")
    .file("project/Build/foo", "bar")
    .file("project/DerivedDataCache/foo", "bar")
    .file("project/Intermediate/Build/Win64/foo.obj", "bar")
    .file("project/Saved/Autosaves/foo.umap", "bar")
    .file("project/Saved/Backup/foo.uasset", "bar")
    .file("project/Saved/Config/Windows/Editor.ini", "bar")
    .file("project/Saved/Cooked/WindowsNoEditor/foo.uasset", "bar")
    .file("project/Saved/Logs/foo.log", "bar")
    .file("project/Saved/SaveGames/foo.sav", "bar")
    .file("project/Saved/StagedBuilds/WindowsNoEditor/foo.pak", "bar")
    .file("project/Saved/foo/bar", "bar")
    .file("project/Saved/foo.log", "bar")
    .exists(&[
      "project/foo.uproject",
      "project/Build",
      "project/Build/Android/foo.keystore",
      "project/Build/Windows/foo.ico",
      "project/Build/foo",
      "project/Saved",
      "project/Saved/Autosaves/foo.umap",
      "project/Saved/Backup/foo.uasset",
      "project/Saved/Config/Windows/Editor.ini",
      "project/Saved/SaveGames/foo.sav",
      "project/Saved/foo/bar",
      "project/Saved/foo.log",
    ])
    .expected_stdout(indoc! {
      "
      [ROOT]/project Unreal Engine project (0 seconds ago)
        ├─ Binaries (3 bytes)
        ├─ DerivedDataCache (3 bytes)
        ├─ Intermediate (3 bytes)
        ├─ Saved/Cooked (3 bytes)
        ├─ Saved/Logs (3 bytes)
        └─ Saved/StagedBuilds (3 bytes)
      Projects cleaned: 1, Bytes deleted: 18 bytes
      "
    })
    .run()
}

#[test]
fn unreal_does_not_remove_ancestor_outputs() -> Result {
  Test::new()?
    .file("nested/foo.uproject", "")
    .file("nested/Saved/Logs/foo.log", "bar")
    .file("Build/foo", "bar")
    .file("Saved/Logs/foo.log", "bar")
    .exists(&[
      "nested/foo.uproject",
      "nested/Saved",
      "Build/foo",
      "Saved/Logs/foo.log",
    ])
    .expected_stdout(indoc! {
      "
      [ROOT]/nested Unreal Engine project (0 seconds ago)
        └─ Saved/Logs (3 bytes)
      Projects cleaned: 1, Bytes deleted: 3 bytes
      "
    })
    .run()
}

#[test]
fn vcpkg_removes_installed_directory() -> Result {
  Test::new()?
    .file("project/vcpkg.json", "")
    .file(
      "project/vcpkg_installed/x64-linux/lib/libfoo.a",
      &"a".repeat(1000),
    )
    .exists(&["project/vcpkg.json"])
    .expected_stdout(indoc! {
      "
      [ROOT]/project vcpkg project (0 seconds ago)
        └─ vcpkg_installed (1000 bytes)
      Projects cleaned: 1, Bytes deleted: 1000 bytes
      "
    })
    .run()
}

#[test]
fn dry_run_does_not_delete_files() -> Result {
  Test::new()?
    .argument("--dry-run")
    .file("project/Cargo.toml", "")
    .file("project/target/debug/app", &"a".repeat(1000))
    .exists(&["project/Cargo.toml", "project/target/debug/app"])
    .expected_stdout(indoc! {
      "
      [ROOT]/project Cargo project (0 seconds ago)
        └─ target (1000 bytes)
      Projects matched: 1, Bytes matched: 1000 bytes
      "
    })
    .run()
}

#[test]
fn quiet_mode_suppresses_output() -> Result {
  Test::new()?
    .argument("--quiet")
    .file("project/Cargo.toml", "")
    .file("project/target/debug/app", &"a".repeat(1000))
    .exists(&["project/Cargo.toml"])
    .expected_stdout("")
    .run()
}

#[test]
fn no_matching_projects() -> Result {
  Test::new()?
    .file("project/README.md", "# Hello")
    .exists(&["project/README.md"])
    .expected_stdout(indoc! {
      "
      Projects cleaned: 0, Bytes deleted: 0 bytes
      "
    })
    .run()
}

#[test]
fn multiple_projects_different_rules() -> Result {
  Test::new()?
    .file("rust-app/Cargo.toml", "")
    .file("rust-app/target/debug/app", &"a".repeat(1000))
    .file("node-app/package.json", "")
    .file("node-app/node_modules/lodash/index.js", &"b".repeat(500))
    .file("python-app/pyproject.toml", "")
    .file("python-app/.venv/bin/python", &"c".repeat(300))
    .exists(&[
      "rust-app/Cargo.toml",
      "node-app/package.json",
      "python-app/pyproject.toml",
    ])
    .expected_stdout(indoc! {
      "
      [ROOT]/node-app Node project (0 seconds ago)
        └─ node_modules (500 bytes)
      [ROOT]/python-app Python project (0 seconds ago)
        └─ .venv (300 bytes)
      [ROOT]/rust-app Cargo project (0 seconds ago)
        └─ target (1000 bytes)
      Projects cleaned: 3, Bytes deleted: 1.76 KiB
      "
    })
    .run()
}

#[test]
fn multiple_projects_same_rule() -> Result {
  Test::new()?
    .file("frontend/package.json", "")
    .file("frontend/node_modules/react/index.js", &"a".repeat(1000))
    .file("backend/package.json", "")
    .file("backend/node_modules/express/index.js", &"b".repeat(500))
    .file("shared/package.json", "")
    .file("shared/node_modules/lodash/index.js", &"c".repeat(300))
    .exists(&[
      "frontend/package.json",
      "backend/package.json",
      "shared/package.json",
    ])
    .expected_stdout(indoc! {
      "
      [ROOT]/backend Node project (0 seconds ago)
        └─ node_modules (500 bytes)
      [ROOT]/frontend Node project (0 seconds ago)
        └─ node_modules (1000 bytes)
      [ROOT]/shared Node project (0 seconds ago)
        └─ node_modules (300 bytes)
      Projects cleaned: 3, Bytes deleted: 1.76 KiB
      "
    })
    .run()
}

#[test]
fn older_than_filters_recent_projects() -> Result {
  Test::new()?
    .argument("--older-than")
    .argument("7d")
    .file("project/Cargo.toml", "")
    .file("project/target/debug/app", &"a".repeat(1000))
    .exists(&["project/Cargo.toml", "project/target/debug/app"])
    .expected_stdout(indoc! {
      "
      Projects cleaned: 0, Bytes deleted: 0 bytes
      "
    })
    .run()
}

#[test]
fn older_than_includes_old_projects() -> Result {
  Test::new()?
    .argument("--older-than")
    .argument("7d")
    .age(Duration::from_hours(720))
    .file("project/Cargo.toml", "")
    .file("project/target/debug/app", &"a".repeat(1000))
    .exists(&["project/Cargo.toml"])
    .expected_stdout(indoc! {
      "
      [ROOT]/project Cargo project (30 days ago)
        └─ target (1000 bytes)
      Projects cleaned: 1, Bytes deleted: 1000 bytes
      "
    })
    .run()
}

#[test]
fn older_than_with_ago_suffix() -> Result {
  Test::new()?
    .argument("--older-than")
    .argument("1w ago")
    .age(Duration::from_hours(336))
    .file("project/package.json", "")
    .file("project/node_modules/foo/index.js", &"a".repeat(500))
    .exists(&["project/package.json"])
    .expected_stdout(indoc! {
      "
      [ROOT]/project Node project (14 days ago)
        └─ node_modules (500 bytes)
      Projects cleaned: 1, Bytes deleted: 500 bytes
      "
    })
    .run()
}

#[test]
fn invalid_path_error() -> Result {
  Test::new()?
    .directory("nonexistent")
    .expected_status(1)
    .expected_stderr(
      "error: the path `[ROOT]/nonexistent` is not a valid directory\n",
    )
    .run()
}

#[test]
fn file_path_instead_of_directory_error() -> Result {
  Test::new()?
    .directory("file.txt")
    .file("file.txt", "content")
    .exists(&["file.txt"])
    .expected_status(1)
    .expected_stderr(
      "error: the path `[ROOT]/file.txt` is not a valid directory\n",
    )
    .run()
}
