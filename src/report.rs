use super::*;

#[derive(Debug)]
pub(crate) struct Report {
  pub(crate) bytes: u64,
  pub(crate) items: Vec<(u64, PathBuf)>,
  pub(crate) modified: SystemTime,
  pub(crate) root: PathBuf,
  pub(crate) rule_name: String,
}

impl Display for Report {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let age = self.modified.format();

    writeln!(
      f,
      "{} {} project ({age})",
      self.root.display(),
      self.rule_name
    )?;

    for (index, item) in self.items.iter().enumerate() {
      let branch = if index + 1 == self.items.len() {
        "└─"
      } else {
        "├─"
      };

      writeln!(f, "  {branch} {} ({})", item.1.display(), Bytes(item.0))?;
    }

    Ok(())
  }
}

impl TryFrom<(&Context, &dyn Rule)> for Report {
  type Error = Error;

  fn try_from((context, rule): (&Context, &dyn Rule)) -> Result<Self> {
    let mut items = Vec::new();

    let mut total_bytes = 0;

    for relative_path in context.matches(rule)? {
      let full_path = context.root.join(&relative_path);

      let bytes = full_path.size()?;

      total_bytes += bytes;

      items.push((bytes, relative_path))
    }

    items.sort_by(|left, right| left.1.cmp(&right.1));

    Ok(Report {
      bytes: total_bytes,
      items,
      modified: context.modified_time(),
      root: context.root.clone(),
      rule_name: rule.name().to_string(),
    })
  }
}
