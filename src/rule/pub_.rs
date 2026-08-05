use super::*;

define_rule! {
  Pub {
    id: "pub",
    name: "Pub (Dart/Flutter)",
    detection: Detection::Pattern("pubspec.yaml"),
    actions: [
      Action::Remove("build"),
      Action::Remove(".dart_tool"),
      Action::Remove(".android"),
      Action::Remove("ios/Flutter/ephemeral"),
      Action::Remove(".ios"),
      Action::Remove("ios/Flutter/Generated.xcconfig"),
      Action::Remove("ios/Flutter/flutter_export_environment.sh"),
      Action::Remove("ios/Flutter/App.framework"),
      Action::Remove("ios/Flutter/Flutter.framework"),
      Action::Remove("ios/Flutter/Flutter.podspec"),
      Action::Remove("linux/flutter/ephemeral"),
      Action::Remove("macos/Flutter/ephemeral"),
      Action::Remove("windows/flutter/ephemeral"),
      Action::Remove(".flutter-plugins-dependencies"),
    ],
  }
}
