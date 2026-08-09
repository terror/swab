use super::*;

define_rule! {
  Unreal {
    id: "unreal",
    name: "Unreal Engine",
    detection: Detection::Pattern("*.uproject"),
    actions: [
      Action::Remove("Binaries"),
      Action::Remove("Build"),
      Action::Remove("Saved"),
      Action::Remove("DerivedDataCache"),
      Action::Remove("Intermediate"),
    ],
  }
}
