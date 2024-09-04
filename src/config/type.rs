//! Handling package type configuration file at TYPES_CONFIG.

use super::util::{prompt, sort_keys};
use crate::{add, error, remove, SCRIPT_ROOT, TYPES_CONFIG};

use anyhow::{anyhow, Result};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::collections::{hash_map::Entry, BTreeMap, HashMap};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Stdio;
use std::{fmt, fs};
use tabwriter::TabWriter;

// Separate from the Config struct to allow more flexibility in the future.
#[derive(Debug, Deserialize, Serialize)]
struct TomlTypeConfig {
	#[serde(serialize_with = "sort_keys")]
	shell: HashMap<String, Box<[String]>>,
	/// Key: type name, Value: type properties
	#[serde(serialize_with = "sort_keys")]
	types: HashMap<String, TomlTypeProp>,
}

impl From<TypeConfig> for TomlTypeConfig {
	fn from(t: TypeConfig) -> Self {
		Self {
			types: t
				.types
				.into_iter()
				.map(|(name, type_prop)| (name, type_prop.into()))
				.collect(),
			shell: t.shell,
		}
	}
}

#[derive(Debug, Deserialize, Serialize)]
struct TomlTypeProp {
	ext: String,
	shell: String,
}

impl From<TypeProp> for TomlTypeProp {
	fn from(prop: TypeProp) -> Self {
		Self {
			ext: prop.ext,
			shell: prop.shell,
		}
	}
}

/// Configuration for package types.
#[derive(Debug)]
pub struct TypeConfig {
	/// Key: type name, Value: type properties
	shell: HashMap<String, Box<[String]>>,
	types: HashMap<String, TypeProp>,
}

impl TypeConfig {
	pub fn new() -> Self {
		#[cfg(target_os = "windows")]
		{
			Self {
				shell: HashMap::from([("powershell".into(), Box::from(["-c".into()]))]),
				types: HashMap::new(),
			}
		}
		#[cfg(not(target_os = "windows"))]
		{
			Self {
				shell: HashMap::from([("bash".into(), Box::from(["-c".into()]))]),
				types: HashMap::new(),
			}
		}
	}

	/// Load the configuration, or calls `new()` if it doesn't exist.
	pub fn load() -> Result<Self> {
		if !TYPES_CONFIG.exists() {
			Ok(Self::new())
		} else {
			toml::from_str::<TomlTypeConfig>(&fs::read_to_string(&*TYPES_CONFIG)?)
				.map(|c| c.into())
				.map_err(Into::into)
		}
	}

	/// Save the configuration.
	pub fn save(self) -> Result<()> {
		fs::write(
			&*TYPES_CONFIG,
			toml::to_string(&TomlTypeConfig::from(self))?,
		)
		.map_err(Into::into)
	}

	/// Add a new type.
	pub fn add(&mut self, name: String, ext: String, shell: String) -> Result<()> {
		if let Entry::Vacant(e) = self.types.entry(name.clone()) {
			let path = SCRIPT_ROOT.join(format!("{}.{}", name, ext));
			if !path.exists() {
				File::create(path)?;
			}
			add!("{}\t{}\t{}", name.bright_cyan(), ext.bright_purple(), shell);
			e.insert(TypeProp::new(ext, shell));
			Ok(())
		} else {
			Err(anyhow!("type '{}' already exists", name.bright_yellow()))
		}
	}

	/// Remove types and delete the script files.
	pub fn remove(&mut self, names: Vec<String>) {
		for name in names {
			match self.types.remove(&name) {
				Some(r#type) => {
					match fs::remove_file(SCRIPT_ROOT.join(&name).with_extension(&r#type.ext)) {
						Ok(_) => remove!("{}", name.bright_cyan()),
						Err(e) => {
							error!(e);
							match prompt("Remove from registry?") {
								Ok(true) => remove!("{}", name.bright_cyan()),
								Ok(false) => {}
								Err(e) => error!(e),
							}
						}
					}
				}
				None => error!("type '{}' does not exist", name.bright_yellow()),
			}
		}
	}

	/// Remove types without deleting the script files.
	pub fn remove_registry(&mut self, names: Vec<String>) {
		for name in names {
			match self.types.remove(&name) {
				Some(_) => remove!("{}", name.bright_cyan()),
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
		cwd: Option<&str>,
		args: &[String],
	) -> Result<String> {
		let prop = match self.types.get(type_name) {
			Some(prop) => prop,
			None => {
				return Err(anyhow!(
					"type '{}' does not exist",
					type_name.bright_yellow()
				))
			}
		};

		let (shell, shell_args) = match self.shell.get_key_value(&prop.shell) {
			Some(s) => s,
			None => {
				return Err(anyhow!(
					"shell '{}' does not exist",
					prop.shell.bright_yellow()
				))
			}
		};
		let mut cmd = std::process::Command::new(shell);
		cmd.current_dir(repo_path).args(shell_args.iter());
		cmd.arg(SCRIPT_ROOT.join(type_name).with_extension(&prop.ext))
			.arg("-name")
			.arg(name);
		if let Some(cwd) = cwd {
			cmd.arg("-cwd").arg(cwd);
		}
		if let Some(etag) = etag {
			cmd.arg("-etag").arg(etag);
		}
		cmd.args(args);
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

impl From<TomlTypeConfig> for TypeConfig {
	fn from(t: TomlTypeConfig) -> Self {
		Self {
			types: t
				.types
				.into_iter()
				.map(|(name, type_prop)| (name, type_prop.into()))
				.collect(),
			shell: t.shell,
		}
	}
}

impl fmt::Display for TypeConfig {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut tw = TabWriter::new(vec![]);
		writeln!(&mut tw, "{}", "Shell:".bright_green()).unwrap();
		let btree_map: BTreeMap<_, _> = self.shell.iter().collect();
		for (name, args) in &btree_map {
			writeln!(
				&mut tw,
				"  {}\t{}",
				name.bright_cyan(),
				args.join(" ").bright_purple()
			)
			.unwrap();
		}

		writeln!(&mut tw, "{}", "Types:".bright_green()).unwrap();
		for (name, prop) in &self.types {
			writeln!(
				&mut tw,
				"  {}\t{}\t{}",
				name.bright_cyan(),
				prop.ext.bright_purple(),
				prop.shell,
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
	ext: String,
	shell: String,
}

impl TypeProp {
	pub fn new(ext: String, shell: String) -> Self {
		Self { ext, shell }
	}
}

impl From<TomlTypeProp> for TypeProp {
	fn from(prop: TomlTypeProp) -> Self {
		Self {
			ext: prop.ext,
			shell: prop.shell,
		}
	}
}
