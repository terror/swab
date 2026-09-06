use super::*;

define_rule! {
  Gleam {
    id: "gleam",
    name: "Gleam",
    detection: Detection::Pattern("gleam.toml"),
    actions: [
      Action::Remove("build"),
    ],
  }
}
