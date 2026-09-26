use super::*;

#[derive(Debug)]
pub(crate) struct Context {
  pub(crate) directories: HashSet<PathBuf>,
  pub(crate) files: HashSet<PathBuf>,
  pub(crate) follow_symlinks: bool,
  pub(crate) root: PathBuf,
}

impl Context {
  pub(crate) fn contains(&self, matcher: &GlobMatcher) -> bool {
    self
      .directories
      .iter()
      .chain(self.files.iter())
      .any(|path| matcher.is_match(path))
  }

  pub(crate) fn matches(&self, rule: &Rule) -> Vec<PathBuf> {
    let matches = rule
      .actions
      .iter()
      .filter_map(|action| match action {
        Action::Remove(matcher) => Some(matcher),
        Action::Command(_) => None,
      })
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

    let mut pruned = Vec::new();
    let mut containing_directory = None;

    for relative_path in matched {
      if containing_directory
        .as_ref()
        .is_some_and(|directory| relative_path.starts_with(directory))
      {
        continue;
      }

      containing_directory = None;

      let full_path = self.root.join(&relative_path);

      let metadata = if self.follow_symlinks {
        fs::metadata(&full_path)
      } else {
        fs::symlink_metadata(&full_path)
      };

      let Ok(metadata) = metadata else {
        continue;
      };

      if metadata.is_dir() {
        containing_directory = Some(relative_path.clone());
      }

      pruned.push(relative_path);
    }

    pruned
  }

  pub(crate) fn modified_time(&self) -> Result<SystemTime> {
    Ok(fs::metadata(&self.root)?.modified()?)
  }

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

  pub(crate) fn report(&self, rule: &Rule) -> Result<Report> {
    let mut tasks = Vec::new();

    for action in &rule.actions {
      if let Action::Command(command) = action {
        tasks.push(Task::Command(command.clone()));
      }
    }

    for relative_path in self.matches(rule) {
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
      rule_name: rule.name.clone(),
      tasks,
    })
  }
}

#[cfg(test)]
mod tests {
  use {super::*, temptree::temptree};

  fn rule(actions: Vec<Action>) -> Rule {
    Rule {
      actions,
      detection: Detection::pattern("**").unwrap(),
      id: "foo".into(),
      name: "foo".into(),
    }
  }

  #[test]
  fn matches_returns_empty_when_no_patterns_match() {
    let tree = temptree! {
      "README.md": "hello",
    };

    let context = Context::new(tree.path().to_path_buf(), false).unwrap();

    let rule = rule(vec![Action::remove("nope/**").unwrap()]);

    assert_eq!(context.matches(&rule), Vec::<PathBuf>::new());
  }

  #[test]
  fn matches_only_files() {
    let tree = temptree! {
      "b.log": "b",
      "a.log": "a",
      "foo": {
        "bar.log": "baz",
      },
    };

    let context = Context::new(tree.path().to_path_buf(), false).unwrap();

    let rule = rule(vec![Action::remove("*.log").unwrap()]);

    assert_eq!(
      context.matches(&rule),
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

    let rule = rule(vec![Action::remove("*.log").unwrap()]);

    assert_eq!(context.matches(&rule), Vec::<PathBuf>::new());
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

    let rule = rule(vec![
      Action::remove("node_modules").unwrap(),
      Action::remove("node_modules/**").unwrap(),
      Action::remove("target").unwrap(),
      Action::remove("target/**").unwrap(),
      Action::remove("*.md").unwrap(),
      Action::Command("echo foo".into()),
    ]);

    assert_eq!(
      context.matches(&rule),
      vec![
        PathBuf::from("README.md"),
        PathBuf::from("node_modules"),
        PathBuf::from("target"),
      ],
    );
  }

  #[test]
  fn report_owns_commands() {
    let tree = temptree! {};

    let context = Context::new(tree.path().to_path_buf(), false).unwrap();

    let report = context
      .report(&rule(vec![Action::Command("foo".into())]))
      .unwrap();

    assert!(matches!(
      report.tasks.as_slice(),
      [Task::Command(command)] if command == "foo",
    ));
  }
}
