use super::*;

define_rule! {
  Elixir {
    id: "elixir",
    name: "Elixir",
    detection: Detection::Pattern("mix.exs"),
    actions: [
      Action::Remove("_build"),
      Action::Remove(".elixir-tools"),
      Action::Remove(".elixir_ls"),
      Action::Remove(".lexical"),
      Action::Remove("deps"),
    ],
  }
}
