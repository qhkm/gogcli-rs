// Config command.
// Mirrors: internal/cmd/config_cmd.go

use clap::{Parser, Subcommand};
use anyhow::{bail, Result};

use gog_core::config;

use crate::GlobalFlags;

/// Show or edit configuration.
#[derive(Debug, Parser)]
#[command(about = "Show/edit config")]
pub struct ConfigCmd {
    #[command(subcommand)]
    pub subcommand: Option<ConfigSubcommand>,
}

#[derive(Debug, Subcommand)]
pub enum ConfigSubcommand {
    /// Show the current configuration.
    Show(ConfigShowArgs),

    /// Get a specific configuration value.
    Get(ConfigGetArgs),

    /// Set a configuration value.
    Set(ConfigSetArgs),

    /// Reset configuration to defaults.
    Reset(ConfigResetArgs),
}

#[derive(Debug, Parser)]
pub struct ConfigShowArgs {
    /// Show the config file path.
    #[arg(long)]
    pub path: bool,
}

#[derive(Debug, Parser)]
pub struct ConfigGetArgs {
    /// Configuration key.
    pub key: String,
}

#[derive(Debug, Parser)]
pub struct ConfigSetArgs {
    /// Configuration key.
    pub key: String,

    /// Configuration value.
    pub value: String,
}

#[derive(Debug, Parser)]
pub struct ConfigResetArgs {
    /// Key to reset (resets all if not specified).
    pub key: Option<String>,
}

/// Execute the config command.
pub async fn execute(cmd: &ConfigCmd, flags: &GlobalFlags) -> Result<()> {
    match &cmd.subcommand {
        None | Some(ConfigSubcommand::Show(_)) => execute_show(flags).await,
        Some(ConfigSubcommand::Get(args)) => execute_get(args, flags).await,
        Some(ConfigSubcommand::Set(args)) => execute_set(args, flags).await,
        Some(ConfigSubcommand::Reset(args)) => execute_reset(args, flags).await,
    }
}

async fn execute_show(flags: &GlobalFlags) -> Result<()> {
    let cfg = config::read_config()?;
    if flags.json {
        crate::client::output_json(flags, &cfg)?;
    } else {
        let json = serde_json::to_string_pretty(&cfg)?;
        println!("{json}");
    }
    Ok(())
}

async fn execute_get(args: &ConfigGetArgs, flags: &GlobalFlags) -> Result<()> {
    let cfg = config::read_config()?;
    let value: Option<String> = match args.key.as_str() {
        "keyring_backend" => cfg.keyring_backend.clone(),
        "default_timezone" => cfg.default_timezone.clone(),
        key if key.starts_with("account_aliases.") => {
            let alias = key.strip_prefix("account_aliases.").unwrap();
            cfg.account_aliases.get(alias).cloned()
        }
        key if key.starts_with("account_clients.") => {
            let email = key.strip_prefix("account_clients.").unwrap();
            cfg.account_clients.get(email).cloned()
        }
        key if key.starts_with("client_domains.") => {
            let domain = key.strip_prefix("client_domains.").unwrap();
            cfg.client_domains.get(domain).cloned()
        }
        other => bail!("unknown config key: {other}"),
    };

    match value {
        Some(v) => {
            if flags.json {
                crate::client::output_json(flags, &serde_json::json!({ &args.key: v }))?;
            } else {
                println!("{v}");
            }
        }
        None => {
            if !flags.json {
                println!("(not set)");
            }
        }
    }
    Ok(())
}

async fn execute_set(args: &ConfigSetArgs, _flags: &GlobalFlags) -> Result<()> {
    let mut cfg = config::read_config()?;

    match args.key.as_str() {
        "keyring_backend" => cfg.keyring_backend = Some(args.value.clone()),
        "default_timezone" => cfg.default_timezone = Some(args.value.clone()),
        key if key.starts_with("account_aliases.") => {
            let alias = key.strip_prefix("account_aliases.").unwrap().to_string();
            cfg.account_aliases.insert(alias, args.value.clone());
        }
        key if key.starts_with("account_clients.") => {
            let email = key.strip_prefix("account_clients.").unwrap().to_string();
            cfg.account_clients.insert(email, args.value.clone());
        }
        key if key.starts_with("client_domains.") => {
            let domain = key.strip_prefix("client_domains.").unwrap().to_string();
            cfg.client_domains.insert(domain, args.value.clone());
        }
        other => bail!("unknown config key: {other}"),
    }

    config::write_config(&cfg)?;
    println!("Set {} = {}", args.key, args.value);
    Ok(())
}

async fn execute_reset(args: &ConfigResetArgs, _flags: &GlobalFlags) -> Result<()> {
    match &args.key {
        Some(key) => {
            let mut cfg = config::read_config()?;
            match key.as_str() {
                "keyring_backend" => cfg.keyring_backend = None,
                "default_timezone" => cfg.default_timezone = None,
                "account_aliases" => cfg.account_aliases.clear(),
                "account_clients" => cfg.account_clients.clear(),
                "client_domains" => cfg.client_domains.clear(),
                other => bail!("unknown config key: {other}"),
            }
            config::write_config(&cfg)?;
            println!("Reset {key}");
        }
        None => {
            config::write_config(&config::ConfigFile::default())?;
            println!("Reset all configuration to defaults");
        }
    }
    Ok(())
}
