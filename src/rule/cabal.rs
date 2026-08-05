use super::*;

define_rule! {
  Cabal {
    id: "cabal",
    name: "Cabal (Haskell)",
    detection: Detection::Any(
      Box::new(Detection::Pattern("cabal.project")),
      Box::new(Detection::Pattern("*.cabal")),
    ),
    actions: [
      Action::Remove("dist-newstyle"),
    ],
  }
}
