use super::*;

define_rule! {
  Dotnet {
    id: "dotnet",
    name: ".NET",
    detection: Detection::All(
      Box::new(Detection::Any(
        Box::new(Detection::Pattern("*.csproj")),
        Box::new(Detection::Any(
          Box::new(Detection::Pattern("*.fsproj")),
          Box::new(Detection::Pattern("*.vbproj")),
        )),
      )),
      Box::new(Detection::All(
        Box::new(Detection::Not(Box::new(Detection::Pattern("Assembly-CSharp.csproj")))),
        Box::new(Detection::Not(Box::new(Detection::Pattern("project.godot")))),
      )),
    ),
    actions: [
      Action::Remove("**/bin"),
      Action::Remove("**/obj"),
    ],
  }
}
