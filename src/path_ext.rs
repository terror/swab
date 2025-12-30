use super::*;

pub(crate) trait PathExt {
  fn directories(&self, follow_symlinks: bool) -> Result<Vec<PathBuf>>;
  fn size(&self, follow_symlinks: bool) -> Result<u64>;
}

impl PathExt for Path {
  fn directories(&self, follow_symlinks: bool) -> Result<Vec<PathBuf>> {
    let mut directories = Vec::new();

    for entry in fs::read_dir(self)? {
      let entry = entry?;

      let path = entry.path();

      let is_dir = if follow_symlinks {
        path.is_dir()
      } else {
        entry
          .file_type()
          .map(|file_type| file_type.is_dir())
          .unwrap_or(false)
      };

      if is_dir {
        directories.push(path);
      }
    }

    directories.sort_unstable();

    Ok(directories)
  }

  fn size(&self, follow_symlinks: bool) -> Result<u64> {
    let metadata = if follow_symlinks {
      fs::metadata(self)?
    } else {
      fs::symlink_metadata(self)?
    };

    if metadata.is_file() {
      return Ok(metadata.len());
    }

    if !follow_symlinks && metadata.file_type().is_symlink() {
      return Ok(metadata.len());
    }

    if !metadata.is_dir() {
      return Ok(0);
    }

    let mut total = 0;

    for entry in WalkDir::new(self).follow_links(follow_symlinks) {
      let entry = entry?;

      if entry.file_type().is_file() {
        total += entry.metadata()?.len();
      } else if !follow_symlinks && entry.file_type().is_symlink() {
        total += fs::symlink_metadata(entry.path())?.len();
      }
    }

    Ok(total)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn size_of_file() {
    let tree = temptree::temptree! {
      "test.txt": "hello"
    };

    assert_eq!(tree.path().join("test.txt").size(false).unwrap(), 5);
  }

  #[test]
  fn size_of_empty_file() {
    let tree = temptree::temptree! {
      "empty.txt": ""
    };

    assert_eq!(tree.path().join("empty.txt").size(false).unwrap(), 0);
  }

  #[test]
  fn size_of_empty_directory() {
    assert_eq!(temptree::temptree! {}.path().size(false).unwrap(), 0);
  }

  #[test]
  fn size_of_directory_with_files() {
    let tree = temptree::temptree! {
      "a.txt": "aaa",
      "b.txt": "bbbbb"
    };

    assert_eq!(tree.path().size(false).unwrap(), 8);
  }

  #[test]
  fn size_of_nested_directory() {
    let tree = temptree::temptree! {
      "root.txt": "root",
      "subdir": {
        "nested.txt": "nested"
      }
    };

    assert_eq!(tree.path().size(false).unwrap(), 10);
  }

  #[test]
  fn size_of_deeply_nested_directory() {
    let tree = temptree::temptree! {
      "a.txt": "a",
      "level1": {
        "b.txt": "bb",
        "level2": {
          "c.txt": "ccc"
        }
      }
    };

    assert_eq!(tree.path().size(false).unwrap(), 6);
  }

  #[test]
  fn size_of_subdirectory() {
    let tree = temptree::temptree! {
      "root.txt": "root",
      "subdir": {
        "nested.txt": "nested"
      }
    };

    assert_eq!(tree.path().join("subdir").size(false).unwrap(), 6);
  }

  #[test]
  fn size_of_empty_subdirectory() {
    let tree = temptree::temptree! {
      "subdir": {}
    };

    assert_eq!(tree.path().join("subdir").size(false).unwrap(), 0);
  }

  #[test]
  fn size_of_nonexistent_path_returns_error() {
    assert!(
      temptree::temptree! {}
        .path()
        .join("does_not_exist")
        .size(false)
        .is_err()
    );
  }

  #[test]
  fn directories_returns_sorted_subdirectories() {
    let tree = temptree::temptree! {
      "file.txt": "content",
      "zebra": {},
      "alpha": {},
      "middle": {}
    };

    let directories = tree.path().directories(false).unwrap();

    assert_eq!(directories.len(), 3);

    assert_eq!(directories[0], tree.path().join("alpha"));
    assert_eq!(directories[1], tree.path().join("middle"));
    assert_eq!(directories[2], tree.path().join("zebra"));
  }

  #[test]
  fn directories_excludes_files() {
    let tree = temptree::temptree! {
      "file1.txt": "content",
      "file2.txt": "content",
      "only_dir": {}
    };

    let directories = tree.path().directories(false).unwrap();

    assert_eq!(directories.len(), 1);

    assert_eq!(*directories.first().unwrap(), tree.path().join("only_dir"));
  }

  #[test]
  fn directories_empty_directory() {
    let tree = temptree::temptree! {};

    let directories = tree.path().directories(false).unwrap();

    assert!(directories.is_empty());
  }

  #[test]
  fn directories_nonexistent_path_returns_error() {
    assert!(
      temptree::temptree! {}
        .path()
        .join("does_not_exist")
        .directories(false)
        .is_err()
    );
  }
}
