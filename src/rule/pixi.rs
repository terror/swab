use super::*;

define_rule! {
  Pixi {
    id: "pixi",
    name: "Pixi",
    detection: Detection::Any(vec![
      Detection::Pattern("pixi.toml"),
      Detection::All(vec![
        Detection::Pattern("pyproject.toml"),
        Detection::Pattern(".pixi"),
      ]),
    ]),
    actions: [
      Action::Remove(".pixi"),
    ],
  }
}
