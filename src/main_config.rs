use crate::package_config;
use crate::{GPM_CONFIG, NAMESPACES_CONFIG};
use anyhow::Result;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::Path;
use tabwriter::TabWriter;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub namespaces: Vec<NamespaceConfig>,
}

impl Config {
    pub fn print(&self) -> String {
        let mut tw = TabWriter::new(vec![]);
        for ns in &self.namespaces {
            writeln!(&mut tw, "  {}\t{}", ns.name.bright_cyan(), ns.path).unwrap();
        }
        tw.flush().unwrap();
        String::from_utf8(tw.into_inner().unwrap()).unwrap()
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NamespaceConfig {
    pub name: String,
    /// Full path to the namespace directory
    pub path: String,
}

impl NamespaceConfig {
    pub fn new(name: String, path: String) -> Self {
        Self { name, path }
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

pub fn load_config() -> Result<Config> {
    toml::from_str(&fs::read_to_string(GPM_CONFIG.as_path())?).map_err(Into::into)
}

pub fn save_config(config: &Config) -> Result<()> {
    fs::write(GPM_CONFIG.as_path(), toml::to_string(config)?).map_err(Into::into)
}
