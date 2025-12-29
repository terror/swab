use super::*;

define_rule! {
  Gradle {
    id: "gradle",
    name: "Gradle",
    actions: [
      Action::Remove("build"),
      Action::Remove(".gradle"),
    ],
    applies(context) {
      context.files.contains(&PathBuf::from("build.gradle"))
        || context.files.contains(&PathBuf::from("build.gradle.kts"))
    }
  }
}
