//! Handling package type configuration file at TYPES_CONFIG.

use crate::{error, TYPES_CONFIG};

use anyhow::{anyhow, Result};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::io::Write;
use std::{fmt, fs};
use tabwriter::TabWriter;

// Separate from the Config struct to allow more flexibility in the future.
#[derive(Debug, Deserialize, Serialize)]
struct TomlTypeConfig {
    /// Key: type name, Value: type properties
    types: HashMap<String, TomlTypeProp>,
}

impl TomlTypeConfig {
    fn into_config(self) -> TypeConfig {
        TypeConfig {
            types: self
                .types
                .into_iter()
                .map(|(name, ns)| (name, TypeProp { ext: ns.ext }))
                .collect(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct TomlTypeProp {
    ext: String,
}

/// Configuration for package types.
#[derive(Debug)]
pub struct TypeConfig {
    /// Key: type name, Value: type properties
    pub types: HashMap<String, TypeProp>,
}

impl TypeConfig {
    pub fn new() -> Self {
        Self {
            types: HashMap::new(),
        }
    }

    /// Load the configuration, or calls `new()` if it doesn't exist.
    pub fn load() -> Result<Self> {
        let path = TYPES_CONFIG.as_path();
        if !path.exists() {
            Ok(Self::new())
        } else {
            toml::from_str::<TomlTypeConfig>(&fs::read_to_string(TYPES_CONFIG.as_path())?)
                .map(|c| c.into_config())
                .map_err(Into::into)
        }
    }

    /// Save the configuration.
    pub fn save(self) -> Result<()> {
        fs::write(
            TYPES_CONFIG.as_path(),
            toml::to_string(&self.into_toml_config())?,
        )
        .map_err(Into::into)
    }

    fn into_toml_config(self) -> TomlTypeConfig {
        TomlTypeConfig {
            types: self
                .types
                .into_iter()
                .map(|(name, ns)| (name, TomlTypeProp { ext: ns.ext }))
                .collect(),
        }
    }

    /// Add a new type.
    pub fn add(&mut self, name: String, prop: TypeProp) -> Result<()> {
        if let Entry::Vacant(e) = self.types.entry(name.clone()) {
            e.insert(prop);
            Ok(())
        } else {
            Err(anyhow!("type '{}' already exists", name.bright_yellow()))
        }
    }

    /// Remove types.
    pub fn remove(&mut self, names: Vec<String>) {
        for name in names {
            match self.types.remove(&name) {
                Some(_) => (),
                None => error!("type '{}' does not exist", name.bright_yellow()),
            }
        }
    }
}

impl Default for TypeConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for TypeConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut tw = TabWriter::new(vec![]);
        writeln!(&mut tw, "{}", "Types:".bright_green()).unwrap();
        for (name, prop) in &self.types {
            writeln!(
                &mut tw,
                "  {}\t{}",
                name.bright_cyan(),
                prop.ext.bright_purple()
            )
            .unwrap();
        }
        tw.flush().unwrap();
        let result = String::from_utf8(tw.into_inner().unwrap()).unwrap();
        write!(f, "{}", result)
    }
}

#[derive(Debug)]
pub struct TypeProp {
    pub ext: String,
}

impl TypeProp {
    pub fn new(ext: String) -> Self {
        Self { ext }
    }
}
