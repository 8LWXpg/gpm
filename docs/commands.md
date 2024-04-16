
# `gpm`

## Commands

```
Usage: gpm <COMMAND>

Commands:
  init      Initialize the package manager, creating the necessary directories [aliases: i]
  add       Add a new repository [aliases: a]
  remove    Remove repositories [aliases: r]
  list      List all repositories [aliases: l]
  repo      Manage packages in a repository
  type      Manage package types [aliases: t]
  generate  Generate shell completion scripts
  help      Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

### `init`

create the necessary directories

- `~/.gpm/`
- `~/.gpm/repositories/`
- `~/.gpm/scripts/`

### `add`

Add a new repository, default path is `~/.gpm/repositories/<NAME>`

```
Usage: gpm add [OPTIONS] <NAME>

Arguments:
  <NAME>  Repository name

Options:
  -p, --path <PATH>  Repository path
  -h, --help         Print help
```

### `remove`

Remove repositories

```
Usage: gpm remove [NAME]...

Arguments:
  [NAME]...  Repository name

Options:
  -h, --help  Print help
```

### `list`

List all repositories

### `repo`

Manage packages in a repository, detailed documentation [here](./repo.md)

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

### `type`

Manage package types

```
Usage: gpm type <COMMAND>

Commands:
  add     Add a new package type [aliases: a]
  remove  Remove package types [aliases: r]
  list    List all package types [aliases: l]
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

### `generate`

Generate shell completion scripts

```
Usage: gpm generate <SHELL>

Arguments:
  <SHELL>  The shell to generate the completion script for [possible values: bash, elvish, fish, powershell, zsh]

Options:
  -h, --help  Print help
```
