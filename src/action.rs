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
