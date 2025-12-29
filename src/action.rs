#[derive(Debug)]
pub(crate) struct Action {
  pub(crate) pattern: &'static str,
  #[allow(unused)]
  pub(crate) reason: &'static str,
}
