use super::*;

define_rule! {
  Terragrunt {
    id: "terragrunt",
    name: "Terragrunt",
    detection: Detection::Pattern("terragrunt.hcl"),
    actions: [
      Action::Remove("**/.terragrunt-cache"),
    ],
  }
}
