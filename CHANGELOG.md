# Changelog

## [0.1.3](https://github.com/terror/swab/releases/tag/0.1.3) - 2026-01-03

### Added

- Default to current directory when no directories are specified ([#25](https://github.com/terror/swab/pull/25) by [terror](https://github.com/terror))
- Include root directories in project scan ([#24](https://github.com/terror/swab/pull/24) by [terror](https://github.com/terror))
- Add `rules` subcommand ([#23](https://github.com/terror/swab/pull/23) by [terror](https://github.com/terror))

### Misc

- Bump clap from 4.5.53 to 4.5.54 ([#20](https://github.com/terror/swab/pull/20) by [app/dependabot](https://github.com/app/dependabot))
- Add dependabot workflow ([#19](https://github.com/terror/swab/pull/19) by [terror](https://github.com/terror))
- Remove explicit `dead_code` attribute from `Test` ([#18](https://github.com/terror/swab/pull/18) by [terror](https://github.com/terror))
- Refactor `Arguments::run` to use iterator combinators ([#16](https://github.com/terror/swab/pull/16) by [terror](https://github.com/terror))

## [0.1.2](https://github.com/terror/swab/releases/tag/0.1.2) - 2026-01-02

### Added

- Accept `default` as alias for `default_rules` in config ([#13](https://github.com/terror/swab/pull/13) by [terror](https://github.com/terror))

### Misc

- Scaffold integration test suite ([#14](https://github.com/terror/swab/pull/14) by [terror](https://github.com/terror))
- Auto-register rules via `inventory` crate ([#12](https://github.com/terror/swab/pull/12) by [terror](https://github.com/terror))
- Consolidate `Context` implementation blocks ([#11](https://github.com/terror/swab/pull/11) by [terror](https://github.com/terror))

## [0.1.1](https://github.com/terror/swab/releases/tag/0.1.1) - 2025-12-26

### Added

- Make `--interactive` and `--quiet` arguments mutually exclusive ([#6](https://github.com/terror/swab/pull/6) by [terror](https://github.com/terror))

### Fixed

- Honor `--follow-symlinks` across discovery, size calculation, and removal ([#7](https://github.com/terror/swab/pull/7) by [terror](https://github.com/terror))
- Deduplicate per-project counts and bytes across rules ([#5](https://github.com/terror/swab/pull/5) by [terror](https://github.com/terror))
- Run command actions through the system shell ([#4](https://github.com/terror/swab/pull/4) by [terror](https://github.com/terror))

### Misc

- Validate custom remove globs at config load ([#9](https://github.com/terror/swab/pull/9) by [terror](https://github.com/terror))
- Make remove actions idempotent to avoid missing-path failures ([#8](https://github.com/terror/swab/pull/8) by [terror](https://github.com/terror))
- Validate detection glob patterns during config load ([#3](https://github.com/terror/swab/pull/3) by [terror](https://github.com/terror))
- Fix repository name in changelog generator binary ([#2](https://github.com/terror/swab/pull/2) by [terror](https://github.com/terror))

## [0.1.0](https://github.com/terror/swab/releases/tag/0.1.0) - 2025-12-26

Initial release 🎉
