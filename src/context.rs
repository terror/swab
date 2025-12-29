use super::*;

#[derive(Debug)]
pub(crate) struct Context {
  pub(crate) directories: HashSet<PathBuf>,
  pub(crate) files: HashSet<PathBuf>,
  pub(crate) follow_symlinks: bool,
  pub(crate) root: PathBuf,
}

impl Context {
  pub(crate) fn new(root: PathBuf, follow_symlinks: bool) -> Result<Self> {
    let (mut directories, mut files) = (HashSet::new(), HashSet::new());

    for entry in WalkDir::new(&root).follow_links(follow_symlinks) {
      let entry = entry?;

      if entry.depth() == 0 {
        continue;
      }

      let relative = entry
        .path()
        .strip_prefix(&root)
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
      follow_symlinks,
      root,
    })
  }
}

impl Context {
  pub(crate) fn contains(&self, pattern: &str) -> bool {
    let matcher = match Glob::new(pattern) {
      Ok(glob) => glob.compile_matcher(),
      Err(_) => return false,
    };

    self
      .directories
      .iter()
      .chain(self.files.iter())
      .any(|path| matcher.is_match(path))
  }

  pub(crate) fn matches(&self, rule: &dyn Rule) -> Result<Vec<PathBuf>> {
    let matchers = rule
      .actions()
      .iter()
      .filter_map(|action| match action {
        Action::Remove(pattern) => Some(pattern),
        Action::Command(_) => None,
      })
      .map(|pattern| Ok(Glob::new(pattern)?.compile_matcher()))
      .collect::<Result<Vec<_>>>()?;

    let matches = matchers
      .into_iter()
      .flat_map(|matcher| {
        self
          .directories
          .iter()
          .chain(self.files.iter())
          .filter(move |path| matcher.is_match(path))
          .cloned()
      })
      .collect::<HashSet<_>>();

    let mut matched = matches.into_iter().collect::<Vec<PathBuf>>();
    matched.sort_unstable();

    let (pruned, _) = matched.into_iter().fold(
      (Vec::new(), Vec::new()),
      |(mut pruned, mut kept_directories), relative_path| {
        let full_path = self.root.join(&relative_path);

        if !full_path.exists() {
          return (pruned, kept_directories);
        }

        if kept_directories
          .iter()
          .any(|dir| relative_path.starts_with(dir))
        {
          return (pruned, kept_directories);
        }

        if full_path.is_dir() {
          kept_directories.push(relative_path.clone());
        }

        pruned.push(relative_path);

        (pruned, kept_directories)
      },
    );

    Ok(pruned)
  }

  pub(crate) fn modified_time(&self) -> Result<SystemTime> {
    Ok(fs::metadata(&self.root)?.modified()?)
  }

  pub(crate) fn report(&self, rule: &dyn Rule) -> Result<Report> {
    let mut tasks = Vec::new();

    for action in rule.actions() {
      if let Action::Command(command) = action {
        tasks.push(Task::Command(command));
      }
    }

    for relative_path in self.matches(rule)? {
      let full_path = self.root.join(&relative_path);

      let bytes = full_path.size(self.follow_symlinks)?;

      tasks.push(Task::Remove {
        path: relative_path,
        size: bytes,
      });
    }

    Ok(Report {
      modified: self.modified_time()?,
      root: self.root.clone(),
      rule_name: rule.name().to_string(),
      tasks,
    })
  }
}

#[cfg(test)]
mod tests {
  use {super::*, temptree::temptree};

  struct TestRule {
    actions: &'static [Action],
  }

  impl Rule for TestRule {
    fn actions(&self) -> &[Action] {
      self.actions
    }

    fn detection(&self) -> Detection {
      Detection::Pattern("**")
    }

    fn id(&self) -> &'static str {
      "test"
    }

    fn name(&self) -> &'static str {
      "test"
    }
  }

  #[test]
  fn matches_returns_empty_when_no_patterns_match() {
    let tree = temptree! {
      "README.md": "hello",
    };

    let context = Context::new(tree.path().to_path_buf(), false).unwrap();

    let rule = TestRule {
      actions: &[Action::Remove("nope/**")],
    };

    assert!(context.matches(&rule).unwrap().is_empty());
  }

  #[test]
  fn matches_only_files() {
    let tree = temptree! {
      "b.log": "b",
      "a.log": "a",
    };

    let context = Context::new(tree.path().to_path_buf(), false).unwrap();

    let rule = TestRule {
      actions: &[Action::Remove("*.log")],
    };

    assert_eq!(
      context.matches(&rule).unwrap(),
      vec![PathBuf::from("a.log"), PathBuf::from("b.log")],
    );
  }

  #[test]
  fn matches_skips_deleted_paths() {
    let tree = temptree! {
      "stale.log": "x",
    };

    let root = tree.path();

    let context = Context::new(root.to_path_buf(), false).unwrap();

    fs::remove_file(root.join("stale.log")).unwrap();

    let rule = TestRule {
      actions: &[Action::Remove("*.log")],
    };

    assert!(context.matches(&rule).unwrap().is_empty());
  }

  #[test]
  fn matches_prunes_nested_paths() {
    let tree = temptree! {
      "node_modules": {
        "left-pad": {
          "index.js": "x",
        },
      },
      "target": {
        "debug": {
          "app": "x",
        },
      },
      "README.md": "hello",
    };

    let context = Context::new(tree.path().to_path_buf(), false).unwrap();

    let rule = TestRule {
      actions: &[
        Action::Remove("node_modules"),
        Action::Remove("node_modules/**"),
        Action::Remove("target"),
        Action::Remove("target/**"),
        Action::Remove("*.md"),
        Action::Command("echo ignored"),
      ],
    };

    assert_eq!(
      context.matches(&rule).unwrap(),
      vec![
        PathBuf::from("README.md"),
        PathBuf::from("node_modules"),
        PathBuf::from("target"),
      ],
    );
  }
}
