//! Handling repository configuration file at main_config::RepositoryProp.path.

use crate::type_config::{ReturnType, TypeConfig};
use crate::{error, REPO_PATH};

use anyhow::{anyhow, Ok, Result};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use reqwest::{header, Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::env::current_dir;
use std::fmt;
use std::fs::{self, File};
use std::io;
use std::io::Write;
use std::path::Path;
use tokio::runtime::Builder;

// Separate from the Config struct to allow more flexibility in the future.
#[derive(Debug, Deserialize, Serialize)]
pub struct TomlConfig {
    /// Key: package name, Value: package details
    pub packages: HashMap<String, TomlPackage>,
}

impl TomlConfig {
    pub fn into_config(self) -> Config {
        Config {
            packages: self
                .packages
                .into_iter()
                .map(|(name, package)| {
                    (
                        name,
                        Package {
                            r#type: package.r#type,
                            args: package.args,
                            etag: package.etag,
                        },
                    )
                })
                .collect(),
            type_config: TypeConfig::load().expect("failed to load type config"),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TomlPackage {
    pub r#type: String,
    pub args: Box<[String]>,
    /// ETag for the package
    pub etag: Option<String>,
}

#[derive(Debug)]
pub struct Config {
    /// Key: package name, Value: package details
    pub packages: HashMap<String, Package>,
    type_config: TypeConfig,
}

impl Config {
    /// Create a empty config, panic if failed to load TypeConfig.
    pub fn new() -> Self {
        Self {
            packages: HashMap::new(),
            type_config: TypeConfig::load().expect("failed to load type config"),
        }
    }

    /// Load from a TOML file at path.
    pub fn load(path: &Path) -> Result<Self> {
        toml::from_str::<TomlConfig>(&fs::read_to_string(path)?)
            .map(|c| c.into_config())
            .map_err(Into::into)
    }

    /// Save to a TOML file at path.
    pub fn save(self, path: &Path) -> Result<()> {
        fs::write(path, toml::to_string(&self.into_toml_config())?).map_err(Into::into)
    }

    pub fn print(&self) -> String {
        let mut tw = tabwriter::TabWriter::new(vec![]);
        for (name, package) in &self.packages {
            writeln!(
                &mut tw,
                "  {}\t{}\t{}",
                name.bright_cyan(),
                package.r#type.bright_purple(),
                package.args.join(" ")
            )
            .unwrap();
        }
        tw.flush().unwrap();
        String::from_utf8(tw.into_inner().unwrap()).unwrap()
    }

    pub fn into_toml_config(self) -> TomlConfig {
        TomlConfig {
            packages: self
                .packages
                .into_iter()
                .map(|(name, package)| {
                    (
                        name,
                        TomlPackage {
                            r#type: package.r#type,
                            args: package.args,
                            etag: package.etag,
                        },
                    )
                })
                .collect(),
        }
    }

    /// Add a package and execute the script.
    pub fn add(&mut self, name: String, r#type: String, args: Box<[String]>) -> Result<()> {
        if let Entry::Vacant(e) = self.packages.entry(name.clone()) {
            let mut package = Package::new(r#type, args);
            package.add(&name, &self.type_config)?;
            e.insert(package);
            Ok(())
        } else {
            Err(anyhow!("package '{}' already exists", name.bright_yellow()))
        }
    }

    /// Remove packages.
    pub fn remove(&mut self, names: Vec<String>) {
        for name in names {
            match self.packages.remove(&name) {
                Some(package) => package.remove(&name).unwrap_or_else(|e| {
                    error!("failed to remove package '{}' {}", name.bright_yellow(), e)
                }),
                None => error!("package '{}' does not exist", name.bright_yellow()),
            }
        }
    }

    /// Update packages.
    pub fn update(&mut self, names: Vec<String>) {
        for name in names {
            match self.packages.get_mut(&name) {
                Some(package) => package.add(&name, &self.type_config).unwrap_or_else(|e| {
                    error!("failed to update package '{}' {}", name.bright_yellow(), e)
                }),
                None => error!("package '{}' does not exist", name.bright_yellow()),
            }
        }
    }

    /// Update all packages.
    pub fn update_all(&mut self) {
        for (name, package) in &mut self.packages {
            package.add(name, &self.type_config).unwrap_or_else(|e| {
                error!("failed to update package '{}' {}", name.bright_yellow(), e)
            });
        }
    }

    /// Clone packages to the current directory.
    pub fn copy(&self, names: Vec<String>) {
        for name in names {
            match self.packages.get(&name) {
                Some(package) => package.copy(&name).unwrap_or_else(|e| {
                    error!("failed to copy package '{}' {}", name.bright_yellow(), e)
                }),
                None => error!("package '{}' does not exist", name.bright_yellow()),
            }
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct Package {
    r#type: String,
    args: Box<[String]>,
    /// ETag for the package
    etag: Option<String>,
}

impl Package {
    pub fn new(r#type: String, args: Box<[String]>) -> Self {
        Self {
            r#type,
            args,
            etag: None,
        }
    }

    /// Add package, execute the script.
    fn add(&mut self, name: &str, type_config: &TypeConfig) -> Result<()> {
        let type_prop = type_config
            .types
            .get(&self.r#type)
            .ok_or_else(|| anyhow!("type '{}' does not exist", self.r#type.bright_yellow()))?;
        match type_prop.return_type {
            ReturnType::Url => {
                let mut file = File::create(REPO_PATH.join(name))?;
                let rt = Builder::new_current_thread().build()?;
                if let Some(etag) = rt.block_on(download_with_progress(
                    type_config.execute(&self.r#type, &self.args)?.as_str(),
                    &mut file,
                    self.etag.as_deref(),
                ))? {
                    self.etag = Some(etag);
                    type_config.execute_post(&self.r#type, &self.args)?;
                }
            }
            ReturnType::None => {
                type_config.execute(&self.r#type, &self.args)?;
                type_config.execute_post(&self.r#type, &self.args)?;
            }
        }
        Ok(())
    }

    fn remove(&self, name: &str) -> Result<()> {
        let path = REPO_PATH.join(name);
        match fs::metadata(&path) {
            io::Result::Ok(meta) => {
                if meta.is_dir() {
                    fs::remove_dir_all(path)?;
                } else {
                    fs::remove_file(path)?;
                }
            }
            Err(e) => return Err(e.into()),
        }
        Ok(())
    }

    fn copy(&self, name: &str) -> Result<()> {
        let path = REPO_PATH.join(name);
        let to = current_dir()?.join(name);
        if fs::metadata(&path)?.is_dir() {
            copy_dir_all(REPO_PATH.join(name), path)?;
        } else {
            fs::copy(path, to)?;
        }
        Ok(())
    }
}

/// Download a file from a URL and save it to a path.
async fn download_with_progress(
    url: &str,
    file: &mut File,
    etag: Option<&str>,
) -> Result<Option<String>> {
    let client = Client::new();
    let request = client.get(url);

    let mut response = if let Some(etag) = etag {
        request.header(header::IF_NONE_MATCH, etag).send().await?
    } else {
        request.send().await?
    };

    match response.status() {
        code if code.is_success() => {
            let etag = response
                .headers()
                .get(header::ETAG)
                .map(|etag| etag.to_str().unwrap().to_owned());

            match &response.content_length() {
                Some(len) => {
                    let bar = ProgressBar::new(*len);
                    bar.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar}] {bytes}/{total_bytes} (ETA {eta})")?
                .with_key("eta", |state: &ProgressState, w: &mut dyn fmt::Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
                .progress_chars("=> "));
                    while let Some(chunk) = response.chunk().await? {
                        file.write_all(&chunk)?;
                        bar.inc(chunk.len() as u64);
                    }
                    bar.finish();
                }
                None => {
                    let content = response.bytes().await?;
                    file.write_all(&content)?;
                }
            }
            Ok(etag)
        }
        StatusCode::NOT_MODIFIED => Ok(None),
        code => Err(anyhow!(
            "failed to download '{}' {}",
            url,
            code.canonical_reason().unwrap_or_default()
        )),
    }
}

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}
