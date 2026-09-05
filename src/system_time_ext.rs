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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn format() {
    #[track_caller]
    fn case(time: SystemTime, expected: &str) {
      assert_eq!(time.format(), expected);
    }

    let now = SystemTime::now();

    case(now, "0 seconds ago");
    case(now - Duration::from_secs(1), "1 second ago");
    case(now - Duration::from_secs(59), "59 seconds ago");
    case(now - Duration::from_mins(1), "1 minute ago");
    case(now - Duration::from_mins(5), "5 minutes ago");
    case(now - Duration::from_mins(59), "59 minutes ago");
    case(now - Duration::from_hours(1), "1 hour ago");
    case(now - Duration::from_hours(12), "12 hours ago");
    case(now - Duration::from_hours(23), "23 hours ago");
    case(now - Duration::from_hours(24), "1 day ago");
    case(now - Duration::from_hours(168), "7 days ago");
    case(now + Duration::from_mins(1), "0 seconds ago");
  }
}
