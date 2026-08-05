use super::*;

define_rule! {
  Node {
    id: "node",
    name: "Node",
    detection: Detection::Pattern("package.json"),
    actions: [
      Action::Remove("**/node_modules"),
      Action::Remove(".angular/cache"),
    ],
  }
}
