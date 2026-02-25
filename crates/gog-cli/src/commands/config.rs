// Config command.
// Mirrors: internal/cmd/config_cmd.go

use clap::{Parser, Subcommand};
use anyhow::Result;

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

async fn execute_show(_flags: &GlobalFlags) -> Result<()> {
    let cfg = gog_core::config::read_config()?;
    let json = serde_json::to_string_pretty(&cfg)?;
    println!("{json}");
    Ok(())
}

async fn execute_get(_args: &ConfigGetArgs, _flags: &GlobalFlags) -> Result<()> {
    todo!("implement config get")
}

async fn execute_set(_args: &ConfigSetArgs, _flags: &GlobalFlags) -> Result<()> {
    todo!("implement config set")
}

async fn execute_reset(_args: &ConfigResetArgs, _flags: &GlobalFlags) -> Result<()> {
    todo!("implement config reset")
}
