use super::*;

define_rule! {
  Sbt {
    id: "sbt",
    name: "SBT (Scala)",
    actions: [
      Action::Remove("target"),
      Action::Remove("project/target"),
    ],
    applies(context) {
      context.files.contains(&PathBuf::from("build.sbt"))
    }
  }
}
