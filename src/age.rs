use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Age(pub(crate) Duration);

impl Age {
  pub(crate) fn older_than(&self, modified: SystemTime) -> bool {
    let Ok(elapsed) = modified.elapsed() else {
      return false;
    };

    elapsed > self.0
  }
}

impl FromStr for Age {
  type Err = Error;

  fn from_str(text: &str) -> Result<Self, Self::Err> {
    let text = text.trim().trim_end_matches("ago").trim();

    let digits = text
      .chars()
      .take_while(char::is_ascii_digit)
      .collect::<String>();

    let suffix = text.chars().skip(digits.len()).collect::<String>();

    let amount = digits
      .parse::<u64>()
      .map_err(|_| anyhow!("invalid age amount: `{digits}`"))?;

    let seconds = match suffix.as_str() {
      "s" => 1,
      "m" => 60,
      "h" => 60 * 60,
      "d" => 60 * 60 * 24,
      "w" => 60 * 60 * 24 * 7,
      "mo" => 60 * 60 * 24 * 30,
      "y" => 60 * 60 * 24 * 365,
      _ => bail!("invalid age unit: `{suffix}`"),
    };

    let seconds = amount
      .checked_mul(seconds)
      .ok_or_else(|| anyhow!("age is too large: `{text}`"))?;

    Ok(Age(Duration::from_secs(seconds)))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn older_than() {
    let now = SystemTime::now();
    let age = Age(Duration::from_mins(1));

    assert!(!age.older_than(now));
    assert!(age.older_than(now - Duration::from_mins(2)));
  }

  #[test]
  fn parse_invalid_amount() {
    assert_eq!(
      "abcd".parse::<Age>().unwrap_err().to_string(),
      "invalid age amount: ``"
    );
  }

  #[test]
  fn parse_invalid_unit() {
    assert_eq!(
      "5x".parse::<Age>().unwrap_err().to_string(),
      "invalid age unit: `x`"
    );
  }

  #[test]
  fn parse_overflow() {
    assert_eq!(
      "307445734561825861m"
        .parse::<Age>()
        .unwrap_err()
        .to_string(),
      "age is too large: `307445734561825861m`"
    );
  }

  #[test]
  fn parsing() {
    #[track_caller]
    fn case(text: &str, expected_secs: u64) {
      assert_eq!(
        text.parse::<Age>().unwrap(),
        Age(Duration::from_secs(expected_secs)),
      );
    }

    case("1s", 1);
    case("5s", 5);
    case("1m", 60);
    case("5m", 300);
    case("1h", 3600);
    case("2h", 7200);
    case("1d", 86400);
    case("7d", 604_800);
    case("1w", 604_800);
    case("2w", 1_209_600);
    case("1mo", 2_592_000);
    case("1y", 31_536_000);
    case("18446744073709551615s", u64::MAX);
    case("307445734561825860m", 18_446_744_073_709_551_600);
  }

  #[test]
  fn parsing_with_ago_suffix() {
    #[track_caller]
    fn case(text: &str, expected_secs: u64) {
      assert_eq!(
        text.parse::<Age>().unwrap(),
        Age(Duration::from_secs(expected_secs)),
      );
    }

    case("1d ago", 86400);
    case("1dago", 86400);
    case("  5d ago  ", 432_000);
  }
}
