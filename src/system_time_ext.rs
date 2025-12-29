use super::*;

pub(crate) trait SystemTimeExt {
  fn format(self) -> String;
}

impl SystemTimeExt for SystemTime {
  fn format(self) -> String {
    let duration = SystemTime::now()
      .duration_since(self)
      .unwrap_or(Duration::ZERO);

    let seconds = duration.as_secs();

    let plural_suffix =
      |value: u64| -> &'static str { if value == 1 { "" } else { "s" } };

    if seconds < 60 {
      return format!("{seconds} second{} ago", plural_suffix(seconds));
    }

    let minutes = seconds / 60;

    if minutes < 60 {
      return format!("{minutes} minute{} ago", plural_suffix(minutes));
    }

    let hours = minutes / 60;

    if hours < 24 {
      return format!("{hours} hour{} ago", plural_suffix(hours));
    }

    let days = hours / 24;

    format!("{days} day{} ago", plural_suffix(days))
  }
}
