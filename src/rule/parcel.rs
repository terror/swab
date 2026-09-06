use super::*;

define_rule! {
  Parcel {
    id: "parcel",
    name: "Parcel",
    detection: Detection::All(vec![
      Detection::Pattern("package.json"),
      Detection::Pattern(".parcel-cache"),
    ]),
    actions: [
      Action::Remove(".parcel-cache"),
    ],
  }
}
