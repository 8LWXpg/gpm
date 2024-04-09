mod main_config;
mod namespace;
mod package_config;
mod type_config;

use clap::builder::styling;
use clap::{Args, Parser, Subcommand};
use colored::{ColoredString, Colorize};
use once_cell::sync::Lazy;
use std::path::PathBuf;

static GPM_HOME: Lazy<PathBuf> = Lazy::new(|| dirs::home_dir().unwrap().join(".gpm"));
static GPM_CONFIG: Lazy<PathBuf> = Lazy::new(|| GPM_HOME.join("config.toml"));
static NAMESPACES_CONFIG: &str = "version.toml";
static SCRIPT_ROOT: Lazy<PathBuf> = Lazy::new(|| GPM_HOME.join("scripts"));
static ERROR: Lazy<ColoredString> = Lazy::new(|| "error:".bright_red().bold());

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
    /// Add a new namespace
    #[clap(visible_alias = "a")]
    Add {
        /// Namespace name
        name: String,

        /// Namespace path
        #[clap(short, long)]
        path: Option<PathBuf>,
    },

    /// Remove a namespace
    #[clap(visible_alias = "r")]
    Remove {
        /// The name of the namespace
        name: String,
    },

    /// List all namespaces
    #[clap(visible_alias = "l")]
    List,

    /// Manage packages in a namespace
    #[clap(visible_alias = "n")]
    Namespace(Namespace),

    /// Manage package types
    #[clap(subcommand, visible_alias = "t")]
    Type(TypeCommand),
}

#[derive(Debug, Args)]
struct Namespace {
    /// The name of the namespace
    name: String,

    #[clap(subcommand)]
    command: NamespaceCommand,
}

#[derive(Debug, Subcommand)]
enum NamespaceCommand {
    /// Add a package to the namespace
    #[clap(visible_alias = "a")]
    Add {
        /// Package name
        name: String,

        /// Package type
        r#type: String,

        /// Args get passed to the script
        args: Vec<String>,
    },

    /// Remove a package from the namespace
    #[clap(visible_alias = "r")]
    Remove {
        /// The name of the package
        #[clap(num_args = 1..)]
        name: Vec<String>,
    },

    /// Update a package in the namespace
    #[clap(visible_alias = "u")]
    Update {
        /// The name of the package, space separated
        #[clap(num_args = 1..)]
        name: Vec<String>,

        /// switch for update all
        #[clap(short, long)]
        all: bool,
    },

    /// Clone a package in the namespace to the current directory
    #[clap(visible_alias = "c")]
    Clone {
        /// The name of the package, space separated
        #[clap(num_args = 1..)]
        name: Vec<String>,
    },

    /// List all packages in the namespace
    #[clap(visible_alias = "l")]
    List,
}

#[derive(Debug, Subcommand)]
enum TypeCommand {
    /// Add a new package type
    #[clap(visible_alias = "a")]
    Add {
        /// Package type
        name: String,

        /// Package type script
        script: String,
    },

    /// Remove a package type
    #[clap(visible_alias = "r")]
    Remove {
        /// The name of the package type
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

fn main() {
    let args = App::parse();

    match args.command {
        TopCommand::Add { name, path } => {
            // update global config
            let mut gpm_cfg = main_config::load_config()
                .unwrap_or_else(|_| main_config::Config { namespaces: vec![] });
            if gpm_cfg.namespaces.iter().any(|ns| ns.name == name) {
                eprintln!(
                    "{} namespace '{}' already exists",
                    &*ERROR,
                    name.bright_yellow()
                );
                return;
            }

            let dir = path.unwrap_or_else(|| GPM_HOME.join(&name));
            let namespace =
                main_config::NamespaceConfig::new(name, dir.to_str().unwrap().to_string());
            namespace.add().unwrap();
            gpm_cfg.namespaces.push(namespace);
            main_config::save_config(&gpm_cfg).unwrap();
        }
        TopCommand::Remove { name } => match main_config::load_config() {
            Ok(mut gpm_cfg) => match gpm_cfg.namespaces.iter().position(|ns| ns.name == name) {
                Some(index) => {
                    // remove namespace registry and folder
                    let remove = &gpm_cfg.namespaces[index];
                    remove.remove().unwrap();
                    gpm_cfg.namespaces.remove(index);
                    main_config::save_config(&gpm_cfg).unwrap();
                }
                None => eprintln!(
                    "{} namespace '{}' does not exist",
                    &*ERROR,
                    name.bright_yellow()
                ),
            },
            Err(e) => eprintln!("{} {}", &*ERROR, e),
        },
        TopCommand::List => match main_config::load_config() {
            Ok(gpm_cfg) => {
                println!("{}", "Namespaces:".bright_green());
                println!("{}", gpm_cfg.print());
            }
            Err(e) => eprintln!("{} {}", &*ERROR, e),
        },
        TopCommand::Namespace(ns) => match main_config::load_config() {
            Ok(gpm_cfg) => match gpm_cfg.namespaces.iter().find(|n| n.name == ns.name) {
                Some(n) => match namespace::namespace(ns, n) {
                    Ok(_) => {}
                    Err(e) => eprintln!("{} {}", &*ERROR, e),
                },
                None => eprintln!(
                    "{} namespace '{}' does not exist",
                    &*ERROR,
                    ns.name.bright_yellow()
                ),
            },
            Err(e) => eprintln!("{} {}", &*ERROR, e),
        },
        TopCommand::Type(t) => match t {
            TypeCommand::Add { name, script } => {}
            TypeCommand::Remove { name } => {}
            TypeCommand::List => {}
        },
    }
}
