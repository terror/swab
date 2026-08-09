use super::*;

define_rule! {
  Jupyter {
    id: "jupyter",
    name: "Jupyter",
    detection: Detection::Pattern("*.ipynb"),
    actions: [
      Action::Remove("**/.ipynb_checkpoints"),
    ],
  }
}
