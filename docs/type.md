# `type`

Manage package types.

## What is a package type?

A package type is a script file that is executed by `gpm` to install, update.

For example, a configuration file for a package type could look like this:

```toml
shell = "pwsh"
args = ["-c"]

[types.gh]
ext = "ps1"
```

If you execute `gpm repo <NAME> add <PACKAGE> gh [ARGS]...`, `gpm` will execute `pwsh -c gh.ps1` with following arguments:

- `-name <PACKAGE>`: The name of the package.
- `-dest <PACKAGE_PATH>`: The path to the package.
- `[-etag <ETAG>]`: If the script returns an etag in `stdout`, it will be saved and passed to the script on the next run.
- `[ARGS]...`: Additional arguments passed when adding the package

## Writing a script for a package type

As mentioned above, a package type is a script file that is executed by `gpm`. Hence, it is important to follow the following rules:

- The script must able to receive arguments described above.
- The script must return an etag in `stdout` if it is available, or an empty string if it is not.
- The resulted file/folder must be the same name as the package name. For example, if the package name is `test`, the resulted file/folder must be `test` at repository root.

### Example

Check [script.md](./script.md)

## Commands

```
Manage package types

Usage: gpm type <COMMAND>

Commands:
  add     Add a new package type [aliases: a]
  remove  Remove package types [aliases: r]
  list    List all package types [aliases: l]
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

### `add`

Add a new package type.

```
Usage: gpm type add <NAME> <EXT>

Arguments:
  <NAME>  Package type
  <EXT>   Script file extension

Options:
  -h, --help  Print help
```

### `remove`

Remove package types, space separated.

```
Usage: gpm type remove [NAME]...

Arguments:
  [NAME]...  Type name

Options:
  -h, --help  Print help
```

### `list`

List all package types.
