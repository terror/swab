use super::*;

define_rule! {
  Nextjs {
    id: "nextjs",
    name: "Next.js",
    detection: Detection::All(vec![
      Detection::Pattern("package.json"),
      Detection::Any(vec![
        Detection::Pattern(".next"),
        Detection::Pattern("next.config.js"),
        Detection::Pattern("next.config.mjs"),
        Detection::Pattern("next.config.ts"),
      ]),
    ]),
    actions: [
      Action::Remove(".next"),
    ],
  }
}
