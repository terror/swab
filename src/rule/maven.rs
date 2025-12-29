use super::*;

define_rule! {
  Maven {
    id: "maven",
    name: "Maven",
    actions: [
      Action::Remove("target"),
    ],
    applies(context) {
      context.files.contains(&PathBuf::from("pom.xml"))
    }
  }
}
