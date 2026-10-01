use super::*;

pub(crate) fn run() -> Result {
  let style = Style::stdout();

  for ResolvedRule { rule, status } in Config::load()?.resolve()? {
    let status = match status {
      RuleStatus::Enabled => style.apply(GREEN, "enabled"),
      RuleStatus::Custom => style.apply(YELLOW, "custom"),
      RuleStatus::Disabled => style.apply(RED, "disabled"),
    };

    println!(
      "{} ({}) [{}]",
      style.apply(BOLD, &rule.name),
      style.apply(DIM, &rule.id),
      status,
    );

    println!("  {}: {}", style.apply(CYAN, "detection"), rule.detection);

    println!("  {}:", style.apply(CYAN, "actions"));

    for action in &rule.actions {
      println!("    {action}");
    }
  }

  Ok(())
}
