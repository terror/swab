use super::*;

#[derive(Debug)]
pub(crate) enum Detection {
  All(Vec<Detection>),
  Any(Vec<Detection>),
  Not(Box<Detection>),
  Pattern(GlobMatcher),
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
      Self::Pattern(matcher) => context.contains(matcher),
    }
  }

  pub(crate) fn pattern(pattern: &str) -> Result<Self> {
    ensure!(
      !pattern.trim().is_empty(),
      "detection pattern cannot be empty"
    );

    Ok(Self::Pattern(
      GlobBuilder::new(pattern)
        .literal_separator(true)
        .build()
        .map_err(|error| {
          anyhow!("invalid detection pattern `{pattern}`: {error}")
        })?
        .compile_matcher(),
    ))
  }
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
      Self::Pattern(matcher) => write!(f, "{}", matcher.glob()),
    }
  }
}

impl TryFrom<ConfigDetection> for Detection {
  type Error = Error;

  fn try_from(value: ConfigDetection) -> Result<Self> {
    match value {
      ConfigDetection::Pattern(pattern)
      | ConfigDetection::PatternMap { pattern } => Self::pattern(&pattern),
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn matches() {
    #[track_caller]
    fn case(detection: &Detection, paths: &[&str], expected: bool) {
      assert_eq!(
        detection.matches(&Context {
          follow_symlinks: false,
          paths: paths.iter().map(PathBuf::from).collect(),
          root: PathBuf::new(),
        }),
        expected,
      );
    }

    let detection = Detection::try_from(ConfigDetection::All {
      all: vec![
        ConfigDetection::Pattern("*.foo".into()),
        ConfigDetection::Any {
          any: vec![
            ConfigDetection::PatternMap {
              pattern: "bar".into(),
            },
            ConfigDetection::Not {
              not: Box::new(ConfigDetection::Pattern("baz".into())),
            },
          ],
        },
      ],
    })
    .unwrap();

    assert_eq!(detection.to_string(), "(*.foo AND (bar OR NOT baz))");

    case(&detection, &[], false);
    case(&detection, &["foo.foo"], true);
    case(&detection, &["foo.foo", "baz"], false);
    case(&detection, &["bar", "foo.foo", "baz"], true);
    case(&detection, &["bar", "bar/foo.foo"], false);
  }
}
