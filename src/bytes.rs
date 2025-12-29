use super::*;
use std::{
  fmt::{self, Display, Formatter},
  str::FromStr,
};

const KI: u64 = 1 << 10;
const MI: u64 = KI << 10;
const GI: u64 = MI << 10;
const TI: u64 = GI << 10;
const PI: u64 = TI << 10;
const EI: u64 = PI << 10;

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct Bytes(pub(crate) u64);

fn float_to_int(x: f64) -> u64 {
  #![allow(
    clippy::as_conversions,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation
  )]
  x as u64
}

fn int_to_float(x: u64) -> f64 {
  #![allow(clippy::as_conversions, clippy::cast_precision_loss)]
  x as f64
}

impl<I: Into<u64>> From<I> for Bytes {
  fn from(n: I) -> Bytes {
    Bytes(n.into())
  }
}

impl FromStr for Bytes {
  type Err = Error;

  fn from_str(text: &str) -> Result<Self, Self::Err> {
    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn is_digit(c: &char) -> bool {
      matches!(c, '0'..='9' | '.')
    }

    let digits = text.chars().take_while(is_digit).collect::<String>();

    let suffix = text.chars().skip_while(is_digit).collect::<String>();

    let value = digits.parse::<f64>()?;

    let multiple = match suffix.to_lowercase().as_str() {
      "" | "b" | "byte" | "bytes" => 1,
      "kib" => KI,
      "mib" => MI,
      "gib" => GI,
      "tib" => TI,
      "pib" => PI,
      "eib" => EI,
      _ => {
        bail!("invalid byte suffix: {suffix}");
      }
    };

    Ok(Bytes(float_to_int(value * int_to_float(multiple))))
  }
}

impl Display for Bytes {
  #![allow(clippy::float_cmp)]
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    const DISPLAY_SUFFIXES: &[&str] =
      &["KiB", "MiB", "GiB", "TiB", "PiB", "EiB"];

    let mut value = int_to_float(self.0);

    let mut i = 0;

    while value >= 1024.0 {
      value /= 1024.0;
      i += 1;
    }

    let suffix = if i == 0 {
      if value == 1.0 { "byte" } else { "bytes" }
    } else {
      DISPLAY_SUFFIXES[i - 1]
    };

    let formatted = format!("{value:.2}");

    write!(
      f,
      "{} {suffix}",
      formatted.trim_end_matches('0').trim_end_matches('.')
    )
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse_plain_numbers() {
    const CASES: &[(&str, u64)] = &[("0", 0), ("1", 1)];

    for (text, value) in CASES {
      assert_eq!(
        text.parse::<Bytes>().unwrap(),
        Bytes(*value),
        "text: {text}",
      );
    }
  }

  #[test]
  fn parse_byte_suffixes() {
    const CASES: &[(&str, u64)] =
      &[("1b", 1), ("1byte", 1), ("1bytes", 1), ("0kib", 0)];

    for (text, value) in CASES {
      assert_eq!(
        text.parse::<Bytes>().unwrap(),
        Bytes(*value),
        "text: {text}",
      );
    }
  }

  #[test]
  fn parse_binary_units() {
    const CASES: &[(&str, u64)] = &[
      ("1kib", KI),
      ("1KiB", KI),
      ("12kib", 12 * KI),
      ("1.5mib", MI + 512 * KI),
    ];

    for (text, value) in CASES {
      assert_eq!(
        text.parse::<Bytes>().unwrap(),
        Bytes(*value),
        "text: {text}",
      );
    }
  }

  #[test]
  fn parse_invalid_suffix() {
    assert_eq!(
      "100foo".parse::<Bytes>().unwrap_err().to_string(),
      "invalid byte suffix: foo"
    );
  }

  #[test]
  fn parse_invalid_number() {
    assert_eq!(
      "1.0.0foo".parse::<Bytes>().unwrap_err().to_string(),
      "invalid float literal"
    );
  }

  #[test]
  fn display_bytes() {
    assert_eq!(Bytes(0).to_string(), "0 bytes");
    assert_eq!(Bytes(1).to_string(), "1 byte");
    assert_eq!(Bytes(2).to_string(), "2 bytes");
  }

  #[test]
  fn display_binary_units() {
    assert_eq!(Bytes(KI).to_string(), "1 KiB");
    assert_eq!(Bytes(512 * KI).to_string(), "512 KiB");
    assert_eq!(Bytes(MI).to_string(), "1 MiB");
    assert_eq!(Bytes(MI + 512 * KI).to_string(), "1.5 MiB");
  }

  #[test]
  fn display_large_units() {
    assert_eq!(Bytes(1024 * MI + 512 * MI).to_string(), "1.5 GiB");
    assert_eq!(Bytes(GI).to_string(), "1 GiB");
    assert_eq!(Bytes(TI).to_string(), "1 TiB");
    assert_eq!(Bytes(PI).to_string(), "1 PiB");
    assert_eq!(Bytes(EI).to_string(), "1 EiB");
  }
}
