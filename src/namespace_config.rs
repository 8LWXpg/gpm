//! Handling namespace configuration file at main_config::NamespaceProp.path.

use anyhow::{anyhow, Ok, Result};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use reqwest::header::HeaderValue;
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::str::FromStr;
use std::{env, fmt, result};
use tokio::runtime::Builder;
use zip::read::ZipArchive;

// Separate from the Config struct to allow more flexibility in the future.
#[derive(Debug, Deserialize, Serialize)]
pub struct TomlConfig {
    /// Key: package name, Value: package details
    pub packages: HashMap<String, TomlPackage>,
}

impl TomlConfig {
    pub fn to_config(&self) -> Config {
        Config {}
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TomlPackage {
    pub r#type: String,
    pub args: Vec<String>,
    /// ETag for the package
    pub etag: Option<String>,
}

#[derive(Debug)]
pub struct Config {
    /// Key: package name, Value: package details
    pub packages: HashMap<String, Package>,
}

impl Config {
    /// Create a empty config.
    pub fn new() -> Self {
        Self {
            packages: HashMap::new(),
        }
    }

    /// Load from a TOML file at path.
    pub fn load(path: &Path) -> Result<Self> {
        toml::from_str::<TomlConfig>(&fs::read_to_string(path)?)
            .map(|c| c.to_config())
            .map_err(Into::into)
    }

    /// Save to a TOML file at path.
    pub fn save(&self, path: &Path) -> Result<()> {
        fs::write(path, toml::to_string(&self.to_toml_config())?).map_err(Into::into)
    }

    pub fn print(&self) -> String {
        let mut tw = tabwriter::TabWriter::new(vec![]);
        for (name, package) in &self.packages {
            writeln!(
                &mut tw,
                "  {}\t{}\t{}",
                name.bright_cyan(),
                package.r#type.to_string().bright_purple(),
                package.args
            )
            .unwrap();
        }
        tw.flush().unwrap();
        String::from_utf8(tw.into_inner().unwrap()).unwrap()
    }

    pub fn to_toml_config(&self) -> TomlConfig {
        TomlConfig {
            packages: self
                .packages
                .iter()
                .map(|(name, package)| {
                    (
                        name.clone(),
                        TomlPackage {
                            r#type: package.r#type.to_string(),
                            args: package.args.clone(),
                            etag: package.etag.clone(),
                        },
                    )
                })
                .collect(),
        }
    }

    /// Add packages to the namespace.
    pub fn add(&mut self, ns_path: &Path, name: String, mut package: Package) -> Result<()> {
        if let Entry::Vacant(e) = self.packages.entry(name.clone()) {
            package.add(ns_path, &name)?;
            e.insert(package);
            Ok(())
        } else {
            Err(anyhow!(
                "Package '{}' already exists in namespace '{}'",
                name.bright_yellow(),
                ns_path.display().to_string().bright_green(),
            ))
        }
    }

    /// Remove packages from the namespace.
    pub fn remove(&mut self, ns_path: &Path, ns_name: String, names: Vec<String>) {
        for name in names {
            match self.packages.remove(&name) {
                Some(item) => {
                    item.remove(ns_path, &name).unwrap_or_else(|_| {
                        eprintln!("Failed to remove package '{}'", name.bright_yellow())
                    });
                }
                None => eprintln!(
                    "Package '{}' not found in namespace '{}'",
                    name.bright_yellow(),
                    ns_name.bright_green(),
                ),
            }
        }
    }

    /// Update packages in the namespace.
    pub fn update(&mut self, ns_path: &Path, names: Vec<String>, all: bool) {
        if all {
            self.packages.iter_mut().for_each(|(name, package)| {
                match package.update(ns_path, name) {
                    result::Result::Ok(_) => {}
                    Err(e) => eprintln!("{} {}", "error:".bright_red(), e),
                }
            });
        } else {
            for name in names {
                match self.packages.get_mut(&name) {
                    Some(package) => match package.update(ns_path, &name) {
                        result::Result::Ok(_) => {}
                        Err(e) => eprintln!("{} {}", "error:".bright_red(), e),
                    },
                    None => eprintln!(
                        "Package '{}' not found in namespace '{}'",
                        name.bright_yellow(),
                        ns_path.display().to_string().bright_green(),
                    ),
                }
            }
        }
    }

    /// Copy packages from the namespace
    pub fn copy(&self, ns_path: &Path, names: Vec<String>) {
        for name in names {
            match self.packages.get(&name) {
                Some(item) => {
                    item.copy(ns_path, &name).unwrap_or_else(|_| {
                        eprintln!("Failed to clone package '{}'", name.bright_yellow())
                    });
                }
                None => eprintln!(
                    "Package '{}' not found in namespace '{}'",
                    name.bright_yellow(),
                    ns_path.display().to_string().bright_green(),
                ),
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
    pub r#type: PackageType,
    pub args: Vec<String>,
    /// ETag for the package
    pub etag: Option<String>,
}

impl Package {
    pub fn new(r#type: PackageType, args: Vec<String>) -> Self {
        Self {
            r#type,
            args,
            etag: None,
        }
    }

    /// Call script with name `type`
    fn add(&mut self, ns_path: &Path, name: &str) -> Result<()> {
        match self.r#type {
            PackageType::Git => {
                let output = Command::new("git")
                    .args(["clone", self.args.as_ref(), name])
                    .current_dir(ns_path)
                    .status()?;
                if !output.success() {
                    return Err(anyhow!(
                        "failed to clone '{}'",
                        self.args.to_string().bright_yellow(),
                    ));
                }
            }
            PackageType::Zip => {
                let dir = ns_path.join(name);
                let zip_path = dir.with_extension("zip");
                let mut file = File::create(&zip_path)?;

                let rt = Builder::new_current_thread().enable_all().build()?;
                self.etag = rt.block_on(download_with_progress(
                    self.args.as_ref(),
                    &mut file,
                    &HeaderValue::from_static("application/zip"),
                ))?;

                let file = File::open(&zip_path)?;
                let mut archive = ZipArchive::new(file)?;
                archive.extract(dir)?;
                fs::remove_file(zip_path)?;
            }
            PackageType::Exe => {
                let mut file = {
                    #[cfg(target_os = "windows")]
                    {
                        File::create(ns_path.join(name).with_extension("exe"))?
                    }
                    #[cfg(not(target_os = "windows"))]
                    {
                        File::create(ns_path.join(&self.name))?
                    }
                };

                let rt = Builder::new_current_thread().enable_all().build()?;
                self.etag = rt.block_on(download_with_progress(
                    self.args.as_ref(),
                    &mut file,
                    &HeaderValue::from_static("application/octet-stream"),
                ))?;
            }
            PackageType::Local => {
                copy_dir_all(ns_path.join(name), &self.args)?;
            }
        }
        Ok(())
    }

    /// Remove packages on local machine.
    fn remove(&self, ns_path: &Path, name: &str) -> Result<()> {
        match &self.r#type {
            PackageType::Git | PackageType::Zip | PackageType::Local => {
                fs::remove_dir_all(ns_path.join(name))?;
            }
            PackageType::Exe => {
                #[cfg(target_os = "windows")]
                {
                    fs::remove_file(ns_path.join(name).with_extension("exe"))?;
                }
                #[cfg(not(target_os = "windows"))]
                {
                    fs::remove_file(ns_path.join(&self.name))?;
                }
            }
        }
        Ok(())
    }

    fn update(&mut self, ns_path: &Path, name: &str) -> Result<()> {
        match self.r#type {
            PackageType::Git => {
                let output = Command::new("git")
                    .args(["fetch", "--dry-run"])
                    .current_dir(ns_path.join(name))
                    .output()?;
                if !output.status.success() {
                    return Err(anyhow!(
                        "failed at '{}'",
                        "git fetch --dry-run".bright_yellow()
                    ));
                }
                if output.stdout.is_empty() {
                    return Ok(());
                }
                if !Command::new("git")
                    .args(["pull"])
                    .current_dir(ns_path.join(name))
                    .status()?
                    .success()
                {
                    return Err(anyhow!("failed at '{}'", "git pull".bright_yellow()));
                }
            }
            PackageType::Zip => {
                let rt = Builder::new_current_thread().enable_all().build()?;
                let etag = rt.block_on(head_with_etag(self.args.as_ref()))?;
                if etag == self.etag {
                    return Ok(());
                }
                self.add(ns_path, name)?
            }
            PackageType::Exe => {
                let rt = Builder::new_current_thread().enable_all().build()?;
                let etag = rt.block_on(head_with_etag(self.args.as_ref()))?;
                if etag == self.etag {
                    return Ok(());
                }
                self.add(ns_path, name)?
            }
            PackageType::Local => copy_dir_all(ns_path.join(name), &self.args)?,
        }
        Ok(())
    }

    /// Clone a package from the namespace to cwd.
    fn copy(&self, ns_path: &Path, name: &str) -> Result<()> {
        let src = ns_path.join(name);
        let dst = env::current_dir()?.join(name);
        match self.r#type {
            PackageType::Zip | PackageType::Git | PackageType::Local => {
                fs::create_dir_all(&dst)?;
                copy_dir_all(src, dst)?;
            }
            PackageType::Exe => {
                #[cfg(target_os = "windows")]
                {
                    fs::copy(src.with_extension("exe"), dst.with_extension("exe"))?;
                }
                #[cfg(not(target_os = "windows"))]
                {
                    fs::copy(src, dst)?;
                }
            }
        }
        Ok(())
    }
}

/// Download a file from a URL and save it to a path.
async fn download_with_progress(
    url: &str,
    file: &mut File,
    content_type: &header::HeaderValue,
) -> Result<Option<String>> {
    let client = Client::new();
    let mut response = client.get(url).send().await?;

    if response.headers().get(header::CONTENT_TYPE) != Some(content_type) {
        return Err(anyhow!(
            "URL '{}' does not return {}",
            url.bright_yellow(),
            content_type.to_str()?.bright_yellow()
        ));
    }
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

async fn head_with_etag(url: &str) -> Result<Option<String>> {
    let client = Client::new();
    let response = client.head(url).send().await?;
    Ok(response
        .headers()
        .get(header::ETAG)
        .map(|etag| etag.to_str().unwrap().to_owned()))
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
