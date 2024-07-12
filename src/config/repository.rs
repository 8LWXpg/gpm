//! Handling packages under repositories.

use super::r#type::TypeConfig;
use super::util::{prompt, sort_keys};
use crate::{add, clone, error, remove, REPO_PATH};

use anyhow::{anyhow, Result};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::collections::{hash_map::Entry, BTreeMap, HashMap};
use std::io::Write;
use std::path::Path;
use std::{env, fmt, fs, io};

// Separate from the Config struct to allow more flexibility in the future.
#[derive(Debug, Deserialize, Serialize)]
struct TomlRepoConfig {
    /// Key: package name, Value: package details
    #[serde(serialize_with = "sort_keys")]
    packages: HashMap<String, TomlPackage>,
}

impl From<RepoConfig> for TomlRepoConfig {
    fn from(repo: RepoConfig) -> Self {
        Self {
            packages: repo
                .packages
                .into_iter()
                .map(|(name, package)| (name, package.into()))
                .collect(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct TomlPackage {
    r#type: String,
    args: Box<[String]>,
    /// ETag for the package
    etag: Option<String>,
    cwd: Option<String>,
}

impl From<Package> for TomlPackage {
    fn from(package: Package) -> Self {
        Self {
            r#type: package.r#type,
            args: package.args,
            etag: package.etag,
            cwd: package.cwd,
        }
    }
}

#[derive(Debug)]
pub struct RepoConfig {
    /// Key: package name, Value: package details
    packages: HashMap<String, Package>,
    type_config: TypeConfig,
    /// Path to the repository
    path: Box<Path>,
}

impl RepoConfig {
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
        toml::from_str::<TomlRepoConfig>(&fs::read_to_string(path).map_err(|e| {
            anyhow!(
                "failed to load config at '{}' {}",
                path.display().to_string().bright_yellow(),
                e
            )
        })?)
        .map(|repo| (repo, path.parent().unwrap()).into())
        .map_err(Into::into)
    }

    /// Save to a TOML file at path.
    pub fn save(self, path: &Path) -> Result<()> {
        fs::write(path, toml::to_string(&TomlRepoConfig::from(self))?).map_err(Into::into)
    }

    /// Add a package and execute the script.
    pub fn add(
        &mut self,
        name: String,
        r#type: String,
        args: Box<[String]>,
        cwd: bool,
    ) -> Result<()> {
        if let Entry::Vacant(e) = self.packages.entry(name.clone()) {
            let mut package = Package::new(r#type.clone(), args.clone(), cwd);
            package.add(&name, &self.path, &self.type_config)?;
            add!(
                "{}\t{}\t{}{}",
                name.bright_cyan(),
                r#type.bright_purple(),
                args.join(", "),
                (if cwd { "\t(cwd)" } else { "" }).bright_white()
            );
            e.insert(package);
            Ok(())
        } else {
            Err(anyhow!("package '{}' already exists", name.bright_yellow()))
        }
    }

    /// Remove packages.
    pub fn remove(&mut self, names: Vec<String>) {
        for name in names {
            match self.packages.get(&name) {
                Some(package) => match package.remove(&name, &self.path) {
                    std::result::Result::Ok(()) => {
                        self.packages.remove(&name);
                        remove!("{}", name.bright_cyan());
                    }
                    Err(e) => {
                        error!("failed to remove package '{}' {}", name.bright_yellow(), e);
                        match prompt("Remove from registry?") {
                            Ok(true) => {
                                self.packages.remove(&name);
                                remove!("{}", name.bright_cyan());
                            }
                            Ok(false) => {}
                            Err(e) => error!(e),
                        }
                    }
                },
                None => error!("package '{}' does not exist", name.bright_yellow()),
            }
        }
    }

    /// Remove packages from the registry.
    pub fn remove_registry(&mut self, names: Vec<String>) {
        for name in names {
            match self.packages.remove(&name) {
                Some(_) => remove!("{}", name.bright_cyan()),
                None => error!("package '{}' does not exist", name.bright_yellow()),
            }
        }
    }

    /// Remove ETag for packages.
    pub fn remove_etag(&mut self) {
        for package in self.packages.values_mut() {
            package.etag = None;
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
                Some(package) => match package.copy(&self.path, &name) {
                    Ok(_) => clone!("{}", name.bright_yellow()),
                    Err(e) => error!("failed to copy package '{}' {}", name.bright_yellow(), e),
                },
                None => error!("package '{}' does not exist", name.bright_yellow()),
            }
        }
    }
}

impl From<(TomlRepoConfig, &Path)> for RepoConfig {
    fn from((config, path): (TomlRepoConfig, &Path)) -> Self {
        Self {
            packages: config
                .packages
                .into_iter()
                .map(|(name, package)| (name, package.into()))
                .collect(),
            type_config: TypeConfig::load().expect("failed to load type config"),
            path: path.into(),
        }
    }
}

impl fmt::Display for RepoConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut tw = tabwriter::TabWriter::new(vec![]);
        writeln!(&mut tw, "{}", "Packages:".bright_green()).unwrap();
        let btree_map: BTreeMap<_, _> = self.packages.iter().collect();
        for (name, package) in &btree_map {
            writeln!(
                &mut tw,
                "  {}\t{}\t{}\t{}",
                name.bright_cyan(),
                package.r#type.bright_purple(),
                package.args.join(", "),
                package.cwd.as_deref().unwrap_or_default().bright_white()
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
struct Package {
    r#type: String,
    args: Box<[String]>,
    /// ETag for the package
    etag: Option<String>,
    cwd: Option<String>,
}

impl Package {
    fn new(r#type: String, args: Box<[String]>, cwd: bool) -> Self {
        Self {
            r#type,
            args,
            etag: None,
            cwd: if cwd {
                Some(env::current_dir().unwrap().to_str().unwrap().into())
            } else {
                None
            },
        }
    }

    /// Add package, execute the script.
    fn add(&mut self, name: &str, repo_path: &Path, type_config: &TypeConfig) -> Result<()> {
        let etag = type_config.execute(
            &self.r#type,
            name,
            repo_path,
            self.etag.as_deref(),
            self.cwd.as_deref(),
            &self.args,
        )?;
        if !etag.is_empty() {
            self.etag = Some(etag);
        }

        Ok(())
    }

    fn remove(&self, name: &str, repo_path: &Path) -> Result<()> {
        let path = repo_path.join(name);
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
        let to = env::current_dir()?.join(name);
        if fs::metadata(&from)?.is_dir() {
            copy_dir_all(from, to)?;
        } else {
            fs::copy(from, to)?;
        }
        Ok(())
    }
}

impl From<TomlPackage> for Package {
    fn from(package: TomlPackage) -> Self {
        Self {
            r#type: package.r#type,
            args: package.args,
            etag: package.etag,
            cwd: package.cwd,
        }
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
