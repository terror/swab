use super::*;

#[derive(Debug)]
enum RuleSource {
  Builtin,
  Custom,
  Disabled,
}

fn print_builtin_rule(style: Style, rule: &dyn Rule, source: RuleSource) {
  let status = match source {
    RuleSource::Builtin => style.apply(GREEN, "enabled"),
    RuleSource::Custom => style.apply(YELLOW, "custom"),
    RuleSource::Disabled => style.apply(RED, "disabled"),
  };

  println!(
    "{} ({}) [{}]",
    style.apply(BOLD, rule.name()),
    style.apply(DIM, rule.id()),
    status,
  );

  println!("  {}: {}", style.apply(CYAN, "detection"), rule.detection(),);

  println!("  {}:", style.apply(CYAN, "actions"));

  for action in rule.actions() {
    println!("    {action}");
  }
}

fn print_custom_rule(style: Style, rule: &config::RuleConfig) {
  let name = rule.name.as_deref().unwrap_or(&rule.id);

  println!(
    "{} ({}) [{}]",
    style.apply(BOLD, name),
    style.apply(DIM, &rule.id),
    style.apply(YELLOW, "custom"),
  );

  println!("  {}: {}", style.apply(CYAN, "detection"), rule.detection,);

  println!("  {}:", style.apply(CYAN, "actions"));

  for action in &rule.actions {
    println!("    {action}");
  }
}

pub(crate) fn run() -> Result {
  let style = Style::stdout();

  let config = Config::load()?;

  let disabled = config
    .default_rules
    .disabled
    .iter()
    .cloned()
    .collect::<HashSet<_>>();

  let custom_ids = config
    .rules
    .iter()
    .map(|rule| rule.id.clone())
    .collect::<HashSet<_>>();

  let mut default_rules = Config::default_rules().collect::<Vec<_>>();

  default_rules.sort_by_key(|rule| rule.id());

  for rule in &default_rules {
    let id = rule.id();

    let source = if custom_ids.contains(id) {
      RuleSource::Custom
    } else if disabled.contains(id) {
      RuleSource::Disabled
    } else {
      RuleSource::Builtin
    };

    print_builtin_rule(style, *rule, source);
  }

  let default_ids = default_rules
    .iter()
    .map(|rule| rule.id())
    .collect::<HashSet<_>>();

  let mut new_custom_rules = config
    .rules
    .iter()
    .filter(|r| !default_ids.contains(r.id.as_str()))
    .collect::<Vec<_>>();

  new_custom_rules.sort_by(|a, b| a.id.cmp(&b.id));

  for rule in new_custom_rules {
    print_custom_rule(style, rule);
  }

  Ok(())
}
