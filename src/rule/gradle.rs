use super::*;

define_rule! {
  Gradle {
    id: "gradle",
    name: "Gradle",
    detection: Detection::Any(
      Box::new(Detection::Any(
        Box::new(Detection::Pattern("build.gradle")),
        Box::new(Detection::Pattern("build.gradle.kts")),
      )),
      Box::new(Detection::Any(
        Box::new(Detection::Pattern("settings.gradle")),
        Box::new(Detection::Pattern("settings.gradle.kts")),
      )),
    ),
    actions: [
      Action::Remove("**/build"),
      Action::Remove(".gradle"),
    ],
  }
}
