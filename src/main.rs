mod main_config;
// mod repository;
mod repository_config;
mod type_config;

use main_config::Config;
use type_config::ReturnType;

use clap::{builder::styling, Args, Parser, Subcommand};
use colored::Colorize;
use once_cell::sync::Lazy;
use std::path::PathBuf;
use std::{fs, process};
use type_config::TypeConfig;

static GPM_HOME: Lazy<PathBuf> = Lazy::new(|| dirs::home_dir().unwrap().join(".gpm"));
static GPM_CONFIG: Lazy<PathBuf> = Lazy::new(|| GPM_HOME.join("config.toml"));
/// config for each repository
static REPO_CONFIG: &str = "version.toml";
static REPO_PATH: Lazy<PathBuf> = Lazy::new(|| GPM_HOME.join("repositories"));
static SCRIPT_ROOT: Lazy<PathBuf> = Lazy::new(|| GPM_HOME.join("scripts"));
static TYPES_CONFIG: Lazy<PathBuf> = Lazy::new(|| GPM_HOME.join("types.toml"));

#[derive(Debug, Parser)]
#[command(
    version,
    name = "gpm",
    about = "A fully customizable general purpose package manager",
    styles = get_styles(),
    arg_required_else_help = true,
)]
struct App {
    #[clap(subcommand)]
    command: TopCommand,
}

#[derive(Debug, Subcommand)]
enum TopCommand {
    /// Initialize the package manager, creating the necessary directories
    #[clap(visible_alias = "i")]
    Init,

    /// Add a new repository
    #[clap(visible_alias = "a")]
    #[command(arg_required_else_help = true)]
    Add {
        /// Repository name
        name: String,

        /// Repository path
        #[clap(short, long)]
        path: Option<PathBuf>,
    },

    /// Remove repositories
    #[clap(visible_alias = "r")]
    #[command(arg_required_else_help = true)]
    Remove {
        /// Repository name
        #[clap(num_args = 1..)]
        name: Vec<String>,
    },

    /// List all repositories
    #[clap(visible_alias = "l")]
    List,

    /// Manage packages in a repository
    // #[clap(visible_alias = "r")]
    #[command(arg_required_else_help = true)]
    Repo(Repository),

    /// Manage package types
    #[clap(subcommand, visible_alias = "t")]
    #[command(arg_required_else_help = true)]
    Type(TypeCommand),
}

#[derive(Debug, Args)]
struct Repository {
    /// Repository name
    name: String,

    #[clap(subcommand)]
    command: RepositoryCommand,
}

#[derive(Debug, Subcommand)]
enum RepositoryCommand {
    /// Add a package to the repository
    #[clap(visible_alias = "a")]
    #[command(arg_required_else_help = true)]
    Add {
        /// Package name
        name: String,

        /// Package type
        r#type: String,

        /// Args get passed to the script
        args: Vec<String>,

        /// Args get passed to the script.post
        #[arg(short, long)]
        post_args: Vec<String>,
    },

    /// Remove packages in the repository
    #[clap(visible_alias = "r")]
    #[command(arg_required_else_help = true)]
    Remove {
        /// The name of the package
        #[clap(num_args = 1..)]
        name: Vec<String>,
    },

    /// Update packages in the repository
    #[clap(visible_alias = "u")]
    #[command(arg_required_else_help = true)]
    Update {
        /// Package name
        #[clap(num_args = 1..)]
        name: Vec<String>,

        /// Update all
        #[clap(short, long)]
        all: bool,
    },

    /// Clone packages in the repository to the current directory
    #[clap(visible_alias = "c")]
    #[command(arg_required_else_help = true)]
    Clone {
        /// Package name
        #[clap(num_args = 1..)]
        name: Vec<String>,
    },

    /// List all packages in the repository
    #[clap(visible_alias = "l")]
    List,
}

#[derive(Debug, Subcommand)]
enum TypeCommand {
    /// Add a new package type
    #[clap(visible_alias = "a")]
    #[command(arg_required_else_help = true)]
    Add {
        /// Package type
        name: String,

        /// Script file extension
        ext: String,

        /// Return type
        ret: ReturnType,
    },

    /// Remove package types
    #[clap(visible_alias = "r")]
    #[command(arg_required_else_help = true)]
    Remove {
        /// Type name
        #[clap(num_args = 1..)]
        name: Vec<String>,
    },

    /// List all package types
    #[clap(visible_alias = "l")]
    List,
}

fn get_styles() -> clap::builder::Styles {
    clap::builder::Styles::default()
        .usage(styling::AnsiColor::BrightGreen.on_default())
        .header(styling::AnsiColor::BrightGreen.on_default())
        .literal(styling::AnsiColor::BrightCyan.on_default())
        .invalid(styling::AnsiColor::BrightYellow.on_default())
        .error(styling::AnsiColor::BrightRed.on_default().bold())
        .valid(styling::AnsiColor::BrightGreen.on_default())
        .placeholder(styling::AnsiColor::Cyan.on_default())
}

