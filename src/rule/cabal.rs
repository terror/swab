use super::*;

define_rule! {
  Cabal {
    id: "cabal",
    name: "Cabal (Haskell)",
    detection: Detection::Any(vec![
      Detection::Pattern("cabal.project"),
      Detection::Pattern("*.cabal"),
    ]),
    actions: [
      Action::Remove("dist-newstyle"),
    ],
  }
}
