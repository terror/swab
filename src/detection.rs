use super::*;

#[derive(Clone, Debug)]
pub(crate) enum Detection {
  All(Vec<Detection>),
  Any(Vec<Detection>),
  Not(Box<Detection>),
  Pattern(&'static str),
}

impl Display for Detection {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    match self {
      Self::All(detections) => write!(
        f,
        "({})",
        detections
          .iter()
          .map(ToString::to_string)
          .collect::<Vec<_>>()
          .join(" AND ")
      ),
      Self::Any(detections) => write!(
        f,
        "({})",
        detections
          .iter()
          .map(ToString::to_string)
          .collect::<Vec<_>>()
          .join(" OR ")
      ),
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

        GlobBuilder::new(&pattern)
          .literal_separator(true)
          .build()
          .map_err(|error| {
            anyhow!("invalid detection pattern `{pattern}`: {error}")
          })?;

        Ok(Detection::Pattern(Box::leak(pattern.into_boxed_str())))
      }
      ConfigDetection::Any { any } => {
        ensure!(
          !any.is_empty(),
          "`any` detection must contain at least one entry"
        );

        Ok(Detection::Any(
          any
            .into_iter()
            .map(ConfigDetection::try_into)
            .collect::<Result<Vec<_>>>()?,
        ))
      }
      ConfigDetection::All { all } => {
        ensure!(
          !all.is_empty(),
          "`all` detection must contain at least one entry"
        );

        Ok(Detection::All(
          all
            .into_iter()
            .map(ConfigDetection::try_into)
            .collect::<Result<Vec<_>>>()?,
        ))
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
      Self::All(detections) => detections
        .iter()
        .all(|detection| detection.matches(context)),
      Self::Any(detections) => detections
        .iter()
        .any(|detection| detection.matches(context)),
      Self::Not(inner) => !inner.matches(context),
      Self::Pattern(pattern) => context.contains(pattern),
    }
  }
}
