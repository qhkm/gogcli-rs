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

async fn execute_list(args: &KeepListArgs, flags: &GlobalFlags) -> Result<()> {
    let auth = crate::client::build_client(gog_auth::Service::Keep, flags).await?;
    let params = gog_keep::list::ListParams {
        page_size: Some(args.max_results),
        page_token: None,
        filter: args.label.as_ref().map(|l| format!("label = {l}")),
    };
    let list = gog_keep::list::list_notes(&auth.client, &auth.access_token, &params).await?;
    crate::client::output(flags, &list, || {
        let mut rows = vec![vec!["RESOURCE".into(), "TITLE".into(), "UPDATED".into()]];
        for n in &list.notes {
            rows.push(vec![
                n.name.clone(),
                n.title.clone(),
                n.update_time.clone().unwrap_or_default(),
            ]);
        }
        rows
    })
}

async fn execute_get(args: &KeepGetArgs, flags: &GlobalFlags) -> Result<()> {
    let auth = crate::client::build_client(gog_auth::Service::Keep, flags).await?;
    let note = gog_keep::get::get_note(&auth.client, &auth.access_token, &args.id).await?;
    crate::client::output_json(flags, &note)
}
