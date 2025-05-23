# `type`

Manage package types.

## What is a package type?

A package type decides how `gpm` install and update packages.

For example, a configuration file for a package type could look like this:

```toml
[shell]
pwsh = ["-nop"]  # default arg for shell

[types.gh]
ext = "ps1"      # file extension for script file
shell = "pwsh"   # shell to use
```

This executes `pwsh -nop gh.ps1 [ARGS]...` when executing `gpm repo <NAME> add <PACKAGE> gh [ARGS]...`.

## Writing a script for a package type

As mentioned above, a package type is a script file that is executed by `gpm`.

Here is how a command executed by `gpm`:

```shell
<SHELL> [SHELL_ARGS]... <TYPE> "-n" <NAME> ["-d" <CWD>] ["-t" <TAG>] [ARGS]...
```

It should look like this in practice:

```shell
cd "/home/user/.gpm/repositories/exe" && "pwsh" "-nop" "/home/user/.gpm/scripts/zip_exe.ps1" "-n" "fzf" "-t" "0.55.0" "junegunn/fzf" "linux_amd64"
```

Hence the script must be able to process these arguments

```
-n <NAME>
[-d <CWD>]
[-t <TAG>]
[ARGS]...
```

With the following rules:

- The script must able to process arguments described below:
  - `-n <PACKAGE>`: The name of the package.
  - `[-d <CWD>]`: If `--cwd` is passed, the current working directory will be passed to the script.
  - `[-t <TAG>]`: If the script returns a string in `stdout`, it will be saved and passed to the script on the next run.
  - `[ARGS]...`: Additional positional arguments passed when adding the package
- The script must return a tag or an empty string (nothing) in `stdout`.
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
