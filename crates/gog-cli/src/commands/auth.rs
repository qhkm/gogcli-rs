// Auth command - Auth and credentials management.
// Mirrors: internal/cmd/auth.go

use clap::{Parser, Subcommand};
use anyhow::Result;

use crate::GlobalFlags;

/// Auth and credentials management.
#[derive(Debug, Parser)]
#[command(about = "Auth and credentials")]
pub struct AuthCmd {
    #[command(subcommand)]
    pub subcommand: AuthSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum AuthSubcommand {
    /// Authorize and store a refresh token (OAuth browser flow).
    #[command(aliases = ["login"])]
    Add(AuthAddArgs),

    /// Remove a stored refresh token.
    #[command(aliases = ["logout"])]
    Remove(AuthRemoveArgs),

    /// Show auth/config status for an account.
    #[command(aliases = ["st"])]
    Status(AuthStatusArgs),

    /// List all authenticated accounts.
    #[command(aliases = ["accounts"])]
    List(AuthListArgs),
}

#[derive(Debug, Parser)]
pub struct AuthAddArgs {
    /// Account email to authorize.
    pub email: Option<String>,

    /// Google API services to authorize (comma-separated).
    #[arg(long, value_delimiter = ',')]
    pub services: Vec<String>,
}

#[derive(Debug, Parser)]
pub struct AuthRemoveArgs {
    /// Account email to remove.
    pub email: Option<String>,
}

#[derive(Debug, Parser)]
pub struct AuthStatusArgs {
    /// Account email to check.
    pub email: Option<String>,
}

#[derive(Debug, Parser)]
pub struct AuthListArgs {}

/// Execute the auth command.
pub async fn execute(cmd: &AuthCmd, _flags: &GlobalFlags) -> Result<()> {
    match &cmd.subcommand {
        AuthSubcommand::Add(args) => execute_add(args, _flags).await,
        AuthSubcommand::Remove(args) => execute_remove(args, _flags).await,
        AuthSubcommand::Status(args) => execute_status(args, _flags).await,
        AuthSubcommand::List(args) => execute_list(args, _flags).await,
    }
}

async fn execute_add(_args: &AuthAddArgs, _flags: &GlobalFlags) -> Result<()> {
    todo!("implement auth add")
}

async fn execute_remove(_args: &AuthRemoveArgs, _flags: &GlobalFlags) -> Result<()> {
    todo!("implement auth remove")
}

async fn execute_status(_args: &AuthStatusArgs, _flags: &GlobalFlags) -> Result<()> {
    todo!("implement auth status")
}

async fn execute_list(_args: &AuthListArgs, _flags: &GlobalFlags) -> Result<()> {
    todo!("implement auth list")
}
