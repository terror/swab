use super::*;

define_rule! {
  Python {
    id: "python",
    name: "Python",
    detection: Detection::Any(vec![
      Detection::Pattern("pyproject.toml"),
      Detection::Pattern("setup.py"),
      Detection::Pattern("setup.cfg"),
    ]),
    actions: [
      Action::Remove(".mypy_cache"),
      Action::Remove(".nox"),
      Action::Remove(".pytest_cache"),
      Action::Remove(".ruff_cache"),
      Action::Remove(".tox"),
      Action::Remove(".venv"),
      Action::Remove("**/__pycache__"),
      Action::Remove("__pypackages__"),
    ],
  }
}
