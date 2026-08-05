use super::*;

define_rule! {
  Buck2 {
    id: "buck2",
    name: "Buck2",
    detection: Detection::Pattern(".buckconfig"),
    actions: [
      Action::Remove("buck-out"),
    ],
  }
}
