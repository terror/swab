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

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub(crate) enum ConfigAction {
  Command { command: String },
  Remove { remove: String },
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct Config {
  #[serde(alias = "default")]
  pub(crate) default_rules: DefaultRulesConfig,
  pub(crate) rules: Vec<RuleConfig>,
}

impl Config {
  pub(crate) fn load() -> Result<Self> {
    Ok(confy::load("swab", "config")?)
  }

  pub(crate) fn resolve(self) -> Result<Vec<ResolvedRule>> {
    let mut custom_rules = self
      .rules
      .into_iter()
      .map(Rule::try_from)
      .try_fold(BTreeMap::new(), |mut acc, item| {
        let rule = item?;

        let id = rule.id.clone();

        ensure!(
          acc.insert(id.clone(), rule).is_none(),
          "duplicate rule id `{id}` in config"
        );

        Ok(acc)
      })?;

    let disabled = self
      .default_rules
      .disabled
      .into_iter()
      .collect::<HashSet<String>>();

    let mut rules = Rule::builtins()?
      .into_iter()
      .map(|rule| {
        let (rule, status) = if let Some(custom) = custom_rules.remove(&rule.id)
        {
          (custom, RuleStatus::Custom)
        } else if disabled.contains(&rule.id) {
          (rule, RuleStatus::Disabled)
        } else {
          (rule, RuleStatus::Enabled)
        };

        ResolvedRule { rule, status }
      })
      .collect::<Vec<_>>();

    rules.sort_by(|a, b| a.rule.id.cmp(&b.rule.id));

    rules.extend(custom_rules.into_values().map(|rule| ResolvedRule {
      rule,
      status: RuleStatus::Custom,
    }));

    Ok(rules)
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
        Config {
          rules,
          ..Config::default()
        }
        .resolve()
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
    let disabled = Rule::builtins()
      .unwrap()
      .into_iter()
      .map(|rule| rule.id)
      .filter(|id| id != "node")
      .collect::<Vec<_>>();

    let rules = Config {
      default_rules: DefaultRulesConfig {
        disabled: disabled.clone(),
      },
      rules: vec![
        rule("foo"),
        rule("bar"),
        RuleConfig {
          actions: vec![ConfigAction::Command {
            command: "foo".into(),
          }],
          name: Some("baz".into()),
          ..rule("cargo")
        },
      ],
    }
    .resolve()
    .unwrap();

    assert_eq!(
      rules
        .iter()
        .filter(|rule| rule.status == RuleStatus::Disabled)
        .map(|rule| &rule.rule.id)
        .collect::<Vec<_>>(),
      disabled
        .iter()
        .filter(|id| *id != "cargo")
        .collect::<Vec<_>>(),
    );

    assert_eq!(
      rules
        .iter()
        .filter(|rule| rule.status != RuleStatus::Disabled)
        .map(|ResolvedRule { rule, status }| (
          rule.id.as_str(),
          rule.name.as_str(),
          rule.detection.to_string(),
          rule
            .actions
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
          *status,
        ))
        .collect::<Vec<_>>(),
      vec![
        (
          "cargo",
          "baz",
          "bar".into(),
          vec!["run `foo`".into()],
          RuleStatus::Custom,
        ),
        (
          "node",
          "Node",
          "package.json".into(),
          vec![
            "remove **/node_modules".into(),
            "remove .angular/cache".into()
          ],
          RuleStatus::Enabled,
        ),
        (
          "bar",
          "bar",
          "bar".into(),
          vec!["remove foo".into()],
          RuleStatus::Custom,
        ),
        (
          "foo",
          "foo",
          "bar".into(),
          vec!["remove foo".into()],
          RuleStatus::Custom,
        ),
      ],
    );
  }
}
