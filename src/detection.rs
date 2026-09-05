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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn from_config() {
    #[track_caller]
    fn case(config: ConfigDetection, expected: &str) {
      assert_eq!(Detection::try_from(config).unwrap().to_string(), expected);
    }

    case(ConfigDetection::Pattern("foo".into()), "foo");
    case(
      ConfigDetection::PatternMap {
        pattern: "foo".into(),
      },
      "foo",
    );
    case(
      ConfigDetection::All {
        all: vec![ConfigDetection::Pattern("foo".into())],
      },
      "(foo)",
    );
    case(
      ConfigDetection::Any {
        any: vec![ConfigDetection::Pattern("foo".into())],
      },
      "(foo)",
    );
    case(
      ConfigDetection::All {
        all: ["foo", "bar", "baz"]
          .map(|pattern| ConfigDetection::Pattern(pattern.into()))
          .into(),
      },
      "(foo AND bar AND baz)",
    );
    case(
      ConfigDetection::Any {
        any: ["foo", "bar", "baz"]
          .map(|pattern| ConfigDetection::Pattern(pattern.into()))
          .into(),
      },
      "(foo OR bar OR baz)",
    );
    case(
      ConfigDetection::All {
        all: vec![
          ConfigDetection::Pattern("foo".into()),
          ConfigDetection::Not {
            not: Box::new(ConfigDetection::Any {
              any: vec![
                ConfigDetection::Pattern("bar".into()),
                ConfigDetection::Pattern("baz".into()),
              ],
            }),
          },
        ],
      },
      "(foo AND NOT (bar OR baz))",
    );
  }

  #[test]
  fn from_config_rejects_empty_lists() {
    #[track_caller]
    fn case(config: ConfigDetection, expected: &str) {
      assert_eq!(
        Detection::try_from(config).unwrap_err().to_string(),
        expected
      );
    }

    case(
      ConfigDetection::All { all: Vec::new() },
      "`all` detection must contain at least one entry",
    );
    case(
      ConfigDetection::Any { any: Vec::new() },
      "`any` detection must contain at least one entry",
    );
    case(
      ConfigDetection::All {
        all: vec![
          ConfigDetection::Pattern("foo".into()),
          ConfigDetection::Any { any: Vec::new() },
        ],
      },
      "`any` detection must contain at least one entry",
    );
    case(
      ConfigDetection::Any {
        any: vec![
          ConfigDetection::Pattern("foo".into()),
          ConfigDetection::All { all: Vec::new() },
        ],
      },
      "`all` detection must contain at least one entry",
    );
    case(
      ConfigDetection::Not {
        not: Box::new(ConfigDetection::All { all: Vec::new() }),
      },
      "`all` detection must contain at least one entry",
    );
  }

  #[test]
  fn matches() {
    #[track_caller]
    fn case(files: &[&str], all: bool, any: bool) {
      let context = Context {
        directories: HashSet::new(),
        files: files.iter().map(PathBuf::from).collect(),
        follow_symlinks: false,
        root: PathBuf::new(),
      };

      let detections = ["foo", "bar", "baz"].map(Detection::Pattern).to_vec();

      assert_eq!(Detection::All(detections.clone()).matches(&context), all);
      assert_eq!(Detection::Any(detections).matches(&context), any);
    }

    case(&["foo", "bar", "baz"], true, true);
    case(&["bar", "baz"], false, true);
    case(&["foo", "baz"], false, true);
    case(&["foo", "bar"], false, true);
    case(&["foo"], false, true);
    case(&["bar"], false, true);
    case(&["baz"], false, true);
    case(&[], false, false);
  }
}
