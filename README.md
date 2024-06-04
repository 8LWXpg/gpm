# GPM

So you want to make your own package manager?

For something that feels like overkill to create standalone package manager, but is just too lazy to manually check for updates and download.

## Installation

```
cargo install --git https://github.com/8LWXpg/gpm.git
```

## Quick Start

### Initialize the package manager

```bash
gpm init
```

### Add a new package type

```bash
gpm type add <NAME> <EXT> <SHELL>
```

Change your shell config at `~/.gpm/types.toml`.

> [!NOTE]
> `EXT` is the file extension of the script file.

### Edit the script

Script file is created at `~/.gpm/scripts/<NAME>.<EXT>`, see [here](./docs/type.md) for more information.

### Add a new repository

```bash
gpm add <NAME> 
```

For more information see [here](./docs/repo.md).

### Add a package to the repository

```bash
gpm repo <NAME> add <NAME> <TYPE> [ARGS]...
```

> [!IMPORTANT]
> Package name must be the same as file/folder name in order to work properly.

### Make an alias for the repo

```bash
alias <NAME>='gpm repo <NAME>'
```

## Features

### Download third party cargo subcommand instead of compile locally

1. Add a new repository `gpm add cargo --path ~/.cargo/bin`
2. Download the subcommand `gpm repo cargo add <NAME> <TYPE> [ARGS]...`

### Port packages

1. Remove ETag field under `<repo>/version.toml` with `gpm repo <repo> remove-etag`
2. Add a new repository.
3. Copy the `<repo>/version.toml` to the new repository.
4. Update all packages with `gpm repo <repo> update -a`

## Documentation

- [Commands](./docs/commands.md)
- [Repo](./docs/repo.md)
- [Type](./docs/type.md)
- [Script Example](./docs/script.md)

## Windows

There's no standard way to pass arguments to executables in Windows, each executable has its own parsing logic. So, if you have issues with passing arguments to certain executables, please add a fix to the [escape_win.rs](./src/escape_win.rs) file.
