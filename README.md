## swab

[![release](https://img.shields.io/github/release/terror/swab.svg?label=release&style=flat&labelColor=1d1d1d&color=424242&logo=github&logoColor=white)](https://github.com/terror/swab/releases/latest)
[![build](https://img.shields.io/github/actions/workflow/status/terror/swab/ci.yaml?branch=master&style=flat&labelColor=1d1d1d&color=424242&logo=GitHub%20Actions&logoColor=white&label=build)](https://github.com/terror/swab/actions/workflows/ci.yaml)
[![codecov](https://img.shields.io/codecov/c/gh/terror/swab?style=flat&labelColor=1d1d1d&color=424242&logo=Codecov&logoColor=white)](https://codecov.io/gh/terror/swab)
[![downloads](https://img.shields.io/github/downloads/terror/swab/total.svg?style=flat&labelColor=1d1d1d&color=424242&logo=github&logoColor=white)](https://github.com/terror/swab/releases)

**swab** is a configurable project cleaning tool.

<img width="1337" alt="demo" src="screenshot.png" />

Build artifacts, dependency caches, and generated files accumulate quickly across
projects. Running `cargo clean` in one project, `rm -rf node_modules` in another,
and hunting down `.venv` directories in a third gets tedious. **swab** automates
this by detecting project types and cleaning them with a single command.

We currently provide
[21 built-in rules](https://github.com/terror/swab/tree/master/src/rule) that
cover popular ecosystems: Rust (Cargo), Node.js, Python, Go, .NET, Swift, Elixir,
Zig, and many more. The rule system is designed to be easily extended with custom
rules to fit any project's specific needs.

## Installation

`swab` should run on any system, including Linux, MacOS, and Windows.

The easiest way to install it is by using
[cargo](https://doc.rust-lang.org/cargo/index.html), the Rust package manager:

```bash
cargo install swab
```

Otherwise, see below for the complete package list:

#### Cross-platform

<table>
  <thead>
    <tr>
      <th>Package Manager</th>
      <th>Package</th>
      <th>Command</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td><a href=https://www.rust-lang.org>Cargo</a></td>
      <td><a href=https://crates.io/crates/swab>swab</a></td>
      <td><code>cargo install swab</code></td>
    </tr>
    <tr>
      <td><a href=https://brew.sh>Homebrew</a></td>
      <td><a href=https://github.com/terror/homebrew-tap>terror/tap/swab</a></td>
      <td><code>brew install terror/tap/swab</code></td>
    </tr>
  </tbody>
</table>

### Pre-built binaries

Pre-built binaries for Linux, MacOS, and Windows can be found on
[the releases page](https://github.com/terror/swab/releases).

## Usage

Point `swab` at one or more directories containing projects to clean:

```bash
swab ~/projects
```

Below is the output of `swab --help`:

```present cargo run -- --help
A configurable project cleaning tool

Usage: swab [OPTIONS] [DIRECTORIES]... [COMMAND]

Commands:
  rules  List all available rules
  help   Print this message or the help of the given subcommand(s)

Arguments:
  [DIRECTORIES]...  Directories to scan for projects to clean

Options:
      --dry-run                  Enable dry run mode
      --follow-symlinks          Follow symlinks during traversal
  -i, --interactive              Prompt before each task
      --older-than <OLDER_THAN>  Only clean projects older than the specified age
  -q, --quiet                    Suppress all output
  -h, --help                     Print help
  -V, --version                  Print version
```

## Configuration

You can configure rules in a configuration file. The config file is located at:

- **Linux**: `~/.config/swab/config.toml`
- **macOS**: `~/.config/swab/config.toml`
- **Windows**: `C:\Users\<User>\AppData\Roaming\swab\config\config.toml`

To disable specific built-in rules, add them to the `disabled` list:

```toml
[default]
disabled = ["node", "python"]
```

You can define custom rules with detection patterns and actions:

```toml
[[rules]]
id = "my-custom-rule"
name = "My Custom Rule"
detection = "Makefile"
actions = [
  { remove = "build" },
  { remove = "dist" },
]
```

Detection patterns can use glob syntax and can be combined with logic operators:

```toml
detection = "Cargo.toml"
detection = { any = ["package.json", "yarn.lock"] }
detection = { all = ["Dockerfile", "docker-compose.yaml"] }
detection = { not = "*.lock" }
```

Actions can either remove files/directories or run commands:

```toml
actions = [
  { remove = "build" },
  { remove = "**/cache" },
  { command = "make clean" },
]
```

To customize a built-in rule, define a rule with the same `id`:

```toml
[[rules]]
id = "cargo"
name = "Cargo (custom)"
detection = "Cargo.toml"
actions = [
  { remove = "target" },
  { command = "cargo clean" },
]
```

## Prior Art

This project was inspired by [kondo](https://github.com/tbillington/kondo), a
similar tool for cleaning project directories. **swab** aims to provide more
flexibility through its configurable rule system and support for custom actions.
