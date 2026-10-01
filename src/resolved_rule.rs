use super::*;

#[derive(Debug)]
pub(crate) struct ResolvedRule {
  pub(crate) rule: Rule,
  pub(crate) status: RuleStatus,
}
