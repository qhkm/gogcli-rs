// Gmail command.
// Mirrors: internal/cmd/gmail.go

use clap::{Parser, Subcommand};
use anyhow::Result;

use crate::GlobalFlags;

/// Gmail operations.
#[derive(Debug, Parser)]
#[command(about = "Gmail")]
pub struct GmailCmd {
    #[command(subcommand)]
    pub subcommand: GmailSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum GmailSubcommand {
    /// Search messages.
    Search(GmailSearchArgs),

    /// Get a message by ID.
    Get(GmailGetArgs),

    /// Send a message.
    Send(GmailSendArgs),

    /// List and manage labels.
    Labels(GmailLabelsArgs),

    /// Thread operations.
    #[command(aliases = ["threads"])]
    Thread(GmailThreadArgs),
}

#[derive(Debug, Parser)]
pub struct GmailSearchArgs {
    /// Gmail search query.
    pub query: String,

    /// Maximum number of results to return.
    #[arg(long, default_value = "20")]
    pub max_results: u32,

    /// Include message body in results.
    #[arg(long)]
    pub body: bool,

    /// Page token for pagination.
    #[arg(long)]
    pub page_token: Option<String>,
}

#[derive(Debug, Parser)]
pub struct GmailGetArgs {
    /// Message ID.
    pub id: String,

    /// Include raw MIME message.
    #[arg(long)]
    pub raw: bool,
}

#[derive(Debug, Parser)]
pub struct GmailSendArgs {
    /// Recipient email address(es).
    #[arg(long, short = 't', value_delimiter = ',')]
    pub to: Vec<String>,

    /// CC recipient(s).
    #[arg(long, value_delimiter = ',')]
    pub cc: Vec<String>,

    /// BCC recipient(s).
    #[arg(long, value_delimiter = ',')]
    pub bcc: Vec<String>,

    /// Subject line.
    #[arg(long, short = 's')]
    pub subject: Option<String>,

    /// Message body (plain text).
    #[arg(long, short = 'b')]
    pub body: Option<String>,

    /// File to attach.
    #[arg(long)]
    pub attach: Vec<String>,

    /// Reply to this message ID.
    #[arg(long)]
    pub reply_to: Option<String>,
}

#[derive(Debug, Parser)]
pub struct GmailLabelsArgs {
    /// Label name filter.
    pub filter: Option<String>,
}

#[derive(Debug, Parser)]
pub struct GmailThreadArgs {
    /// Thread ID.
    pub id: Option<String>,
}

/// Execute the gmail command.
pub async fn execute(cmd: &GmailCmd, flags: &GlobalFlags) -> Result<()> {
    match &cmd.subcommand {
        GmailSubcommand::Search(args) => execute_search(args, flags).await,
        GmailSubcommand::Get(args) => execute_get(args, flags).await,
        GmailSubcommand::Send(args) => execute_send(args, flags).await,
        GmailSubcommand::Labels(args) => execute_labels(args, flags).await,
        GmailSubcommand::Thread(args) => execute_thread(args, flags).await,
    }
}

async fn execute_search(_args: &GmailSearchArgs, _flags: &GlobalFlags) -> Result<()> {
    todo!("implement gmail search")
}

async fn execute_get(_args: &GmailGetArgs, _flags: &GlobalFlags) -> Result<()> {
    todo!("implement gmail get")
}

async fn execute_send(_args: &GmailSendArgs, _flags: &GlobalFlags) -> Result<()> {
    todo!("implement gmail send")
}

async fn execute_labels(_args: &GmailLabelsArgs, _flags: &GlobalFlags) -> Result<()> {
    todo!("implement gmail labels")
}

async fn execute_thread(_args: &GmailThreadArgs, _flags: &GlobalFlags) -> Result<()> {
    todo!("implement gmail thread")
}
