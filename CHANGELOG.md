# Changelog

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
