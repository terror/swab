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

impl ConfigDetection {
  pub(crate) fn fold(
    items: Vec<ConfigDetection>,
    combine: fn(Box<Detection>, Box<Detection>) -> Detection,
    label: &str,
  ) -> Result<Detection> {
    let mut detections = items
      .into_iter()
      .map(ConfigDetection::try_into)
      .collect::<Result<Vec<_>>>()?
      .into_iter();

    let first = detections.next().ok_or_else(|| {
      anyhow!("`{label}` detection must contain at least one entry")
    })?;

    Ok(detections.fold(first, |left, right| {
      combine(Box::new(left), Box::new(right))
    }))
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

#[derive(Debug)]
struct CustomRule {
  actions: Vec<Action>,
  detection: Detection,
  id: String,
  name: String,
}

impl TryFrom<RuleConfig> for CustomRule {
  type Error = Error;

  fn try_from(rule: RuleConfig) -> Result<Self> {
    ensure!(!rule.id.trim().is_empty(), "rule id cannot be empty");

    ensure!(!rule.actions.is_empty(), "rule actions cannot be empty");

    let actions = rule
      .actions
      .into_iter()
      .map(ConfigAction::try_into)
      .collect::<Result<Vec<_>>>()?;

    Ok(Self {
      actions,
      detection: rule.detection.try_into()?,
      id: rule.id.clone(),
      name: rule.name.unwrap_or(rule.id),
    })
  }
}

impl Rule for CustomRule {
  fn actions(&self) -> &[Action] {
    &self.actions
  }

  fn detection(&self) -> Detection {
    self.detection.clone()
  }

  fn id(&self) -> &str {
    self.id.as_str()
  }

  fn name(&self) -> &str {
    self.name.as_str()
  }
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct Config {
  #[serde(alias = "default")]
  pub(crate) default_rules: DefaultRulesConfig,
  pub(crate) rules: Vec<RuleConfig>,
}

impl TryInto<Vec<Box<dyn Rule>>> for Config {
  type Error = Error;

  fn try_into(self) -> Result<Vec<Box<dyn Rule>>> {
    let mut custom_rules = self
      .rules
      .into_iter()
      .map(|rule| {
        Ok::<_, Error>((rule.id.clone(), CustomRule::try_from(rule)?))
      })
      .try_fold(HashMap::new(), |mut acc, item| {
        let (id, rule) = item?;

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

    let mut rules = Self::default_rules()
      .filter_map(|default| {
        let id = default.id().to_string();

        if let Some(custom) = custom_rules.remove(&id) {
          return Some(Box::new(custom) as Box<dyn Rule>);
        }

        if disabled.contains(&id) {
          return None;
        }

        Some(Box::new(default) as Box<dyn Rule>)
      })
      .collect::<Vec<Box<dyn Rule>>>();

    rules.extend(
      custom_rules
        .into_values()
        .map(|rule| Box::new(rule) as Box<dyn Rule>),
    );

    Ok(rules)
  }
}

impl Config {
  pub(crate) fn default_rules()
  -> impl Iterator<Item = &'static (dyn Rule + Sync)> {
    inventory::iter::<&'static (dyn Rule + Sync)>
      .into_iter()
      .copied()
  }

  pub(crate) fn load() -> Result<Self> {
    Ok(confy::load("swab", "config")?)
  }
}
