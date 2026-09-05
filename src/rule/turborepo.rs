use super::*;

define_rule! {
  Turborepo {
    id: "turborepo",
    name: "Turborepo",
    detection: Detection::Any(vec![
      Detection::Pattern("turbo.json"),
      Detection::Pattern("turbo.jsonc"),
    ]),
    actions: [
      Action::Remove(".turbo"),
    ],
  }
}
