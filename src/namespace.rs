use crate::main_config;
use crate::package_config;
use crate::package_config::Package;
use crate::NAMESPACES_CONFIG;
use crate::{Namespace, NamespaceCommand};
use anyhow::{Ok, Result};
use colored::Colorize;
use std::path::Path;

pub fn namespace(ns: Namespace, cfg: &main_config::NamespaceConfig) -> Result<()> {
    let ns_path = Path::new(&cfg.path);
    let ns_cfg_path = ns_path.join(NAMESPACES_CONFIG);
    let mut ns_cfg = package_config::Config::load(&ns_cfg_path).unwrap_or_default();

    match ns.command {
        NamespaceCommand::Add { name, r#type, url } => {
            let package = Package::new(r#type, url);
            ns_cfg.add(ns_path, name, package)?;
            ns_cfg.save(&ns_cfg_path)?;
        }
        NamespaceCommand::Remove { name } => {
            ns_cfg.remove(ns_path, ns.name, name);
            ns_cfg.save(&ns_cfg_path)?;
        }
        NamespaceCommand::Update { name, all } => {
            ns_cfg.update(ns_path, name, all);
            ns_cfg.save(&ns_cfg_path)?;
        }
        NamespaceCommand::Clone { name } => {
            ns_cfg.copy(ns_path, name);
            ns_cfg.save(&ns_cfg_path)?;
        }
        NamespaceCommand::List => {
            println!(
                "{} in namespace '{}':",
                "Packages".bright_green(),
                ns.name.bright_green()
            );
            println!("{}", ns_cfg.print());
        }
    }
    Ok(())
}
