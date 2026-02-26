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

async fn execute_spaces(args: &ChatSpacesArgs, flags: &GlobalFlags) -> Result<()> {
    let auth = crate::client::build_client(gog_auth::Service::Chat, flags).await?;
    let list = gog_chat::spaces::list_spaces(&auth.client, None, Some(args.max_results)).await?;
    crate::client::output(flags, &list, || {
        let mut rows = vec![vec!["RESOURCE".into(), "NAME".into(), "TYPE".into()]];
        for s in &list.spaces {
            rows.push(vec![
                s.name.clone(),
                s.display_name.clone(),
                s.space_type.clone(),
            ]);
        }
        rows
    })
}

async fn execute_messages(args: &ChatMessagesArgs, flags: &GlobalFlags) -> Result<()> {
    let auth = crate::client::build_client(gog_auth::Service::Chat, flags).await?;
    if let Some(ref text) = args.message {
        let msg = gog_chat::messages::send_message(&auth.client, &args.space, text).await?;
        crate::client::output_json(flags, &msg)
    } else {
        let list = gog_chat::messages::list_messages(
            &auth.client,
            &args.space,
            None,
            Some(args.max_results),
            None,
        )
        .await?;
        crate::client::output(flags, &list, || {
            let mut rows = vec![vec!["RESOURCE".into(), "SENDER".into(), "TIME".into(), "TEXT".into()]];
            for m in &list.messages {
                let sender = m.sender.as_ref().map_or("-".into(), |s| s.display_name.clone());
                let text_preview: String = m.text.chars().take(60).collect();
                rows.push(vec![m.name.clone(), sender, m.create_time.clone(), text_preview]);
            }
            rows
        })
    }
}

async fn execute_members(args: &ChatMembersArgs, flags: &GlobalFlags) -> Result<()> {
    let auth = crate::client::build_client(gog_auth::Service::Chat, flags).await?;
    let list =
        gog_chat::members::list_members(&auth.client, &args.space, None, Some(args.max_results))
            .await?;
    crate::client::output(flags, &list, || {
        let mut rows = vec![vec!["RESOURCE".into(), "NAME".into(), "ROLE".into()]];
        for m in &list.memberships {
            let name = m.member.as_ref().map_or("-".into(), |u| u.display_name.clone());
            let role = m.role.clone();
            rows.push(vec![m.name.clone(), name, role]);
        }
        rows
    })
}
