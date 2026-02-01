use super::*;

#[derive(Clone, Debug)]
pub(crate) enum Detection {
  All(Box<Detection>, Box<Detection>),
  Any(Box<Detection>, Box<Detection>),
  Not(Box<Detection>),
  Pattern(&'static str),
}

impl Display for Detection {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    match self {
      Self::All(left, right) => write!(f, "({left} AND {right})"),
      Self::Any(left, right) => write!(f, "({left} OR {right})"),
      Self::Not(inner) => write!(f, "NOT {inner}"),
      Self::Pattern(pattern) => write!(f, "{pattern}"),
    }
  }
}

impl TryFrom<ConfigDetection> for Detection {
  type Error = Error;

  fn try_from(value: ConfigDetection) -> Result<Self> {
    match value {
      ConfigDetection::Pattern(pattern)
      | ConfigDetection::PatternMap { pattern } => {
        ensure!(
          !pattern.trim().is_empty(),
          "detection pattern cannot be empty"
        );

        Glob::new(&pattern).map_err(|error| {
          anyhow!("invalid detection pattern `{pattern}`: {error}")
        })?;

        Ok(Detection::Pattern(Box::leak(pattern.into_boxed_str())))
      }
      ConfigDetection::Any { any } => {
        ConfigDetection::fold(any, Detection::Any, "any")
      }
      ConfigDetection::All { all } => {
        ConfigDetection::fold(all, Detection::All, "all")
      }
      ConfigDetection::Not { not } => {
        Ok(Detection::Not(Box::new((*not).try_into()?)))
      }
    }
  }
}

impl Detection {
  pub(crate) fn matches(&self, context: &Context) -> bool {
    match self {
      Self::All(left, right) => left.matches(context) && right.matches(context),
      Self::Any(left, right) => left.matches(context) || right.matches(context),
      Self::Not(inner) => !inner.matches(context),
      Self::Pattern(pattern) => context.contains(pattern),
    }
  }
}
