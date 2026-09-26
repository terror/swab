use super::*;

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct DefaultRulesConfig {
  pub(crate) disabled: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct RuleConfig {
  #[serde(default)]
  pub(crate) actions: Vec<ConfigAction>,
  pub(crate) detection: ConfigDetection,
  pub(crate) id: String,
  pub(crate) name: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub(crate) enum ConfigDetection {
  All { all: Vec<ConfigDetection> },
  Any { any: Vec<ConfigDetection> },
  Not { not: Box<ConfigDetection> },
  Pattern(String),
  PatternMap { pattern: String },
}

impl Display for ConfigDetection {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    match self {
      Self::All { all } => {
        write!(
          f,
          "({})",
          all
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(" AND ")
        )
      }
      Self::Any { any } => {
        write!(
          f,
          "({})",
          any
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(" OR ")
        )
      }
      Self::Not { not } => write!(f, "NOT {not}"),
      Self::Pattern(pattern) | Self::PatternMap { pattern } => {
        write!(f, "{pattern}")
      }
    }
  }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub(crate) enum ConfigAction {
  Command { command: String },
  Remove { remove: String },
}

impl Display for ConfigAction {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    match self {
      Self::Command { command } => write!(f, "run `{command}`"),
      Self::Remove { remove } => write!(f, "remove {remove}"),
    }
  }
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct Config {
  #[serde(alias = "default")]
  pub(crate) default_rules: DefaultRulesConfig,
  pub(crate) rules: Vec<RuleConfig>,
}

impl TryFrom<Config> for Vec<Rule> {
  type Error = Error;

  fn try_from(config: Config) -> Result<Self> {
    let mut custom_rules = config
      .rules
      .into_iter()
      .map(Rule::try_from)
      .try_fold(HashMap::new(), |mut acc, item| {
        let rule = item?;
        let id = rule.id.clone();

        ensure!(
          acc.insert(id.clone(), rule).is_none(),
          "duplicate rule id `{id}` in config"
        );

        Ok(acc)
      })?;

    let disabled = config
      .default_rules
      .disabled
      .into_iter()
      .collect::<HashSet<String>>();

    let mut rules = Rule::builtins()?
      .into_iter()
      .filter_map(|default| {
        if let Some(custom) = custom_rules.remove(&default.id) {
          return Some(custom);
        }

        if disabled.contains(&default.id) {
          return None;
        }

        Some(default)
      })
      .collect::<Vec<_>>();

    rules.extend(custom_rules.into_values());

    Ok(rules)
  }
}

impl Config {
  pub(crate) fn load() -> Result<Self> {
    Ok(confy::load("swab", "config")?)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn rule(id: &str) -> RuleConfig {
    RuleConfig {
      actions: vec![ConfigAction::Remove {
        remove: "foo".into(),
      }],
      detection: ConfigDetection::Pattern("bar".into()),
      id: id.into(),
      name: None,
    }
  }

  #[test]
  fn invalid_rules() {
    #[track_caller]
    fn case(rules: Vec<RuleConfig>, expected: &str) {
      assert_eq!(
        Vec::<Rule>::try_from(Config {
          rules,
          ..Config::default()
        })
        .unwrap_err()
        .to_string(),
        expected,
      );
    }

    case(vec![rule(" ")], "rule id cannot be empty");

    case(
      vec![RuleConfig {
        actions: Vec::new(),
        ..rule("foo")
      }],
      "rule actions cannot be empty",
    );

    case(
      vec![rule("foo"), rule("foo")],
      "duplicate rule id `foo` in config",
    );

    case(
      vec![RuleConfig {
        detection: ConfigDetection::Pattern(" ".into()),
        ..rule("foo")
      }],
      "detection pattern cannot be empty",
    );

    case(
      vec![RuleConfig {
        detection: ConfigDetection::All { all: Vec::new() },
        ..rule("foo")
      }],
      "`all` detection must contain at least one entry",
    );

    case(
      vec![RuleConfig {
        detection: ConfigDetection::Any { any: Vec::new() },
        ..rule("foo")
      }],
      "`any` detection must contain at least one entry",
    );

    case(
      vec![RuleConfig {
        detection: ConfigDetection::Not {
          not: Box::new(ConfigDetection::PatternMap {
            pattern: "[".into(),
          }),
        },
        ..rule("foo")
      }],
      concat!(
        "invalid detection pattern `[`: error parsing glob '[': ",
        "unclosed character class; missing ']'",
      ),
    );

    case(
      vec![RuleConfig {
        actions: vec![ConfigAction::Command {
          command: " ".into(),
        }],
        ..rule("foo")
      }],
      "command action cannot be empty",
    );

    case(
      vec![RuleConfig {
        actions: vec![ConfigAction::Remove { remove: " ".into() }],
        ..rule("foo")
      }],
      "remove action cannot be empty",
    );

    case(
      vec![RuleConfig {
        actions: vec![ConfigAction::Remove { remove: "[".into() }],
        ..rule("foo")
      }],
      concat!(
        "invalid remove pattern `[`: error parsing glob '[': ",
        "unclosed character class; missing ']'",
      ),
    );
  }

  #[test]
  fn rules() {
    let mut rules = Vec::<Rule>::try_from(Config {
      default_rules: DefaultRulesConfig {
        disabled: Rule::builtins()
          .unwrap()
          .into_iter()
          .map(|rule| rule.id)
          .filter(|id| id != "cargo")
          .collect(),
      },
      rules: vec![
        rule("foo"),
        RuleConfig {
          actions: vec![ConfigAction::Command {
            command: "foo".into(),
          }],
          name: Some("baz".into()),
          ..rule("node")
        },
      ],
    })
    .unwrap();

    rules.sort_by(|a, b| a.id.cmp(&b.id));

    assert_eq!(
      rules
        .iter()
        .map(|rule| (
          rule.id.as_str(),
          rule.name.as_str(),
          rule.detection.to_string(),
          rule
            .actions
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
        ))
        .collect::<Vec<_>>(),
      vec![
        (
          "cargo",
          "Cargo",
          "Cargo.toml".into(),
          vec!["remove **/target".into()]
        ),
        ("foo", "foo", "bar".into(), vec!["remove foo".into()]),
        ("node", "baz", "bar".into(), vec!["run `foo`".into()]),
      ],
    );
  }
}
