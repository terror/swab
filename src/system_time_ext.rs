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

  fn ago(duration: Duration) -> SystemTime {
    SystemTime::now() - duration
  }

  #[test]
  fn format_zero_seconds() {
    assert_eq!(ago(Duration::from_secs(0)).format(), "0 seconds ago");
  }

  #[test]
  fn format_one_second() {
    assert_eq!(ago(Duration::from_secs(1)).format(), "1 second ago");
  }

  #[test]
  fn format_59_seconds() {
    assert_eq!(ago(Duration::from_secs(59)).format(), "59 seconds ago");
  }

  #[test]
  fn format_one_minute() {
    assert_eq!(ago(Duration::from_mins(1)).format(), "1 minute ago");
  }

  #[test]
  fn format_multiple_minutes() {
    assert_eq!(ago(Duration::from_mins(5)).format(), "5 minutes ago");
  }

  #[test]
  fn format_59_minutes() {
    assert_eq!(ago(Duration::from_mins(59)).format(), "59 minutes ago");
  }

  #[test]
  fn format_one_hour() {
    assert_eq!(ago(Duration::from_hours(1)).format(), "1 hour ago");
  }

  #[test]
  fn format_multiple_hours() {
    assert_eq!(ago(Duration::from_hours(12)).format(), "12 hours ago");
  }

  #[test]
  fn format_23_hours() {
    assert_eq!(ago(Duration::from_hours(23)).format(), "23 hours ago");
  }

  #[test]
  fn format_one_day() {
    assert_eq!(ago(Duration::from_hours(24)).format(), "1 day ago");
  }

  #[test]
  fn format_multiple_days() {
    assert_eq!(ago(Duration::from_hours(168)).format(), "7 days ago");
  }

  #[test]
  fn format_future_time_returns_zero() {
    assert_eq!(
      (SystemTime::now() + Duration::from_mins(1)).format(),
      "0 seconds ago"
    );
  }
}
