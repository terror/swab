use super::*;

define_rule! {
  Unreal {
    id: "unreal",
    name: "Unreal Engine",
    detection: Detection::Pattern("*.uproject"),
    actions: [
      Action::Remove("Binaries"),
      Action::Remove("DerivedDataCache"),
      Action::Remove("Intermediate"),
      Action::Remove("Saved/Cooked"),
      Action::Remove("Saved/Logs"),
      Action::Remove("Saved/StagedBuilds"),
    ],
  }
}
