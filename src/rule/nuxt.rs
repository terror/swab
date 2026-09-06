use super::*;

define_rule! {
  Nuxt {
    id: "nuxt",
    name: "Nuxt",
    detection: Detection::All(vec![
      Detection::Pattern("package.json"),
      Detection::Pattern("nuxt.config.*"),
    ]),
    actions: [
      Action::Remove(".nuxt"),
      Action::Remove(".output"),
    ],
  }
}
