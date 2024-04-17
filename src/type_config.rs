//! Handling package type configuration file at TYPES_CONFIG.

use crate::escape_win::EscapePwsh;
use crate::{error, SCRIPT_ROOT, TYPES_CONFIG};

use anyhow::{anyhow, Result};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Stdio;
use std::{fmt, fs};
use tabwriter::TabWriter;

// Separate from the Config struct to allow more flexibility in the future.
#[derive(Debug, Deserialize, Serialize)]
struct TomlTypeConfig {
    /// Key: type name, Value: type properties
    types: HashMap<String, TomlTypeProp>,
    shell: String,
    args: Box<[String]>,
}

impl TomlTypeConfig {
    fn into_config(self) -> TypeConfig {
        TypeConfig {
            types: self
                .types
                .into_iter()
                .map(|(name, type_prop)| (name, TypeProp::new(type_prop.ext)))
                .collect(),
            shell: self.shell,
            args: self.args,
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
    shell: String,
    args: Box<[String]>,
}

impl TypeConfig {
    pub fn new() -> Self {
        #[cfg(target_os = "windows")]
        {
            Self {
                types: HashMap::new(),
                shell: "powershell".into(),
                args: Box::new(["-c".into()]),
            }
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
                .map(|(name, type_prop)| (name, TomlTypeProp { ext: type_prop.ext }))
                .collect(),
            shell: self.shell,
            args: self.args,
        }
    }

    /// Add a new type.
    pub fn add(&mut self, name: String, ext: String) -> Result<()> {
        if let Entry::Vacant(e) = self.types.entry(name.clone()) {
            let path = SCRIPT_ROOT.join(format!("{}.{}", name, ext));
            if !path.exists() {
                File::create(path)?;
            }
            e.insert(TypeProp::new(ext));
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
    pub fn execute(
        &self,
        type_name: &str,
        name: &str,
        repo_path: &Path,
        etag: Option<&str>,
        args: &[String],
    ) -> Result<String> {
        let ext = match self.types.get(type_name) {
            Some(prop) => &prop.ext,
            None => {
                return Err(anyhow!(
                    "type '{}' does not exist",
                    type_name.bright_yellow()
                ))
            }
        };

        let mut cmd = std::process::Command::new(&self.shell);
        cmd.current_dir(repo_path).args(self.args.iter());
        #[cfg(target_os = "windows")]
        {
            match self.shell.as_str() {
                "powershell" | "powershell.exe" | "pwsh" | "pwsh.exe" => {
                    cmd.arg("&")
                        .arg_pwsh(SCRIPT_ROOT.join(type_name).with_extension(ext))
                        .arg("-name")
                        .arg_pwsh(name)
                        .arg("-dest")
                        .arg_pwsh(repo_path);
                    if let Some(etag) = etag {
                        cmd.arg("-etag").arg_pwsh(etag);
                    }
                    cmd.args_pwsh(args);
                }
                _ => {
                    cmd.arg(SCRIPT_ROOT.join(type_name).with_extension(ext))
                        .arg("-name")
                        .arg(name)
                        .arg("-dest")
                        .arg(repo_path);
                    if let Some(etag) = etag {
                        cmd.arg("-etag").arg(etag);
                    }
                    cmd.args(args);
                }
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            cmd.current_dir(repo_path)
                .args(self.args.iter())
                .arg(SCRIPT_ROOT.join(type_name).with_extension(ext))
                .arg("-name")
                .arg(name)
                .arg("-dest")
                .arg(repo_path);
            if let Some(etag) = etag {
                cmd.arg("-etag").arg(etag);
            }
            cmd.args(args);
        }
        println!("{} {:?}", "executing:".bright_blue(), cmd);
        let output = cmd
            .stdin(Stdio::inherit())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .output()?;
        if output.stdout.is_empty() {
            Ok("".to_string())
        } else {
            Ok(String::from_utf8(output.stdout)?.trim().to_string())
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
}

impl TypeProp {
    pub fn new(ext: String) -> Self {
        Self { ext }
    }
}
