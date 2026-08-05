use super::*;

define_rule! {
  Turborepo {
    id: "turborepo",
    name: "Turborepo",
    detection: Detection::Any(
      Box::new(Detection::Pattern("turbo.json")),
      Box::new(Detection::Pattern("turbo.jsonc")),
    ),
    actions: [
      Action::Remove(".turbo"),
    ],
  }
}
