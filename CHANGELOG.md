# Changelog

## [0.1.3](https://github.com/terror/swab/releases/tag/0.1.1) - 2026-01-03

### Added

- Default to current directory when no directories are specified (#25)
- Include root directories in project scan (#24)
- Add `rules` subcommand (#23)

### Misc

- Bump clap from 4.5.53 to 4.5.54 (#20)
- Add dependabot workflow (#19)
- Remove explicit `dead_code` attribute from `Test` (#18)
- Refactor `Arguments::run` to use iterator combinators (#16)

## [0.1.2](https://github.com/terror/swab/releases/tag/0.1.1) - 2026-01-02

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
