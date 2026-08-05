use super::*;

define_rule! {
  Nextjs {
    id: "nextjs",
    name: "Next.js",
    detection: Detection::All(
      Box::new(Detection::Pattern("package.json")),
      Box::new(Detection::Any(
        Box::new(Detection::Pattern(".next")),
        Box::new(Detection::Any(
          Box::new(Detection::Pattern("next.config.js")),
          Box::new(Detection::Any(
            Box::new(Detection::Pattern("next.config.mjs")),
            Box::new(Detection::Pattern("next.config.ts")),
          )),
        )),
      )),
    ),
    actions: [
      Action::Remove(".next"),
    ],
  }
}
