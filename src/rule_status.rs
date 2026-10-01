#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RuleStatus {
  Custom,
  Disabled,
  Enabled,
}
