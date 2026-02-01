use super::*;

#[macro_export]
macro_rules! define_rule {
  (
    $(#[$doc:meta])*
    $name:ident {
      id: $id:literal,
      name: $rule_name:literal,
      detection: $detection:expr,
      actions: [$($action:expr),* $(,)?] $(,)?
    }
  ) => {
    $(#[$doc])*
    pub(crate) struct $name;

    impl Rule for $name {
      fn actions(&self) -> &[Action] {
        &[$($action),*]
      }

      fn detection(&self) -> Detection {
        $detection
      }

      fn id(&self) -> &str {
        $id
      }

      fn name(&self) -> &str {
        $rule_name
      }
    }

    inventory::submit!(&$name as &(dyn Rule + Sync));
  };
}

inventory::collect!(&'static (dyn Rule + Sync));

mod cabal;
mod cargo;
mod cmake;
mod composer;
mod dotnet;
mod elixir;
mod godot;
mod gradle;
mod jupyter;
mod maven;
mod node;
mod pixi;
mod pub_;
mod python;
mod sbt;
mod stack;
mod swift;
mod turborepo;
mod unity;
mod unreal;
mod zig;

pub(crate) trait Rule: Sync {
  /// A description of what the rule does.
  fn actions(&self) -> &[Action];

  /// Builds a detection used to evaluate a context.
  fn detection(&self) -> Detection;

  /// A unique identifier for the rule.
  fn id(&self) -> &str;

  /// A human-readable name for the rule.
  fn name(&self) -> &str;
}

impl<T: Rule + ?Sized> Rule for &T {
  fn actions(&self) -> &[Action] {
    (**self).actions()
  }

  fn detection(&self) -> Detection {
    (**self).detection()
  }

  fn id(&self) -> &str {
    (**self).id()
  }

  fn name(&self) -> &str {
    (**self).name()
  }
}
