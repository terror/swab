use super::*;

define_rule! {
  Composer {
    id: "composer",
    name: "Composer (PHP)",
    actions: [
      Action::Remove("vendor"),
    ],
    applies(context) {
      context.files.contains(&PathBuf::from("composer.json"))
    }
  }
}
