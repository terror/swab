use super::*;

define_rule! {
  Vcpkg {
    id: "vcpkg",
    name: "vcpkg",
    detection: Detection::Pattern("vcpkg.json"),
    actions: [
      Action::Remove("vcpkg_installed"),
    ],
  }
}
