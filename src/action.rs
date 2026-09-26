use super::*;

#[derive(Debug)]
pub(crate) enum Action {
  Command(String),
  Remove(GlobMatcher),
}

impl Action {
  pub(crate) fn remove(pattern: &str) -> Result<Self> {
    ensure!(!pattern.trim().is_empty(), "remove action cannot be empty");

    Ok(Self::Remove(
      GlobBuilder::new(pattern)
        .literal_separator(true)
        .build()
        .map_err(|error| {
          anyhow!("invalid remove pattern `{pattern}`: {error}")
        })?
        .compile_matcher(),
    ))
  }
}

impl Display for Action {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    match self {
      Self::Command(cmd) => write!(f, "run `{cmd}`"),
      Self::Remove(matcher) => write!(f, "remove {}", matcher.glob()),
    }
  }
}

impl TryFrom<ConfigAction> for Action {
  type Error = Error;

  fn try_from(value: ConfigAction) -> Result<Self> {
    match value {
      ConfigAction::Remove { remove } => Self::remove(&remove),
      ConfigAction::Command { command } => {
        ensure!(!command.trim().is_empty(), "command action cannot be empty");
        Ok(Self::Command(command))
      }
    }
  }
}
