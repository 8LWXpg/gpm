use crate::package_config;
use crate::{error, GPM_CONFIG, NAMESPACES_CONFIG, NAMESPACES_PATH};

use anyhow::{anyhow, Result};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use tabwriter::TabWriter;

#[derive(Debug, Deserialize, Serialize)]
struct TomlConfig {
    namespaces: HashMap<String, TomlNamespaceProp>,
}

impl TomlConfig {
    fn into_config(self) -> Config {
        Config {
            namespaces: self
                .namespaces
                .into_iter()
                .map(|(name, ns)| {
                    (
                        name,
                        NamespaceProp::new(PathBuf::from_str(&ns.path).unwrap()),
                    )
                })
                .collect(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct TomlNamespaceProp {
    path: String,
}

/// GPM configuration.
pub struct Config {
    pub namespaces: HashMap<String, NamespaceProp>,
}

impl Config {
    pub fn new() -> Self {
        Self {
            namespaces: HashMap::new(),
        }
    }

    /// Load the configuration.
    pub fn load() -> Result<Self> {
        toml::from_str::<TomlConfig>(&fs::read_to_string(GPM_CONFIG.as_path())?)
            .map(|c| c.into_config())
            .map_err(Into::into)
    }

    /// Save the configuration.
    pub fn save(self) -> Result<()> {
        fs::write(
            GPM_CONFIG.as_path(),
            toml::to_string(&self.into_toml_config())?,
        )
        .map_err(Into::into)
    }

    pub fn print(&self) -> String {
        let mut tw = TabWriter::new(vec![]);
        for (name, ns) in &self.namespaces {
            writeln!(
                &mut tw,
                "  {}\t{}",
                name.bright_cyan(),
                ns.path.to_str().unwrap()
            )
            .unwrap();
        }
        tw.flush().unwrap();
        String::from_utf8(tw.into_inner().unwrap()).unwrap()
    }

    fn into_toml_config(self) -> TomlConfig {
        TomlConfig {
            namespaces: self
                .namespaces
                .into_iter()
                .map(|(name, ns)| {
                    (
                        name,
                        TomlNamespaceProp {
                            path: ns.path.to_string_lossy().to_string(),
                        },
                    )
                })
                .collect(),
        }
    }

    /// Add a namespace to the configuration.
    pub fn add(&mut self, name: String, ns: NamespaceProp) -> Result<()> {
        if let Entry::Vacant(e) = self.namespaces.entry(name) {
            e.insert(ns);
            Ok(())
        } else {
            Err(anyhow!("namespace '{}' already exists", name))
        }
    }

    /// Remove namespaces from the configuration.
    pub fn remove(&mut self, names: Vec<String>) {
        for name in names {
            match self.namespaces.remove(&name) {
                Some(ns) => ns.remove().unwrap_or_else(|e| {
                    error!("failed to remove package '{}' {}", name.bright_yellow(), e)
                }),
                None => error!("namespace '{}' does not exist", name.bright_yellow()),
            }
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

/// Property of a namespace in the GPM configuration.
pub struct NamespaceProp {
    /// Full path to the namespace directory
    pub path: PathBuf,
}

impl NamespaceProp {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path: NAMESPACES_PATH.join(path),
        }
    }

    pub fn add(&self) -> Result<()> {
        fs::create_dir_all(&self.path)?;
        let cfg_path = Path::new(&self.path).join(NAMESPACES_CONFIG);
        package_config::Config::new().save(&cfg_path)?;
        Ok(())
    }

    pub fn remove(&self) -> Result<()> {
        fs::remove_dir_all(&self.path)?;
        Ok(())
    }
}
