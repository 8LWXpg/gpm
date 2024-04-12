use crate::main_config;
use crate::package_config;
use crate::package_config::Package;
use crate::NAMESPACES_CONFIG;
use crate::{Repository, RepositoryCommand};
use anyhow::{Ok, Result};
use colored::Colorize;
use std::path::Path;

pub fn repository(ns: Repository, cfg: &main_config::RepositoryConfig) -> Result<()> {
    let ns_path = Path::new(&cfg.path);
    let ns_cfg_path = ns_path.join(NAMESPACES_CONFIG);
    let mut ns_cfg = package_config::Config::load(&ns_cfg_path).unwrap_or_default();

    match ns.command {
        RepositoryCommand::Add { name, r#type, url } => {
            let package = Package::new(r#type, url);
            ns_cfg.add(ns_path, name, package)?;
            ns_cfg.save(&ns_cfg_path)?;
        }
        RepositoryCommand::Remove { name } => {
            ns_cfg.remove(ns_path, ns.name, name);
            ns_cfg.save(&ns_cfg_path)?;
        }
        RepositoryCommand::Update { name, all } => {
            ns_cfg.update(ns_path, name, all);
            ns_cfg.save(&ns_cfg_path)?;
        }
        RepositoryCommand::Clone { name } => {
            ns_cfg.copy(ns_path, name);
            ns_cfg.save(&ns_cfg_path)?;
        }
        RepositoryCommand::List => {
            println!(
                "{} in repository '{}':",
                "Packages".bright_green(),
                ns.name.bright_green()
            );
            println!("{}", ns_cfg.print());
        }
    }
    Ok(())
}
