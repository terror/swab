use super::*;

define_rule! {
  Dune {
    id: "dune",
    name: "Dune (OCaml)",
    detection: Detection::Any(
      Box::new(Detection::Pattern("dune-project")),
      Box::new(Detection::Pattern("dune-workspace")),
    ),
    actions: [
      Action::Remove("_build"),
    ],
  }
}
