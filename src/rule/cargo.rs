use super::*;

pub(crate) struct Cargo;

impl Rule for Cargo {
  fn actions(&self) -> &[Action] {
    &[Action::Remove {
      pattern: "**/target",
      reason: "Cargo build artifacts",
    }]
  }

  fn applies(&self, context: &Context) -> bool {
    context.files.contains(&PathBuf::from("Cargo.toml"))
  }

  fn id(&self) -> &'static str {
    "cargo"
  }

  fn name(&self) -> &'static str {
    "Cargo"
  }
}
