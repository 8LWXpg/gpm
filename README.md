# GPM

So you want to make your own package manager?

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
gpm type add <NAME> <EXT>
```

Change your shell config at `~/.gpm/types.toml`.

> [!NOTE]
> `EXT` is the file extension of the script file.

### Edit the script

Script file is located at `~/.gpm/scripts/<NAME>.<EXT>`, see [here](./docs/type.md) for more information.

### Add a new repository

```bash
gpm add <NAME> 
```

For more information see [here](./docs/repo.md).

### Add a package to the repository

```bash
gpm repo <NAME> add <NAME> <TYPE> [ARGS]...
```

### Make an alias for the repo

```bash
alias <NAME>='gpm repo <NAME>'
```

## Documentation

- [Commands](./docs/commands.md)
- [Repo](./docs/repo.md)
- [Type](./docs/type.md)
- [Script Example](./docs/script.md)
