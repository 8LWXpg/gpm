//! Handling main configuration file at GPM_CONFIG.

// use crate::namespace_config;
use crate::{error, GPM_CONFIG, NAMESPACES_CONFIG, NAMESPACES_PATH};

use anyhow::{anyhow, Result};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::{fmt, fs};
use tabwriter::TabWriter;

// Separate from the Config struct to allow more flexibility in the future.
#[derive(Debug, Deserialize, Serialize)]
struct TomlConfig {
    /// Key: namespace name, Value: namespace properties
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
    /// Key: namespace name, Value: namespace properties
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
        if let Entry::Vacant(e) = self.namespaces.entry(name.clone()) {
            e.insert(ns);
            Ok(())
        } else {
            Err(anyhow!(
                "namespace '{}' already exists",
                name.bright_yellow()
            ))
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

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut tw = TabWriter::new(vec![]);
        writeln!(&mut tw, "{}", "Namespaces:".bright_green()).unwrap();
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
        let result = String::from_utf8(tw.into_inner().unwrap()).unwrap();
        write!(f, "{}", result)
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

    fn add(&self) -> Result<()> {
        fs::create_dir_all(&self.path)?;
        let cfg_path = Path::new(&self.path).join(NAMESPACES_CONFIG);
        // namespace_config::Config::new().save(&cfg_path)?;
        Ok(())
    }

    fn remove(&self) -> Result<()> {
        fs::remove_dir_all(&self.path)?;
        Ok(())
    }
}
