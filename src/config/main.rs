//! Handling main configuration file at GPM_CONFIG.

use super::repository;
use super::util::{prompt, sort_keys};
use crate::{add, error, remove, GPM_CONFIG, REPO_CONFIG, REPO_PATH};

use anyhow::{anyhow, Result};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::collections::{hash_map::Entry, BTreeMap, HashMap};
use std::io::Write;
use std::path::Path;
use std::{fmt, fs};
use tabwriter::TabWriter;

// Separate from the Config struct to allow more flexibility in the future.
#[derive(Debug, Deserialize, Serialize)]
struct TomlConfig {
	/// Key: repository name, Value: repository properties
	#[serde(serialize_with = "sort_keys")]
	repositories: HashMap<String, TomlRepositoryProp>,
}

impl From<Config> for TomlConfig {
	fn from(main_config: Config) -> Self {
		Self {
			repositories: main_config
				.repositories
				.into_iter()
				.map(|(name, repo_prop)| (name, repo_prop.into()))
				.collect(),
		}
	}
}

#[derive(Debug, Deserialize, Serialize)]
struct TomlRepositoryProp {
	/// Key: repository name, Value: repository properties
	path: Box<str>,
}

impl From<RepositoryProp> for TomlRepositoryProp {
	fn from(repo: RepositoryProp) -> Self {
		Self {
			path: repo.path.to_string_lossy().into(),
		}
	}
}

/// GPM configuration.
pub struct Config {
	repositories: HashMap<String, RepositoryProp>,
}

impl Config {
	fn new() -> Self {
		Self {
			repositories: HashMap::new(),
		}
	}

	/// Load the configuration, or calls `new()` if it doesn't exist.
	pub fn load() -> Result<Self> {
		if !GPM_CONFIG.exists() {
			Ok(Self::new())
		} else {
			toml::from_str::<TomlConfig>(&fs::read_to_string(&*GPM_CONFIG)?)
				.map(Into::into)
				.map_err(Into::into)
		}
	}

	/// Save the configuration.
	pub fn save(self) -> Result<()> {
		fs::write(&*GPM_CONFIG, toml::to_string(&TomlConfig::from(self))?).map_err(Into::into)
	}

	/// Add a repository to the configuration.
	///
	/// `path` is the absolute path.
	pub fn add(&mut self, name: String, path: &Path) -> Result<()> {
		if let Entry::Vacant(e) = self.repositories.entry(name.clone()) {
			e.insert(RepositoryProp::new(path)?);
			add!("{}\t{}", name.bright_cyan(), path.to_str().unwrap());
			Ok(())
		} else {
			Err(anyhow!(
				"repository '{}' already exists",
				name.bright_yellow()
			))
		}
	}

	/// Remove repositories from the configuration.
	pub fn remove(&mut self, names: Vec<String>) {
		for name in names {
			match self.repositories.get(&name) {
				Some(repo) => match repo.remove() {
					Ok(()) => remove!(
						"{}\t{}",
						name.bright_cyan(),
						self.repositories
							.remove(&name)
							.unwrap()
							.path
							.to_str()
							.unwrap()
					),
					Err(e) => {
						error!("failed to remove package '{}' {}", name.bright_yellow(), e);
						match prompt("Remove from registry?") {
							Ok(true) => remove!(
								"{}\t{}",
								name.bright_cyan(),
								self.repositories
									.remove(&name)
									.unwrap()
									.path
									.to_str()
									.unwrap()
							),
							Ok(false) => {}
							Err(e) => error!("{}", e),
						}
					}
				},
				None => error!("repository '{}' does not exist", name.bright_yellow()),
			}
		}
	}

	/// Remove registry entries.
	pub fn remove_registry(&mut self, names: Vec<String>) {
		for name in names {
			match self.repositories.remove(&name) {
				Some(_) => remove!("{}", name.bright_cyan()),
				None => error!("repository '{}' does not exist", name.bright_yellow()),
			}
		}
	}
}

impl From<TomlConfig> for Config {
	fn from(main_config: TomlConfig) -> Self {
		Self {
			repositories: main_config
				.repositories
				.into_iter()
				.map(|(name, repo)| (name, repo.into()))
				.collect(),
		}
	}
}

impl fmt::Display for Config {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let mut tw = TabWriter::new(vec![]);
		writeln!(&mut tw, "{}", "Repositories:".bright_green()).unwrap();
		let btree_map: BTreeMap<_, _> = self.repositories.iter().collect();
		for (name, ns) in &btree_map {
			writeln!(
				&mut tw,
				"  {}\t{}",
				name.bright_cyan(),
				ns.path.to_str().unwrap()
			)
			.unwrap();
		}
		tw.flush().unwrap();
		let result = String::from_utf8(tw.into_inner().unwrap()).unwrap();
		write!(f, "{}", result)
	}
}

impl Default for Config {
	fn default() -> Self {
		Self::new()
	}
}

/// Property of a repository in the GPM configuration.
struct RepositoryProp {
	/// Full path to the repository directory
	path: Box<Path>,
}

impl RepositoryProp {
	/// Create a new repository property, creating the repository directory and configuration file.
	fn new(path: &Path) -> Result<Self> {
		fs::create_dir_all(path)?;
		let cfg_path = path.join(REPO_CONFIG);
		repository::RepoConfig::new(path).save(&cfg_path)?;
		Ok(Self {
			path: REPO_PATH.join(path).into_boxed_path(),
		})
	}

	fn remove(&self) -> Result<()> {
		fs::remove_dir_all(&self.path)?;
		Ok(())
	}
}

impl From<TomlRepositoryProp> for RepositoryProp {
	fn from(repo: TomlRepositoryProp) -> Self {
		Self {
			path: Path::new(&*repo.path).into(),
		}
	}
}

pub fn get_repo_path(name: &str) -> Box<Path> {
	Config::load()
		.unwrap_or_default()
		.repositories
		.get(name)
		.unwrap()
		.path
		.clone()
}
