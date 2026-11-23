use super::*;

#[derive(Debug)]
pub(crate) struct Context {
  pub(crate) follow_symlinks: bool,
  pub(crate) paths: HashSet<PathBuf>,
  pub(crate) root: PathBuf,
}

impl Context {
  pub(crate) fn contains(&self, matcher: &GlobMatcher) -> bool {
    self.paths.iter().any(|path| matcher.is_match(path))
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
          .paths
          .iter()
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

      let Ok(metadata) = self.metadata(&relative_path) else {
        continue;
      };

      if metadata.is_dir() {
        containing_directory = Some(relative_path.clone());
      }

      pruned.push(relative_path);
    }

    pruned
  }

  fn metadata(&self, relative: &Path) -> io::Result<fs::Metadata> {
    let path = self.root.join(relative);

    if self.follow_symlinks {
      fs::metadata(path)
    } else {
      fs::symlink_metadata(path)
    }
  }

  fn modified(&self, excluded: &[&Path]) -> Result<SystemTime> {
    let modified = self
      .paths
      .iter()
      .filter(|path| {
        !excluded.iter().any(|excluded| path.starts_with(excluded))
      })
      .filter_map(|path| {
        self
          .metadata(path)
          .and_then(|metadata| metadata.modified())
          .ok()
      })
      .max();

    match modified {
      Some(modified) => Ok(modified),
      None => Ok(fs::metadata(&self.root)?.modified()?),
    }
  }

  pub(crate) fn new(root: PathBuf, follow_symlinks: bool) -> Result<Self> {
    let mut paths = HashSet::new();

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

      paths.insert(relative);
    }

    Ok(Self {
      follow_symlinks,
      paths,
      root,
    })
  }

  pub(crate) fn reports(&self, rules: &[Rule]) -> Result<Vec<Report>> {
    let tasks = rules
      .iter()
      .filter(|rule| rule.detection.matches(self))
      .map(|rule| Ok((rule, self.tasks(rule)?)))
      .collect::<Result<Vec<_>>>()?
      .into_iter()
      .filter(|(_, tasks)| !tasks.is_empty())
      .collect::<Vec<_>>();

    if tasks.is_empty() {
      return Ok(Vec::new());
    }

    let excluded = tasks
      .iter()
      .flat_map(|(_, tasks)| tasks)
      .filter_map(|task| match task {
        Task::Remove { path, .. } => Some(path.as_path()),
        Task::Command(_) => None,
      })
      .collect::<Vec<_>>();

    let modified = self.modified(&excluded)?;

    Ok(
      tasks
        .into_iter()
        .map(|(rule, tasks)| Report {
          modified,
          root: self.root.clone(),
          rule_name: rule.name.clone(),
          tasks,
        })
        .collect(),
    )
  }

  fn tasks(&self, rule: &Rule) -> Result<Vec<Task>> {
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

    Ok(tasks)
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
  fn matches_returns_empty_when_no_patterns_match() {
    let tree = temptree! {
      "README.md": "hello",
    };

    let context = Context::new(tree.path().to_path_buf(), false).unwrap();

    let rule = rule(vec![Action::remove("nope/**").unwrap()]);

    assert_eq!(context.matches(&rule), Vec::<PathBuf>::new());
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
  fn report_owns_commands() {
    let tree = temptree! {
      "foo": "",
    };

    let context = Context::new(tree.path().to_path_buf(), false).unwrap();

    let reports = context
      .reports(&[rule(vec![Action::Command("foo".into())])])
      .unwrap();

    assert!(matches!(
      reports.as_slice(),
      [Report { tasks, .. }] if matches!(
        tasks.as_slice(),
        [Task::Command(command)] if command == "foo",
      ),
    ));
  }
}
