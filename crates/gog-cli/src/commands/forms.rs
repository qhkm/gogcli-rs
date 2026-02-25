// Forms command.
// Mirrors: internal/cmd/forms.go

use clap::{Parser, Subcommand};
use anyhow::Result;

use crate::GlobalFlags;

/// Google Forms operations.
#[derive(Debug, Parser)]
#[command(about = "Google Forms")]
pub struct FormsCmd {
    #[command(subcommand)]
    pub subcommand: FormsSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum FormsSubcommand {
    /// Get a form by ID.
    Get(FormsGetArgs),

    /// List responses for a form.
    #[command(aliases = ["response"])]
    Responses(FormsResponsesArgs),
}

#[derive(Debug, Parser)]
pub struct FormsGetArgs {
    /// Form ID.
    pub form_id: String,
}

#[derive(Debug, Parser)]
pub struct FormsResponsesArgs {
    /// Form ID.
    pub form_id: String,

    /// Maximum number of responses.
    #[arg(long, default_value = "20")]
    pub max_results: u32,

    /// Filter by response ID.
    #[arg(long)]
    pub response_id: Option<String>,
}

/// Execute the forms command.
pub async fn execute(cmd: &FormsCmd, flags: &GlobalFlags) -> Result<()> {
    match &cmd.subcommand {
        FormsSubcommand::Get(args) => execute_get(args, flags).await,
        FormsSubcommand::Responses(args) => execute_responses(args, flags).await,
    }
}

async fn execute_get(_args: &FormsGetArgs, _flags: &GlobalFlags) -> Result<()> {
    todo!("implement forms get")
}

async fn execute_responses(_args: &FormsResponsesArgs, _flags: &GlobalFlags) -> Result<()> {
    todo!("implement forms responses")
}
