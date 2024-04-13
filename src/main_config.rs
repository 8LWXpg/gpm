//! Handling main configuration file at GPM_CONFIG.

use crate::repository_config;
use crate::{error, GPM_CONFIG, REPO_CONFIG, REPO_PATH};

use anyhow::{anyhow, Result};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::io::Write;
use std::path::Path;
use std::{fmt, fs};
use tabwriter::TabWriter;

// Separate from the Config struct to allow more flexibility in the future.
#[derive(Debug, Deserialize, Serialize)]
struct TomlConfig {
    /// Key: repository name, Value: repository properties
    repositories: HashMap<String, TomlRepositoryProp>,
}

impl TomlConfig {
    fn into_config(self) -> Config {
        Config {
            repositories: self
                .repositories
                .into_iter()
                .map(|(name, ns)| {
                    (
                        name,
                        RepositoryProp::new(Path::new(&*ns.path))
                            .expect("failed to create repository"),
                    )
                })
                .collect(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct TomlRepositoryProp {
    /// Key: repository name, Value: repository properties
    path: Box<str>,
}

/// GPM configuration.
pub struct Config {
    pub repositories: HashMap<String, RepositoryProp>,
}

impl Config {
    fn new() -> Self {
        Self {
            repositories: HashMap::new(),
        }
    }

    /// Load the configuration, or calls `new()` if it doesn't exist.
    pub fn load() -> Result<Self> {
        let path = GPM_CONFIG.as_path();
        if !path.exists() {
            Ok(Self::new())
        } else {
            toml::from_str::<TomlConfig>(&fs::read_to_string(path)?)
                .map(|c| c.into_config())
                .map_err(Into::into)
        }
    }

    /// Save the configuration.
    pub fn save(self) -> Result<()> {
        fs::write(
            GPM_CONFIG.as_path(),
            toml::to_string(&self.into_toml_config())?,
        )
        .map_err(Into::into)
    }

    fn into_toml_config(self) -> TomlConfig {
        TomlConfig {
            repositories: self
                .repositories
                .into_iter()
                .map(|(name, ns)| {
                    (
                        name,
                        TomlRepositoryProp {
                            path: ns.path.to_string_lossy().into(),
                        },
                    )
                })
                .collect(),
        }
    }

    /// Add a repository to the configuration.
    pub fn add(&mut self, name: String, path: &Path) -> Result<()> {
        if let Entry::Vacant(e) = self.repositories.entry(name.clone()) {
            e.insert(RepositoryProp::new(path)?);
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
            match self.repositories.remove(&name) {
                Some(ns) => ns.remove().unwrap_or_else(|e| {
                    error!("failed to remove package '{}' {}", name.bright_yellow(), e)
                }),
                None => error!("repository '{}' does not exist", name.bright_yellow()),
            }
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut tw = TabWriter::new(vec![]);
        writeln!(&mut tw, "{}", "Repositories:".bright_green()).unwrap();
        for (name, ns) in &self.repositories {
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

/// Property of a repository in the GPM configuration.
pub struct RepositoryProp {
    /// Full path to the repository directory
    path: Box<Path>,
}

impl RepositoryProp {
    fn new(path: &Path) -> Result<Self> {
        fs::create_dir_all(path)?;
        let cfg_path = path.join(REPO_CONFIG);
        repository_config::Config::new().save(&cfg_path)?;
        Ok(Self {
            path: REPO_PATH.join(path).into_boxed_path(),
        })
    }

    fn remove(&self) -> Result<()> {
        fs::remove_dir_all(&self.path)?;
        Ok(())
    }
}
