use super::*;

pub(crate) struct Cargo;

impl Rule for Cargo {
  fn id(&self) -> &'static str {
    "cargo"
  }

  fn name(&self) -> &'static str {
    "Cargo"
  }

  fn applies(&self, context: &Context) -> bool {
    context.files.contains(&PathBuf::from("Cargo.toml"))
  }

  fn actions(&self) -> &[Action] {
    &[Action {
      pattern: "target",
      reason: "Cargo build artifacts",
    }]
  }
}
