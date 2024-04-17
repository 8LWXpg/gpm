//! Handling packages under repositories.

use crate::type_config::TypeConfig;
use crate::{error, REPO_PATH};

use anyhow::{anyhow, Ok, Result};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::env::current_dir;
use std::io;
use std::io::Write;
use std::path::Path;
use std::{fmt, fs};

// Separate from the Config struct to allow more flexibility in the future.
#[derive(Debug, Deserialize, Serialize)]
struct TomlRepo {
    /// Key: package name, Value: package details
    packages: HashMap<String, TomlPackage>,
}

impl TomlRepo {
    pub fn into_config(self, path: &Path) -> Repo {
        Repo {
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
            path: path.into(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct TomlPackage {
    r#type: String,
    args: Box<[String]>,
    /// ETag for the package
    etag: Option<String>,
}

#[derive(Debug)]
pub struct Repo {
    /// Key: package name, Value: package details
    pub packages: HashMap<String, Package>,
    type_config: TypeConfig,
    /// Path to the repository
    path: Box<Path>,
}

impl Repo {
    /// Create a empty config, panic if failed to load TypeConfig.
    pub fn new(path: &Path) -> Self {
        Self {
            packages: HashMap::new(),
            type_config: TypeConfig::load().expect("failed to load type config"),
            path: REPO_PATH.join(path).into_boxed_path(),
        }
    }

    /// Load from a TOML file at path.
    pub fn load(path: &Path) -> Result<Self> {
        toml::from_str::<TomlRepo>(&fs::read_to_string(path).map_err(|_| {
            anyhow!(
                "failed to load config at '{}'",
                path.display().to_string().bright_yellow(),
            )
        })?)
        .map(|c| c.into_config(path.parent().unwrap()))
        .map_err(Into::into)
    }

    /// Save to a TOML file at path.
    pub fn save(self, path: &Path) -> Result<()> {
        fs::write(path, toml::to_string(&self.into_toml_config())?).map_err(Into::into)
    }

    fn into_toml_config(self) -> TomlRepo {
        TomlRepo {
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
            package.add(&name, &self.path, &self.type_config)?;
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
                Some(package) => package
                    .add(&name, &self.path, &self.type_config)
                    .unwrap_or_else(|e| {
                        error!("failed to update package '{}' {}", name.bright_yellow(), e)
                    }),
                None => error!("package '{}' does not exist", name.bright_yellow()),
            }
        }
    }

    /// Update all packages.
    pub fn update_all(&mut self) {
        for (name, package) in &mut self.packages {
            package
                .add(name, &self.path, &self.type_config)
                .unwrap_or_else(|e| {
                    error!("failed to update package '{}' {}", name.bright_yellow(), e)
                });
        }
    }

    /// Clone packages to the current directory.
    pub fn copy(&self, names: Vec<String>) {
        for name in names {
            match self.packages.get(&name) {
                Some(package) => package.copy(&self.path, &name).unwrap_or_else(|e| {
                    error!("failed to copy package '{}' {}", name.bright_yellow(), e)
                }),
                None => error!("package '{}' does not exist", name.bright_yellow()),
            }
        }
    }
}

impl fmt::Display for Repo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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
        write!(
            f,
            "{}",
            String::from_utf8(tw.into_inner().unwrap()).unwrap()
        )
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
    fn add(&mut self, name: &str, repo_path: &Path, type_config: &TypeConfig) -> Result<()> {
        let etag = type_config.execute(
            &self.r#type,
            name,
            repo_path,
            self.etag.as_deref(),
            &self.args,
        )?;
        if !etag.is_empty() {
            self.etag = Some(etag);
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

    fn copy(&self, repo_path: &Path, name: &str) -> Result<()> {
        let from = repo_path.join(name);
        let to = current_dir()?.join(name);
        if fs::metadata(&from)?.is_dir() {
            copy_dir_all(from, to)?;
        } else {
            fs::copy(from, to)?;
        }
        Ok(())
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
