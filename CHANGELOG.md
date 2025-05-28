# Changelog

## Unreleased

### Changed

- Optimized for binary size.

## [0.6.1]

### Changed

- Bumped dependencies.
- Removed `once_cell`.

## [0.6.0]

### Changed

- Changed argument name passed to the script (**Breaking Change**)
  - `-name` -> `-n`
  - `-cwd` -> `-d`
  - `-etag` -> `-t`
- Changed field name from `etag` to `tag` in `version.toml` (**Breaking Change**)
- Changed `remove-etag` command to `remove-tag` (**Breaking Change**)

## [0.5.1]

### Changed

- Sort output of `list` command.
- Bumped dependencies.

## [0.5.0]

### Added

- Added `--cwd` option to `add` command to pass and store working directory.

### Changed

- Bumped dependencies.

## [0.4.0]

### Removed

- Removed custom shell escape on windows, original code moved to `win_shell_escape` branch.

## [0.3.0]

### Added

- Added `--registry` option to `remove` command to remove the registry only.
- Added a prompt to remove the registry when the target removal fails.

### Fixed

- Fixed relative path in `--path` option.

## [0.2.0]

### Added

- Added remove-etag command under repo
- Added message on add, clone, remove.

### Changed

- Sort the toml file before writing.

### Fixed

- Fixed default shell config for non-windows.

## [0.1.0]

- initial release
