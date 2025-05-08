mod config;

use crate::config::main::Config;
use crate::config::r#type::TypeConfig;
use crate::config::repository::RepoConfig;

use clap::CommandFactory;
use clap::{builder::styling, Args, Parser, Subcommand};
use clap_complete::Shell;
use colored::Colorize;
use path_clean::PathClean;
use std::path::PathBuf;
use std::sync::LazyLock;
use std::{env, fs, io, process};

static GPM_HOME: LazyLock<PathBuf> = LazyLock::new(|| dirs::home_dir().unwrap().join(".gpm"));
static GPM_CONFIG: LazyLock<PathBuf> = LazyLock::new(|| GPM_HOME.join("config.toml"));
/// config for each repository
static REPO_CONFIG: &str = "version.toml";
static REPO_PATH: LazyLock<PathBuf> = LazyLock::new(|| GPM_HOME.join("repositories"));
static SCRIPT_ROOT: LazyLock<PathBuf> = LazyLock::new(|| GPM_HOME.join("scripts"));
static TYPES_CONFIG: LazyLock<PathBuf> = LazyLock::new(|| GPM_HOME.join("types.toml"));

// region: clap macros
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

		/// Remove registry only
		#[clap(short, long)]
		registry: bool,
	},

	/// List all repositories
	#[clap(visible_alias = "l")]
	List,

	/// Manage packages in a repository
	#[command(arg_required_else_help = true)]
	Repo(Repository),

	/// Manage package types
	#[clap(subcommand, visible_alias = "t")]
	#[command(arg_required_else_help = true)]
	Type(TypeCommand),

	/// Generate shell completion scripts
	Generate {
		/// The shell to generate the completion script for
		shell: Shell,
	},
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

		/// Passing cwd to the script
		#[clap(short, long)]
		cwd: bool,
	},

	/// Remove packages in the repository
	#[clap(visible_alias = "r")]
	#[command(arg_required_else_help = true)]
	Remove {
		/// Package names
		#[clap(num_args = 1..)]
		name: Vec<String>,

		/// Remove registry only
		#[clap(short, long)]
		registry: bool,
	},

	/// Remove tag field for all packages in the repository
	RemoveTag,

	/// Update packages in the repository
	#[clap(visible_alias = "u")]
	#[command(arg_required_else_help = true)]
	Update {
		/// Package names
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
		/// Package names
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

		/// Shell to use
		shell: String,
	},

	/// Remove package types
	#[clap(visible_alias = "r")]
	#[command(arg_required_else_help = true)]
	Remove {
		/// Type names
		#[clap(num_args = 1..)]
		name: Vec<String>,

		/// Remove registry only
		#[clap(short, long)]
		registry: bool,
	},

	/// List all package types
	#[clap(visible_alias = "l")]
	List,
}
// endregion

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
					.add(
						name.clone(),
						&match path {
							Some(p) => env::current_dir().unwrap().join(p).clean(),
							None => REPO_PATH.join(&name),
						},
					)
					.unwrap_or_else(error_exit0);
				gpm_cfg.save().unwrap_or_else(error_exit0);
			}
			Err(e) => error_exit0(e),
		},
		TopCommand::Remove { name, registry } => match Config::load() {
			Ok(mut gpm_cfg) => {
				if registry {
					gpm_cfg.remove_registry(name);
				} else {
					gpm_cfg.remove(name);
				}
				gpm_cfg.save().unwrap_or_else(error_exit0);
			}
			Err(e) => error_exit0(e),
		},
		TopCommand::List => match Config::load() {
			Ok(gpm_cfg) => print!("{}", gpm_cfg),
			Err(e) => error_exit0(e),
		},
		TopCommand::Repo(repo) => {
			let repo_cfg_path = &config::main::get_repo_path(&repo.name).join(REPO_CONFIG);
			match RepoConfig::load(repo_cfg_path) {
				Ok(mut repo_cfg) => {
					match repo.command {
						RepositoryCommand::Add {
							name,
							r#type,
							args,
							cwd,
						} => repo_cfg
							.add(name, r#type, args.into_boxed_slice(), cwd)
							.unwrap_or_else(error_exit0),
						RepositoryCommand::Remove { name, registry } => {
							if registry {
								repo_cfg.remove_registry(name);
							} else {
								repo_cfg.remove(name)
							}
						}
						RepositoryCommand::RemoveTag => repo_cfg.remove_tag(),
						RepositoryCommand::Update { name, all } => {
							if all {
								repo_cfg.update_all();
							} else {
								repo_cfg.update(name);
							}
						}
						RepositoryCommand::Clone { name } => repo_cfg.copy(name),
						RepositoryCommand::List => {
							print!("{}", repo_cfg);
							return;
						}
					}
					repo_cfg.save(repo_cfg_path).unwrap_or_else(error_exit0);
				}
				Err(e) => error_exit0(e),
			}
		}
		TopCommand::Type(t) => match t {
			TypeCommand::Add { name, ext, shell } => match TypeConfig::load() {
				Ok(mut type_cfg) => {
					type_cfg.add(name, ext, shell).unwrap_or_else(error_exit0);
					type_cfg.save().unwrap_or_else(error_exit0);
				}
				Err(e) => error_exit0(e),
			},
			TypeCommand::Remove { name, registry } => match TypeConfig::load() {
				Ok(mut type_cfg) => {
					if registry {
						type_cfg.remove_registry(name);
					} else {
						type_cfg.remove(name);
					}
					type_cfg.save().unwrap_or_else(error_exit0);
				}
				Err(e) => error_exit0(e),
			},
			TypeCommand::List => match TypeConfig::load() {
				Ok(type_cfg) => print!("{}", type_cfg),
				Err(e) => error_exit0(e),
			},
		},
		TopCommand::Generate { shell } => {
			clap_complete::generate(shell, &mut App::command(), "gpm", &mut io::stdout())
		}
	}
}
