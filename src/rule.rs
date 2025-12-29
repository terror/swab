use super::*;

pub(crate) use {cargo::Cargo, node::Node, zig::Zig};

mod cargo;
mod node;
mod zig;

pub(crate) trait Rule: Sync {
  /// A unique identifier for the rule.
  #[allow(unused)]
  fn id(&self) -> &'static str;

  /// A human-readable name for the rule.
  fn name(&self) -> &'static str;

  /// Determines if the rule applies to the given context.
  fn applies(&self, context: &Context) -> bool;

  /// A description of what the rule does.
  fn actions(&self) -> &[Action];
}
