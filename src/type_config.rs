//! Handling package type configuration file at TYPES_CONFIG.

use crate::{error, info, main_config, SCRIPT_ROOT, TYPES_CONFIG};

use anyhow::{anyhow, Result};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::process::Stdio;
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
        if !TYPES_CONFIG.exists() {
            Ok(Self::new())
        } else {
            toml::from_str::<TomlTypeConfig>(&fs::read_to_string(&*TYPES_CONFIG)?)
                .map(|c| c.into_config())
                .map_err(Into::into)
        }
    }

    /// Save the configuration.
    pub fn save(self) -> Result<()> {
        fs::write(&*TYPES_CONFIG, toml::to_string(&self.into_toml_config())?).map_err(Into::into)
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
            let path = SCRIPT_ROOT.join(format!("{}.{}", name, ext));
            if !path.exists() {
                File::create(path)?;
            }
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

    /// Execute script with arguments, returning stdout.
    pub fn execute(&self, script: &str, args: &[String], repo_path: &Path) -> Result<String> {
        let ext = match self.types.get(script.trim_end_matches(".post")) {
            Some(prop) => &prop.ext,
            None => {
                return Err(anyhow!(
                    "script '{}' does not exist",
                    script.bright_yellow()
                ))
            }
        };
        let main_cfg = main_config::Config::load()?;
        let shell = &main_cfg.shell;
        let shell_args = &main_cfg.args;
        let mut output = std::process::Command::new(shell)
            .current_dir(repo_path)
            .args(shell_args.iter())
            .arg(SCRIPT_ROOT.join(script).with_extension(ext))
            .args(args)
            .stdout(Stdio::piped())
            .spawn()?;

        info!(
            "executing script '{:?}'",
            std::process::Command::new(shell)
                .current_dir(repo_path)
                .args(shell_args.iter())
                .arg(SCRIPT_ROOT.join(script).with_extension(ext))
                .args(args)
        );
        let mut out = String::new();
        output.stdout.take().unwrap().read_to_string(&mut out)?;
        Ok(out)
    }

    /// Execute post install script with arguments, returning stdout.
    pub fn execute_post(&self, script: &str, args: &[String], repo_path: &Path) -> Result<String> {
        let path = SCRIPT_ROOT.join(format!("{}.post.{}", script, self.types[script].ext));
        if path.exists() {
            self.execute(&format!("{}.post", script), args, repo_path)
        } else {
            Ok("".to_string())
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
