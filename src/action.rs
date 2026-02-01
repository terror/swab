use super::*;

#[derive(Debug)]
pub(crate) enum Action {
  Command(&'static str),
  Remove(&'static str),
}

impl Display for Action {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    match self {
      Self::Command(cmd) => write!(f, "run `{cmd}`"),
      Self::Remove(pattern) => write!(f, "remove {pattern}"),
    }
  }
}

impl TryFrom<ConfigAction> for Action {
  type Error = Error;

  fn try_from(value: ConfigAction) -> Result<Self> {
    match value {
      ConfigAction::Remove { remove } => {
        ensure!(!remove.trim().is_empty(), "remove action cannot be empty");

        Glob::new(&remove).map_err(|error| {
          anyhow!("invalid remove pattern `{remove}`: {error}")
        })?;

        Ok(Action::Remove(Box::leak(remove.into_boxed_str())))
      }
      ConfigAction::Command { command } => {
        ensure!(!command.trim().is_empty(), "command action cannot be empty");
        Ok(Action::Command(Box::leak(command.into_boxed_str())))
      }
    }
  }
}
