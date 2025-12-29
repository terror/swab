use super::*;

pub(crate) struct Zig;

impl Rule for Zig {
  fn actions(&self) -> &[Action] {
    &[
      Action::Remove {
        pattern: "zig-cache",
        reason: "Zig cache",
      },
      Action::Remove {
        pattern: ".zig-cache",
        reason: "Zig cache",
      },
      Action::Remove {
        pattern: "zig-out",
        reason: "Zig build output",
      },
    ]
  }

  fn applies(&self, context: &Context) -> bool {
    context.files.contains(&PathBuf::from("build.zig"))
  }

  fn id(&self) -> &'static str {
    "zig"
  }

  fn name(&self) -> &'static str {
    "Zig"
  }
}
