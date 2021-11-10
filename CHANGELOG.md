# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog], and this project adheres to
[Semantic Versioning].

Types of changes:
* `Added` for new features.
* `Changed` for changes in existing functionality.
* `Deprecated` for soon-to-be removed features.
* `Removed` for now removed features.
* `Fixed` for any bug fixes.
* `Security` in case of vulnerabilities.

## [0.2.0] - 2021-11-10

### Added

* More information about the enabled actions during startup (as `debug`
  information, requiring the `--verbose` flag). (\#25)
* Add support for 4-finger swipe, configurable via the
  `--four-finger-swipe-{direction}-4` family of arguments. (\#32)
* Settings can now be read from a configuration file using the `--config-file`
  optional argument. If not specified, a `lillinput.toml` file in default
  locations (`/etc`, `$XDG_CONFIG_HOME/lillinput`, `$CWD`) will be used
  instead  (\#54).

### Changed

* The output of each `i3` action and `command` action is now inspected and a
  warning is emitted in case of an error (instead of panicking if they result
  in a failure). (\#46, \#47)
* The command line arguments for specifying swipe actions have been renamed to
  the form `--{number}-finger-swipe-{direction}`, for consistency with the
  configuration file. (\#65)

### Fixed

* Fix finger count for a swipe gesture not being taken into account for
  determining the final event being emitted. (\#31)


## [0.1.0] - 2021-08-01

### Added

* Initial release.

[UNRELEASED]: https://github.com/diego-plan9/lillinput/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/diego-plan9/lillinput/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/diego-plan9/lillinput/releases/tag/v0.1.0

[Keep a Changelog]: https://keepachangelog.com/en/1.0.0/
[Semantic Versioning]: https://semver.org/spec/v2.0.0.html
