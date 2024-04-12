mod main_config;
// mod repository;
mod repository_config;
mod type_config;

use main_config::Config;
use type_config::ReturnType;

use clap::builder::styling;
use clap::{Args, Parser, Subcommand};
use colored::{ColoredString, Colorize};
use once_cell::sync::Lazy;
use std::path::{Path, PathBuf};
use std::process;
use type_config::TypeConfig;

static GPM_HOME: Lazy<PathBuf> = Lazy::new(|| dirs::home_dir().unwrap().join(".gpm"));
static GPM_CONFIG: Lazy<PathBuf> = Lazy::new(|| GPM_HOME.join("config.toml"));
/// config for each repository
static NAMESPACES_CONFIG: &str = "version.toml";
static NAMESPACES_PATH: Lazy<PathBuf> = Lazy::new(|| GPM_HOME.join("repositories"));
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
    #[clap(visible_alias = "n")]
    #[command(arg_required_else_help = true)]
    Repository(Repository),

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
        TopCommand::Add { name, path } => match Config::load() {
            Ok(mut gpm_cfg) => {
                gpm_cfg
                    .add(name, &path.unwrap_or(NAMESPACES_PATH.join(&name)))
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
        TopCommand::Repository(ns) => {
            // TODO: Implement repository command
        }
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
