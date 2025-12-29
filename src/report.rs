use super::*;

#[derive(Debug)]
pub(crate) struct Report {
  pub(crate) bytes: u64,
  pub(crate) commands: Vec<&'static str>,
  pub(crate) items: Vec<(u64, PathBuf)>,
  pub(crate) modified: SystemTime,
  pub(crate) root: PathBuf,
  pub(crate) rule_name: String,
}

impl Display for Report {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let style = Style::stdout();

    let age = self.modified.format();

    writeln!(
      f,
      "{} {} project ({})",
      style.apply(CYAN, self.root.display()),
      style.apply(BOLD, self.rule_name.as_str()),
      style.apply(DIM, age),
    )?;

    let total_entries = self.items.len() + self.commands.len();

    for (index, item) in self.items.iter().enumerate() {
      let branch = if index + 1 == total_entries {
        "└─"
      } else {
        "├─"
      };

      writeln!(
        f,
        "  {} {} {}",
        style.apply(DIM, branch),
        item.1.display(),
        style.apply(GREEN, format_args!("({})", Bytes(item.0))),
      )?;
    }

    for (index, command) in self.commands.iter().enumerate() {
      let entry_index = self.items.len() + index;
      let branch = if entry_index + 1 == total_entries {
        "└─"
      } else {
        "├─"
      };

      writeln!(
        f,
        "  {} {} {}",
        style.apply(DIM, branch),
        style.apply(DIM, "run"),
        style.apply(YELLOW, command),
      )?;
    }

    Ok(())
  }
}

impl TryFrom<(&Context, &dyn Rule)> for Report {
  type Error = Error;

  fn try_from((context, rule): (&Context, &dyn Rule)) -> Result<Self> {
    let mut items = Vec::new();
    let mut commands = Vec::new();

    let mut total_bytes = 0;

    for action in rule.actions() {
      if let Action::Command(command) = action {
        commands.push(*command);
      }
    }

    for relative_path in context.matches(rule)? {
      let full_path = context.root.join(&relative_path);

      let bytes = full_path.size()?;

      total_bytes += bytes;

      items.push((bytes, relative_path));
    }

    items.sort_by(|left, right| left.1.cmp(&right.1));

    Ok(Report {
      bytes: total_bytes,
      commands,
      items,
      modified: context.modified_time(),
      root: context.root.clone(),
      rule_name: rule.name().to_string(),
    })
  }
}
