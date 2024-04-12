//! Handling package type configuration file at TYPES_CONFIG.

use crate::{error, SCRIPT_ROOT, TYPES_CONFIG};

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
                .map(|(name, ns)| (name, TypeProp::new(ns.ext, ns.return_type.parse().unwrap())))
                .collect(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct TomlTypeProp {
    ext: String,
    return_type: String,
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
                .map(|(name, ns)| {
                    (
                        name,
                        TomlTypeProp {
                            ext: ns.ext,
                            return_type: ns.return_type.to_string(),
                        },
                    )
                })
                .collect(),
        }
    }

    /// Add a new type.
    pub fn add(&mut self, name: String, ext: String, ret: ReturnType) -> Result<()> {
        if let Entry::Vacant(e) = self.types.entry(name.clone()) {
            e.insert(TypeProp::new(ext, ret));
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

    /// Execute a script with arguments, returning stdout.
    pub fn execute(&self, script: &str, args: Box<[String]>) -> Result<String> {
        let ext = match self.types.get(script) {
            Some(prop) => &prop.ext,
            None => return Err(anyhow!("type '{}' does not exist", script.bright_yellow())),
        };
        let output = std::process::Command::new(format!("{}.{}", script, ext))
            .current_dir(SCRIPT_ROOT.as_path())
            .args(&*args)
            .output()?;
        Ok(String::from_utf8(output.stdout)?)
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
        write!(
            f,
            "{}",
            String::from_utf8(tw.into_inner().unwrap()).unwrap()
        )
    }
}

#[derive(Debug)]
pub struct TypeProp {
    pub ext: String,
    pub return_type: ReturnType,
}

impl TypeProp {
    pub fn new(ext: String, return_type: ReturnType) -> Self {
        Self { ext, return_type }
    }
}

#[derive(Clone, Debug)]
/// What to except in script stdout.
pub enum ReturnType {
    Url,
    None,
}

impl std::str::FromStr for ReturnType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "url" => Ok(Self::Url),
            "none" => Ok(Self::None),
            _ => Err(anyhow!("invalid return type")),
        }
    }
}

impl fmt::Display for ReturnType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Url => write!(f, "url"),
            Self::None => write!(f, "none"),
        }
    }
}
