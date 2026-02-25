// Keep command.
// Mirrors: internal/cmd/keep.go

use clap::{Parser, Subcommand};
use anyhow::Result;

use crate::GlobalFlags;

/// Google Keep operations.
#[derive(Debug, Parser)]
#[command(about = "Google Keep (Workspace only)")]
pub struct KeepCmd {
    #[command(subcommand)]
    pub subcommand: KeepSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum KeepSubcommand {
    /// List notes.
    #[command(aliases = ["notes", "ls"])]
    List(KeepListArgs),

    /// Get a note by ID.
    Get(KeepGetArgs),
}

#[derive(Debug, Parser)]
pub struct KeepListArgs {
    /// Filter notes by label.
    #[arg(long, short = 'l')]
    pub label: Option<String>,

    /// Maximum number of notes.
    #[arg(long, default_value = "20")]
    pub max_results: u32,
}

#[derive(Debug, Parser)]
pub struct KeepGetArgs {
    /// Note ID.
    pub id: String,
}

/// Execute the keep command.
pub async fn execute(cmd: &KeepCmd, flags: &GlobalFlags) -> Result<()> {
    match &cmd.subcommand {
        KeepSubcommand::List(args) => execute_list(args, flags).await,
        KeepSubcommand::Get(args) => execute_get(args, flags).await,
    }
}

async fn execute_list(_args: &KeepListArgs, _flags: &GlobalFlags) -> Result<()> {
    todo!("implement keep list")
}

async fn execute_get(_args: &KeepGetArgs, _flags: &GlobalFlags) -> Result<()> {
    todo!("implement keep get")
}
