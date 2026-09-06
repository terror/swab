use super::*;

define_rule! {
  Sveltekit {
    id: "sveltekit",
    name: "SvelteKit",
    detection: Detection::All(vec![
      Detection::Pattern("package.json"),
      Detection::Pattern("svelte.config.*"),
    ]),
    actions: [
      Action::Remove(".svelte-kit"),
    ],
  }
}
