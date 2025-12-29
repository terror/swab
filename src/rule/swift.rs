use super::*;

define_rule! {
  Swift {
    id: "swift",
    name: "Swift",
    actions: [
      Action::Remove(".build"),
      Action::Remove(".swiftpm"),
    ],
    applies(context) {
      context.files.contains(&PathBuf::from("Package.swift"))
    }
  }
}
