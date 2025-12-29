use super::*;

#[derive(Clone, Debug)]
pub(crate) enum Detection {
  All(Box<Detection>, Box<Detection>),
  Any(Box<Detection>, Box<Detection>),
  Not(Box<Detection>),
  Pattern(&'static str),
}

impl Detection {
  pub(crate) fn matches(&self, context: &Context) -> bool {
    match self {
      Detection::All(left, right) => {
        left.matches(context) && right.matches(context)
      }
      Detection::Any(left, right) => {
        left.matches(context) || right.matches(context)
      }
      Detection::Not(inner) => !inner.matches(context),
      Detection::Pattern(pattern) => context.contains(pattern),
    }
  }
}
