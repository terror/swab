#[derive(Debug)]
pub(crate) enum Action {
  Command(&'static str),
  Remove(&'static str),
}
