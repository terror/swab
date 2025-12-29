use super::*;

define_rule! {
  Turborepo {
    id: "turborepo",
    name: "Turborepo",
    actions: [
      Action::Remove(".turbo"),
    ],
    applies(context) {
      context.files.contains(&PathBuf::from("turbo.json"))
    }
  }
}
