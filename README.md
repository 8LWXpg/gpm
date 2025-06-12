# GPM

So you want to make your own package manager?

For something that feels like overkill to create standalone package manager, but is just too lazy to manually check for updates and download.

## Introduction

`gpm` is a lightweight command runner and configuration manager that centralizes the information required to execute scripts across custom-defined package types. It simplifies managing reusable scripts, repositories, and execution environments in a structured and extensible way.

By default, `gpm` organizes your environment with the following directory layout:

```
.gpm
├── config.toml     - Stores global repository configuration
├── types.toml      - Defines available package types and their shell arguments
├── repositories    - Managed repositories and package data
│   └── dir
│       ├── .editorconfig
│       ├── FUNDING.yml
│       ├── LICENSE
│       ├── Localizing.md
│       ├── template.typ
│       ├── version.toml    - Contains metadata and script arguments
│       └── ISSUE_TEMPLATE
└── scripts         - Script for all types defined in types.toml
    ├── dir.ps1
    ├── exe.ps1
    ├── file.ps1
    ├── README.md
    ├── zip_exe.ps1
    └── lib
        └── gh_dl.ps1
```

### Core Concept

- Repository - A folder that stores packages
- Package type - Defines how the package is handled handled by the script with the same name.

## Installation

### Download

Download from [latest release](https://github.com/8LWXpg/gpm/releases/latest)

### Install with `cargo-binstall`

```shell
cargo binstall --git https://github.com/8LWXpg/gpm gpm
```

### Compile from source

```shell
cargo install --git https://github.com/8LWXpg/gpm.git
```

## Quick Start

### Initialize the package manager

```shell
gpm init
```

### Add a new package type

```shell
gpm type add <NAME> <EXT> <SHELL>
```

Change your shell config at `~/.gpm/types.toml`.

> [!NOTE]
> `EXT` is the file extension of the script file.

### Edit the script

Script file is created at `~/.gpm/scripts/<NAME>.<EXT>`, see [here](./docs/type.md) for more information.

### Add a new repository

```shell
gpm add <NAME> 
```

For more information see [here](./docs/repo.md).

### Add a package to the repository

```shell
gpm repo <NAME> add <NAME> <TYPE> [ARGS]...
```

> [!IMPORTANT]
> Package name must be the same as resulted package file/folder name in order to work properly.

### Make an alias for the repo

```shell
alias <NAME>='gpm repo <NAME>'
```

## Features

### Download third party cargo subcommand instead of compile locally

1. Add a new repository `gpm add cargo --path ~/.cargo/bin`
2. Download the subcommand `gpm repo cargo add <NAME> <TYPE> [ARGS]...`

### Port packages

1. Remove Tag field under `<repo>/version.toml` with `gpm repo <repo> remove-tag`
2. Add a new repository.
3. Copy the `<repo>/version.toml` to the new repository.
4. Update all packages with `gpm repo <repo> update -a`

## Documentation

- [Commands](./docs/commands.md)
- [Repo](./docs/repo.md)
- [Type](./docs/type.md)
- [Script Example](./docs/script.md)

## Windows

There's no standard way to pass arguments to executables in Windows, each executable has its own parsing logic. So, if you have issues with passing arguments to certain executables, please open an issue.
