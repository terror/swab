use super::*;

define_rule! {
  Terraform {
    id: "terraform",
    name: "Terraform",
    detection: Detection::Any(vec![
      Detection::Pattern(".terraform.lock.hcl"),
      Detection::Pattern(".terraform"),
    ]),
    actions: [
      Action::Remove(".terraform"),
    ],
  }
}
