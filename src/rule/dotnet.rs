use super::*;

define_rule! {
  Dotnet {
    id: "dotnet",
    name: ".NET",
    detection: Detection::All(vec![
      Detection::Any(vec![
        Detection::Pattern("*.csproj"),
        Detection::Pattern("*.fsproj"),
        Detection::Pattern("*.vbproj"),
      ]),
      Detection::Not(Box::new(Detection::Pattern("Assembly-CSharp.csproj"))),
      Detection::Not(Box::new(Detection::Pattern("project.godot"))),
    ]),
    actions: [
      Action::Remove("**/bin"),
      Action::Remove("**/obj"),
    ],
  }
}
