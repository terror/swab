use super::*;

define_rule! {
  Godot {
    id: "godot",
    name: "Godot 4",
    actions: [
      Action::Remove(".godot"),
    ],
    applies(context) {
      context.files.contains(&PathBuf::from("project.godot"))
    }
  }
}
