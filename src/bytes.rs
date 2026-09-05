use super::*;

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct Bytes(pub(crate) u64);

fn int_to_float(x: u64) -> f64 {
  #![allow(clippy::as_conversions, clippy::cast_precision_loss)]
  x as f64
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

  const KI: u64 = 1 << 10;
  const MI: u64 = KI << 10;
  const GI: u64 = MI << 10;
  const TI: u64 = GI << 10;
  const PI: u64 = TI << 10;
  const EI: u64 = PI << 10;

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
