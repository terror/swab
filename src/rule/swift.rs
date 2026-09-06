use super::*;

define_rule! {
  Swift {
    id: "swift",
    name: "Swift",
    detection: Detection::Pattern("Package.swift"),
    actions: [
      Action::Remove(".build"),
    ],
  }
}
