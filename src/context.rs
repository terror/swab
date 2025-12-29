use super::*;

#[derive(Debug)]
pub(crate) struct Context {
  pub(crate) directories: HashSet<PathBuf>,
  pub(crate) files: HashSet<PathBuf>,
  pub(crate) root: PathBuf,
}

impl TryFrom<PathBuf> for Context {
  type Error = Error;

  fn try_from(value: PathBuf) -> Result<Self> {
    let mut directories = HashSet::new();
    let mut files = HashSet::new();

    for entry in WalkDir::new(&value).follow_links(false) {
      let entry = entry?;

      if entry.depth() == 0 {
        continue;
      }

      let relative = entry
        .path()
        .strip_prefix(&value)
        .unwrap_or(entry.path())
        .to_path_buf();

      if entry.file_type().is_dir() {
        directories.insert(relative);
      } else {
        files.insert(relative);
      }
    }

    Ok(Self {
      directories,
      files,
      root: value,
    })
  }
}

impl Context {
  pub(crate) fn matches(&self, rule: &dyn Rule) -> Result<Vec<PathBuf>> {
    let mut matches = HashSet::new();

    for action in rule.actions() {
      let matcher = Glob::new(action.pattern)?.compile_matcher();

      for path in self.directories.iter().chain(self.files.iter()) {
        if matcher.is_match(path) {
          matches.insert(path.clone());
        }
      }
    }

    let mut matched = matches.into_iter().collect::<Vec<PathBuf>>();
    matched.sort_unstable();

    let mut pruned = Vec::new();
    let mut kept_directories = Vec::new();

    for relative_path in matched {
      let full_path = self.root.join(&relative_path);

      if !full_path.exists() {
        continue;
      }

      if kept_directories
        .iter()
        .any(|dir| relative_path.starts_with(dir))
      {
        continue;
      }

      if full_path.is_dir() {
        kept_directories.push(relative_path.clone());
      }

      pruned.push(relative_path);
    }

    Ok(pruned)
  }

  pub(crate) fn modified_time(&self) -> SystemTime {
    fs::metadata(&self.root)
      .and_then(|metadata| metadata.modified())
      .unwrap_or_else(|_| SystemTime::now())
  }
}
