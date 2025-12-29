use super::*;

define_rule! {
  Pub {
    id: "pub",
    name: "Pub (Dart/Flutter)",
    actions: [
      Action::Remove("build"),
      Action::Remove(".dart_tool"),
      Action::Remove("linux/flutter/ephemeral"),
      Action::Remove("windows/flutter/ephemeral"),
    ],
    applies(context) {
      context.files.contains(&PathBuf::from("pubspec.yaml"))
    }
  }
}
