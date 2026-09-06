use super::*;

define_rule! {
  Elm {
    id: "elm",
    name: "Elm",
    detection: Detection::Pattern("elm.json"),
    actions: [
      Action::Remove("elm-stuff"),
    ],
  }
}
