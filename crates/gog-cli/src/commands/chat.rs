// Chat command.
// Mirrors: internal/cmd/chat.go

use clap::{Parser, Subcommand};
use anyhow::Result;

use crate::GlobalFlags;

/// Google Chat operations.
#[derive(Debug, Parser)]
#[command(about = "Google Chat")]
pub struct ChatCmd {
    #[command(subcommand)]
    pub subcommand: ChatSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum ChatSubcommand {
    /// List Chat spaces.
    #[command(aliases = ["space", "rooms"])]
    Spaces(ChatSpacesArgs),

    /// List or send messages in a space.
    #[command(aliases = ["message", "msg"])]
    Messages(ChatMessagesArgs),

    /// List members of a space.
    #[command(aliases = ["member"])]
    Members(ChatMembersArgs),
}

#[derive(Debug, Parser)]
pub struct ChatSpacesArgs {
    /// Maximum number of spaces to return.
    #[arg(long, default_value = "20")]
    pub max_results: u32,

    /// Filter by space type (ROOM, DM).
    #[arg(long)]
    pub space_type: Option<String>,
}

#[derive(Debug, Parser)]
pub struct ChatMessagesArgs {
    /// Space name or ID.
    pub space: String,

    /// Maximum number of messages.
    #[arg(long, default_value = "20")]
    pub max_results: u32,

    /// Text message to send (if provided, sends instead of listing).
    #[arg(long, short = 'm')]
    pub message: Option<String>,
}

#[derive(Debug, Parser)]
pub struct ChatMembersArgs {
    /// Space name or ID.
    pub space: String,

    /// Maximum number of members.
    #[arg(long, default_value = "20")]
    pub max_results: u32,
}

/// Execute the chat command.
pub async fn execute(cmd: &ChatCmd, flags: &GlobalFlags) -> Result<()> {
    match &cmd.subcommand {
        ChatSubcommand::Spaces(args) => execute_spaces(args, flags).await,
        ChatSubcommand::Messages(args) => execute_messages(args, flags).await,
        ChatSubcommand::Members(args) => execute_members(args, flags).await,
    }
}

async fn execute_spaces(_args: &ChatSpacesArgs, _flags: &GlobalFlags) -> Result<()> {
    todo!("implement chat spaces")
}

async fn execute_messages(_args: &ChatMessagesArgs, _flags: &GlobalFlags) -> Result<()> {
    todo!("implement chat messages")
}

async fn execute_members(_args: &ChatMembersArgs, _flags: &GlobalFlags) -> Result<()> {
    todo!("implement chat members")
}
