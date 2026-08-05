use super::*;

define_rule! {
  Nx {
    id: "nx",
    name: "Nx",
    detection: Detection::Pattern("nx.json"),
    actions: [
      Action::Remove(".nx/cache"),
      Action::Remove(".nx/workspace-data"),
    ],
  }
}
