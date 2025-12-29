use super::*;

pub(crate) struct Zig;

impl Rule for Zig {
  fn id(&self) -> &'static str {
    "zig"
  }

  fn name(&self) -> &'static str {
    "Zig"
  }

  fn applies(&self, context: &Context) -> bool {
    context.files.contains(&PathBuf::from("build.zig"))
  }

  fn actions(&self) -> &[Action] {
    &[
      Action {
        pattern: "zig-cache",
        reason: "Zig cache",
      },
      Action {
        pattern: ".zig-cache",
        reason: "Zig cache",
      },
      Action {
        pattern: "zig-out",
        reason: "Zig build output",
      },
    ]
  }
}
