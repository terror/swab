use super::*;

define_rule! {
  Pixi {
    id: "pixi",
    name: "Pixi",
    actions: [
      Action::Remove(".pixi"),
    ],
    applies(context) {
      context.files.contains(&PathBuf::from("pixi.toml"))
    }
  }
}
