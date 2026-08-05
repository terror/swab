use super::*;

define_rule! {
  Terraform {
    id: "terraform",
    name: "Terraform",
    detection: Detection::Any(
      Box::new(Detection::Pattern(".terraform.lock.hcl")),
      Box::new(Detection::Pattern(".terraform")),
    ),
    actions: [
      Action::Remove(".terraform"),
    ],
  }
}
