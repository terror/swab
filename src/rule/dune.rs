use super::*;

define_rule! {
  Dune {
    id: "dune",
    name: "Dune (OCaml)",
    detection: Detection::Any(vec![
      Detection::Pattern("dune-project"),
      Detection::Pattern("dune-workspace"),
    ]),
    actions: [
      Action::Remove("_build"),
    ],
  }
}
