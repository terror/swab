use super::*;

define_rule! {
  Cmake {
    id: "cmake",
    name: "CMake",
    actions: [
      Action::Remove("build"),
      Action::Remove("cmake-build-debug"),
      Action::Remove("cmake-build-release"),
    ],
    applies(context) {
      context.files.contains(&PathBuf::from("CMakeLists.txt"))
    }
  }
}
