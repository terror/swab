use super::*;

define_rule! {
  Elixir {
    id: "elixir",
    name: "Elixir",
    actions: [
      Action::Remove("_build"),
      Action::Remove(".elixir-tools"),
      Action::Remove(".elixir_ls"),
      Action::Remove(".lexical"),
    ],
    applies(context) {
      context.files.contains(&PathBuf::from("mix.exs"))
    }
  }
}
