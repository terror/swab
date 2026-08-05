use super::*;

define_rule! {
  Python {
    id: "python",
    name: "Python",
    detection: Detection::Any(
      Box::new(Detection::Pattern("pyproject.toml")),
      Box::new(Detection::Any(
        Box::new(Detection::Pattern("setup.py")),
        Box::new(Detection::Pattern("setup.cfg")),
      )),
    ),
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
