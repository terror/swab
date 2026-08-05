use super::*;

define_rule! {
  Maven {
    id: "maven",
    name: "Maven",
    detection: Detection::Pattern("pom.xml"),
    actions: [
      Action::Remove("**/target"),
    ],
  }
}
