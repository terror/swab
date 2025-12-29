use super::*;

define_rule! {
  Cabal {
    id: "cabal",
    name: "Cabal (Haskell)",
    actions: [
      Action::Remove("dist-newstyle"),
    ],
    applies(context) {
      context.files.contains(&PathBuf::from("cabal.project"))
    }
  }
}
