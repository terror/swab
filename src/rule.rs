use super::*;

#[derive(Debug)]
pub(crate) struct Rule {
  pub(crate) actions: Vec<Action>,
  pub(crate) detection: Detection,
  pub(crate) id: String,
  pub(crate) name: String,
}

impl Rule {
  pub(crate) fn builtins() -> Result<Vec<Self>> {
    Ok(vec![
      Self {
        actions: vec![Action::remove("buck-out")?],
        detection: Detection::pattern(".buckconfig")?,
        id: "buck2".into(),
        name: "Buck2".into(),
      },
      Self {
        actions: vec![Action::remove("dist-newstyle")?],
        detection: Detection::Any(vec![
          Detection::pattern("cabal.project")?,
          Detection::pattern("*.cabal")?,
        ]),
        id: "cabal".into(),
        name: "Cabal (Haskell)".into(),
      },
      Self {
        actions: vec![Action::remove("**/target")?],
        detection: Detection::pattern("Cargo.toml")?,
        id: "cargo".into(),
        name: "Cargo".into(),
      },
      Self {
        actions: vec![
          Action::remove("build")?,
          Action::remove("cmake-build-debug")?,
          Action::remove("cmake-build-release")?,
        ],
        detection: Detection::pattern("CMakeLists.txt")?,
        id: "cmake".into(),
        name: "CMake".into(),
      },
      Self {
        actions: vec![Action::remove("vendor")?],
        detection: Detection::pattern("composer.json")?,
        id: "composer".into(),
        name: "Composer (PHP)".into(),
      },
      Self {
        actions: vec![Action::remove("**/bin")?, Action::remove("**/obj")?],
        detection: Detection::All(vec![
          Detection::Any(vec![
            Detection::pattern("*.csproj")?,
            Detection::pattern("*.fsproj")?,
            Detection::pattern("*.vbproj")?,
          ]),
          Detection::Not(Box::new(Detection::pattern(
            "Assembly-CSharp.csproj",
          )?)),
          Detection::Not(Box::new(Detection::pattern("project.godot")?)),
        ]),
        id: "dotnet".into(),
        name: ".NET".into(),
      },
      Self {
        actions: vec![Action::remove("_build")?],
        detection: Detection::Any(vec![
          Detection::pattern("dune-project")?,
          Detection::pattern("dune-workspace")?,
        ]),
        id: "dune".into(),
        name: "Dune (OCaml)".into(),
      },
      Self {
        actions: vec![
          Action::remove("_build")?,
          Action::remove(".elixir-tools")?,
          Action::remove(".elixir_ls")?,
          Action::remove(".lexical")?,
          Action::remove("deps")?,
        ],
        detection: Detection::pattern("mix.exs")?,
        id: "elixir".into(),
        name: "Elixir".into(),
      },
      Self {
        actions: vec![Action::remove("elm-stuff")?],
        detection: Detection::pattern("elm.json")?,
        id: "elm".into(),
        name: "Elm".into(),
      },
      Self {
        actions: vec![Action::remove("build")?],
        detection: Detection::pattern("gleam.toml")?,
        id: "gleam".into(),
        name: "Gleam".into(),
      },
      Self {
        actions: vec![Action::remove(".godot")?],
        detection: Detection::pattern("project.godot")?,
        id: "godot".into(),
        name: "Godot 4".into(),
      },
      Self {
        actions: vec![Action::remove("**/build")?, Action::remove(".gradle")?],
        detection: Detection::Any(vec![
          Detection::pattern("build.gradle")?,
          Detection::pattern("build.gradle.kts")?,
          Detection::pattern("settings.gradle")?,
          Detection::pattern("settings.gradle.kts")?,
        ]),
        id: "gradle".into(),
        name: "Gradle".into(),
      },
      Self {
        actions: vec![Action::remove("**/.ipynb_checkpoints")?],
        detection: Detection::pattern("*.ipynb")?,
        id: "jupyter".into(),
        name: "Jupyter".into(),
      },
      Self {
        actions: vec![Action::remove("**/target")?],
        detection: Detection::pattern("pom.xml")?,
        id: "maven".into(),
        name: "Maven".into(),
      },
      Self {
        actions: vec![Action::remove(".next")?],
        detection: Detection::All(vec![
          Detection::pattern("package.json")?,
          Detection::Any(vec![
            Detection::pattern(".next")?,
            Detection::pattern("next.config.js")?,
            Detection::pattern("next.config.mjs")?,
            Detection::pattern("next.config.ts")?,
          ]),
        ]),
        id: "nextjs".into(),
        name: "Next.js".into(),
      },
      Self {
        actions: vec![
          Action::remove("**/node_modules")?,
          Action::remove(".angular/cache")?,
        ],
        detection: Detection::pattern("package.json")?,
        id: "node".into(),
        name: "Node".into(),
      },
      Self {
        actions: vec![Action::remove(".nuxt")?, Action::remove(".output")?],
        detection: Detection::All(vec![
          Detection::pattern("package.json")?,
          Detection::pattern("nuxt.config.*")?,
        ]),
        id: "nuxt".into(),
        name: "Nuxt".into(),
      },
      Self {
        actions: vec![
          Action::remove(".nx/cache")?,
          Action::remove(".nx/workspace-data")?,
        ],
        detection: Detection::pattern("nx.json")?,
        id: "nx".into(),
        name: "Nx".into(),
      },
      Self {
        actions: vec![Action::remove(".parcel-cache")?],
        detection: Detection::All(vec![
          Detection::pattern("package.json")?,
          Detection::pattern(".parcel-cache")?,
        ]),
        id: "parcel".into(),
        name: "Parcel".into(),
      },
      Self {
        actions: vec![Action::remove(".pixi/envs")?],
        detection: Detection::Any(vec![
          Detection::pattern("pixi.toml")?,
          Detection::All(vec![
            Detection::pattern("pyproject.toml")?,
            Detection::pattern(".pixi")?,
          ]),
        ]),
        id: "pixi".into(),
        name: "Pixi".into(),
      },
      Self {
        actions: vec![
          Action::remove("build")?,
          Action::remove(".dart_tool")?,
          Action::remove(".android")?,
          Action::remove("ios/Flutter/ephemeral")?,
          Action::remove(".ios")?,
          Action::remove("ios/Flutter/Generated.xcconfig")?,
          Action::remove("ios/Flutter/flutter_export_environment.sh")?,
          Action::remove("ios/Flutter/App.framework")?,
          Action::remove("ios/Flutter/Flutter.framework")?,
          Action::remove("ios/Flutter/Flutter.podspec")?,
          Action::remove("linux/flutter/ephemeral")?,
          Action::remove("macos/Flutter/ephemeral")?,
          Action::remove("windows/flutter/ephemeral")?,
          Action::remove(".flutter-plugins-dependencies")?,
        ],
        detection: Detection::pattern("pubspec.yaml")?,
        id: "pub".into(),
        name: "Pub (Dart/Flutter)".into(),
      },
      Self {
        actions: vec![
          Action::remove(".mypy_cache")?,
          Action::remove(".nox")?,
          Action::remove(".pytest_cache")?,
          Action::remove(".ruff_cache")?,
          Action::remove(".tox")?,
          Action::remove(".venv")?,
          Action::remove("**/__pycache__")?,
          Action::remove("__pypackages__")?,
        ],
        detection: Detection::Any(vec![
          Detection::pattern("pyproject.toml")?,
          Detection::pattern("setup.py")?,
          Detection::pattern("setup.cfg")?,
        ]),
        id: "python".into(),
        name: "Python".into(),
      },
      Self {
        actions: vec![Action::remove("_build")?],
        detection: Detection::pattern("rebar.config")?,
        id: "rebar3".into(),
        name: "Rebar3 (Erlang)".into(),
      },
      Self {
        actions: vec![Action::remove("**/target")?],
        detection: Detection::pattern("build.sbt")?,
        id: "sbt".into(),
        name: "sbt (Scala)".into(),
      },
      Self {
        actions: vec![Action::remove(".stack-work")?],
        detection: Detection::pattern("stack.yaml")?,
        id: "stack".into(),
        name: "Stack (Haskell)".into(),
      },
      Self {
        actions: vec![Action::remove(".svelte-kit")?],
        detection: Detection::All(vec![
          Detection::pattern("package.json")?,
          Detection::pattern("svelte.config.*")?,
        ]),
        id: "sveltekit".into(),
        name: "SvelteKit".into(),
      },
      Self {
        actions: vec![Action::remove(".build")?],
        detection: Detection::pattern("Package.swift")?,
        id: "swift".into(),
        name: "Swift".into(),
      },
      Self {
        actions: vec![Action::remove(".terraform")?],
        detection: Detection::Any(vec![
          Detection::pattern(".terraform.lock.hcl")?,
          Detection::pattern(".terraform")?,
        ]),
        id: "terraform".into(),
        name: "Terraform".into(),
      },
      Self {
        actions: vec![Action::remove("**/.terragrunt-cache")?],
        detection: Detection::pattern("terragrunt.hcl")?,
        id: "terragrunt".into(),
        name: "Terragrunt".into(),
      },
      Self {
        actions: vec![Action::remove(".turbo/cache")?],
        detection: Detection::Any(vec![
          Detection::pattern("turbo.json")?,
          Detection::pattern("turbo.jsonc")?,
        ]),
        id: "turborepo".into(),
        name: "Turborepo".into(),
      },
      Self {
        actions: vec![
          Action::remove("Library")?,
          Action::remove("Temp")?,
          Action::remove("Obj")?,
          Action::remove("Logs")?,
          Action::remove("MemoryCaptures")?,
          Action::remove("Build")?,
          Action::remove("Builds")?,
        ],
        detection: Detection::pattern("Assembly-CSharp.csproj")?,
        id: "unity".into(),
        name: "Unity".into(),
      },
      Self {
        actions: vec![
          Action::remove("Binaries")?,
          Action::remove("DerivedDataCache")?,
          Action::remove("Intermediate")?,
          Action::remove("Saved/Cooked")?,
          Action::remove("Saved/Logs")?,
          Action::remove("Saved/StagedBuilds")?,
        ],
        detection: Detection::pattern("*.uproject")?,
        id: "unreal".into(),
        name: "Unreal Engine".into(),
      },
      Self {
        actions: vec![Action::remove("vcpkg_installed")?],
        detection: Detection::pattern("vcpkg.json")?,
        id: "vcpkg".into(),
        name: "vcpkg".into(),
      },
      Self {
        actions: vec![
          Action::remove("zig-cache")?,
          Action::remove(".zig-cache")?,
          Action::remove("zig-out")?,
        ],
        detection: Detection::pattern("build.zig")?,
        id: "zig".into(),
        name: "Zig".into(),
      },
    ])
  }
}

impl TryFrom<RuleConfig> for Rule {
  type Error = Error;

  fn try_from(rule: RuleConfig) -> Result<Self> {
    ensure!(!rule.id.trim().is_empty(), "rule id cannot be empty");

    ensure!(!rule.actions.is_empty(), "rule actions cannot be empty");

    let actions = rule
      .actions
      .into_iter()
      .map(ConfigAction::try_into)
      .collect::<Result<Vec<_>>>()?;

    Ok(Self {
      actions,
      detection: rule.detection.try_into()?,
      id: rule.id.clone(),
      name: rule.name.unwrap_or(rule.id),
    })
  }
}
