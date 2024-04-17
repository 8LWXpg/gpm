# `repo`

Manage packages in a repository.

## Commands

```
Usage: gpm repo <NAME> <COMMAND>

Commands:
  add     Add a package to the repository [aliases: a]
  remove  Remove packages in the repository [aliases: r]
  update  Update packages in the repository [aliases: u]
  clone   Clone packages in the repository to the current directory [aliases: c]
  list    List all packages in the repository [aliases: l]
  help    Print this message or the help of the given subcommand(s)

Arguments:
  <NAME>  Repository name

Options:
  -h, --help  Print help
```

### `add`

Add a package to the repository, doc for package types [here](./type.md).

```
Usage: gpm repo <NAME> add <NAME> <TYPE> [ARGS]...

Arguments:
  <NAME>     Package name
  <TYPE>     Package type
  [ARGS]...  Args get passed to the script

Options:
  -h, --help  Print help
```

> [!IMPORTANT]
> Package name must be the same as file/folder name to work properly.

### `remove`

Remove packages in the repository, space separated.

```
Usage: gpm repo <NAME> remove [NAME]...

Arguments:
  [NAME]...  The name of the package

Options:
  -h, --help  Print help
```

### `update`

Update packages in the repository, space separated.

```
Usage: gpm repo <NAME> update [OPTIONS] [NAME]...

Arguments:
  [NAME]...  Package name

Options:
  -a, --all   Update all
  -h, --help  Print help
```

### `clone`

Clone packages in the repository to the current directory, space separated.

```
Usage: gpm repo <NAME> clone [NAME]...

Arguments:
  [NAME]...  Package name

Options:
  -h, --help  Print help
```

### `list`

List all packages in the repository.
