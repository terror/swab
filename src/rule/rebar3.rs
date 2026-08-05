use super::*;

define_rule! {
  Rebar3 {
    id: "rebar3",
    name: "Rebar3 (Erlang)",
    detection: Detection::Pattern("rebar.config"),
    actions: [
      Action::Remove("_build"),
    ],
  }
}
