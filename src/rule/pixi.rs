use super::*;

define_rule! {
  Pixi {
    id: "pixi",
    name: "Pixi",
    detection: Detection::Any(
      Box::new(Detection::Pattern("pixi.toml")),
      Box::new(Detection::All(
        Box::new(Detection::Pattern("pyproject.toml")),
        Box::new(Detection::Pattern(".pixi")),
      )),
    ),
    actions: [
      Action::Remove(".pixi"),
    ],
  }
}