/// Print an error message to stderr.
#[macro_export]
macro_rules! error {
    ($msg:expr) => {
        eprintln!("{} {}", "error:".bright_red().bold(), $msg)
    };
    ($fmt:expr, $($arg:tt)*) => {
        eprintln!("{} {}", "error:".bright_red().bold(), format!($fmt, $($arg)*))
    };
}

/// Print info message to stdout.
#[macro_export]
macro_rules! info {
    ($msg:expr) => {
        println!("{} {}", "info:".bright_blue(), $msg)
    };
    ($fmt:expr, $($arg:tt)*) => {
        println!("{} {}", "info:".bright_blue(), format!($fmt, $($arg)*))
    };
}

fn error_exit0<T>(msg: T)
where
    T: std::fmt::Display,
{
    error!(msg);
    process::exit(0);
}

fn main() {
    let args = App::parse();

    match args.command {
        TopCommand::Init => {
            if !GPM_HOME.exists() {
                fs::create_dir(&*GPM_HOME).unwrap_or_else(error_exit0);
            }
            if !REPO_PATH.exists() {
                fs::create_dir(&*REPO_PATH).unwrap_or_else(error_exit0);
            }
            if !SCRIPT_ROOT.exists() {
                fs::create_dir(&*SCRIPT_ROOT).unwrap_or_else(error_exit0);
            }
        }
        TopCommand::Add { name, path } => match Config::load() {
            Ok(mut gpm_cfg) => {
                gpm_cfg
                    .add(name.clone(), &path.unwrap_or(REPO_PATH.join(&name)))
                    .unwrap_or_else(error_exit0);
                gpm_cfg.save().unwrap_or_else(error_exit0);
            }
            Err(e) => error_exit0(e),
        },
        TopCommand::Remove { name } => match Config::load() {
            Ok(mut gpm_cfg) => {
                gpm_cfg.remove(name);
                gpm_cfg.save().unwrap_or_else(error_exit0);
            }
            Err(e) => error_exit0(e),
        },
        TopCommand::List => print!("{}", Config::load().unwrap_or_default()),
        TopCommand::Repo(repo) => match repo.command {
            RepositoryCommand::Add {
                name,
                r#type,
                args,
                post_args,
            } => {
                let repo_cfg_path = &REPO_PATH.join(&repo.name).join(REPO_CONFIG);
                match repository_config::Repo::load(repo_cfg_path) {
                    Ok(mut repo_cfg) => {
                        repo_cfg
                            .add(
                                name,
                                r#type,
                                args.into_boxed_slice(),
                                post_args.into_boxed_slice(),
                            )
                            .unwrap_or_else(error_exit0);
                        repo_cfg.save(repo_cfg_path).unwrap_or_else(error_exit0);
                    }
                    Err(e) => error_exit0(e),
                }
            }
            RepositoryCommand::Remove { name } => {
                let repo_cfg_path = &REPO_PATH.join(&repo.name).join(REPO_CONFIG);
                match repository_config::Repo::load(repo_cfg_path) {
                    Ok(mut repo_cfg) => {
                        repo_cfg.remove(name);
                        repo_cfg.save(repo_cfg_path).unwrap_or_else(error_exit0);
                    }
                    Err(e) => error_exit0(e),
                }
            }
            RepositoryCommand::Update { name, all } => {
                let repo_cfg_path = &REPO_PATH.join(&repo.name).join(REPO_CONFIG);
                match repository_config::Repo::load(repo_cfg_path) {
                    Ok(mut repo_cfg) => {
                        if all {
                            repo_cfg.update_all();
                        } else {
                            repo_cfg.update(name);
                            repo_cfg.save(repo_cfg_path).unwrap_or_else(error_exit0);
                        }
                    }
                    Err(e) => error_exit0(e),
                }
            }
            RepositoryCommand::Clone { name } => {
                let repo_cfg_path = &REPO_PATH.join(&repo.name).join(REPO_CONFIG);
                match repository_config::Repo::load(repo_cfg_path) {
                    Ok(repo_cfg) => {
                        repo_cfg.copy(name);
                        repo_cfg.save(repo_cfg_path).unwrap_or_else(error_exit0);
                    }
                    Err(e) => error_exit0(e),
                }
            }
            RepositoryCommand::List => {
                let repo_cfg_path = &REPO_PATH.join(&repo.name).join(REPO_CONFIG);
                match repository_config::Repo::load(repo_cfg_path) {
                    Ok(repo_cfg) => println!("{}", repo_cfg.print()),
                    Err(e) => error_exit0(e),
                }
            }
        },
        TopCommand::Type(t) => match t {
            TypeCommand::Add { name, ext, ret } => match TypeConfig::load() {
                Ok(mut type_cfg) => {
                    type_cfg.add(name, ext, ret).unwrap_or_else(error_exit0);
                    type_cfg.save().unwrap_or_else(error_exit0);
                }
                Err(e) => error_exit0(e),
            },
            TypeCommand::Remove { name } => match TypeConfig::load() {
                Ok(mut type_cfg) => {
                    type_cfg.remove(name);
                    type_cfg.save().unwrap_or_else(error_exit0);
                }
                Err(e) => error_exit0(e),
            },
            TypeCommand::List => print!("{}", TypeConfig::load().unwrap_or_default()),
        },
    }
}
