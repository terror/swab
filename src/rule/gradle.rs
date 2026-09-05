use super::*;

define_rule! {
  Gradle {
    id: "gradle",
    name: "Gradle",
    detection: Detection::Any(vec![
      Detection::Pattern("build.gradle"),
      Detection::Pattern("build.gradle.kts"),
      Detection::Pattern("settings.gradle"),
      Detection::Pattern("settings.gradle.kts"),
    ]),
    actions: [
      Action::Remove("**/build"),
      Action::Remove(".gradle"),
    ],
  }
}
