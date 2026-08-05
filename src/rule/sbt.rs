use super::*;

define_rule! {
  Sbt {
    id: "sbt",
    name: "sbt (Scala)",
    detection: Detection::Pattern("build.sbt"),
    actions: [
      Action::Remove("**/target"),
    ],
  }
}
