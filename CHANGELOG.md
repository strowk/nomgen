# Change Log

All notable changes to this project will be documented in this file.
The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

<!-- next-header -->

## [Unreleased] - ReleaseDate

### Changed

- corrected "pattern" to "patterns" in README.md

## [v0.1.0] - 2023-09-22

### Added

- Initial release with following features:
- Define file generators using a configuration file in TOML format.
- Automatically generate and modify files based on configured commands.
- Run check to ensure that files that match configured patterns are not modified manually.
- Integrate with Git pre-commit hooks for automated check before committing unless commit is made by nomgen itself.

<!-- next-url -->
[Unreleased]: https://github.com/strowk/probranchinator/compare/v0.1.0...HEAD
[v0.1.0]: https://github.com/strowk/probranchinator/releases/tag/v0.1.0