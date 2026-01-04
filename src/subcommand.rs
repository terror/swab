use super::*;

mod rules;

#[derive(Debug, Parser)]
pub(crate) enum Subcommand {
  #[command(about = "List all available rules")]
  Rules,
}

impl Subcommand {
  pub(crate) fn run(self) -> Result {
    match self {
      Self::Rules => rules::run(),
    }
  }
}
