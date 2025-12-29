use super::*;

pub(crate) trait PathExt {
  fn size(&self) -> Result<u64>;
}

impl PathExt for Path {
  fn size(&self) -> Result<u64> {
    let metadata = fs::metadata(self)?;

    if metadata.is_file() {
      return Ok(metadata.len());
    }

    if !metadata.is_dir() {
      return Ok(0);
    }

    let mut total = 0;

    for entry in WalkDir::new(self).follow_links(false) {
      let entry = entry?;

      if entry.file_type().is_file() {
        total += entry.metadata()?.len();
      }
    }

    Ok(total)
  }
}
