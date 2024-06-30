# `type`

Manage package types.

## What is a package type?

A package type is a script file that is executed by `gpm` to install, update.

For example, a configuration file for a package type could look like this:

```toml
[shell]
pwsh = ["-nop"]

[types.gh]
ext = "ps1"
shell = "pwsh"
```

If you execute `gpm repo <NAME> add <PACKAGE> gh [ARGS]...`, `gpm` will execute `pwsh -c & gh.ps1` with arguments.

## Writing a script for a package type

As mentioned above, a package type is a script file that is executed by `gpm`. Hence, it is important to follow the following rules:

- The script must able to receive arguments described below:
  - `-name <PACKAGE>`: The name of the package.
  - `-dest <PACKAGE_PATH>`: The path to the package.
  - `[-etag <ETAG>]`: If the script returns an etag in `stdout`, it will be saved and passed to the script on the next run.
  - `[-cwd <CWD>]`: If `--cwd` is passed, the current working directory will be passed to the script.
  - `[ARGS]...`: Additional arguments passed when adding the package
- The script must return an etag or an empty string in `stdout`.
- The resulted file/folder must be the same name as the package name. For example, if the package name is `test`, the resulted file/folder must be `test` at repository root.

Here is an example of a command that executed by `gpm`:

```shell
cd "/home/user/.gpm/repositories/exe" && "pwsh" "-nop" "/home/user/.gpm/scripts/zip_exe.ps1" "-name" "fzf" "-etag" "\"0x8DC862A0850D3BA\"" "junegunn/fzf" "linux_amd64"
```

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
Usage: gpm type add <NAME> <EXT> <SHELL>

Arguments:
  <NAME>  Package type
  <EXT>   Script file extension
  <SHELL>  Shell to use

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
