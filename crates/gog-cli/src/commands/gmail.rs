// Gmail command.
// Mirrors: internal/cmd/gmail.go

use clap::{Parser, Subcommand};
use anyhow::Result;

use gog_auth::Service;
use gog_gmail::search::{search_messages, SearchParams};
use gog_gmail::get::{get_message, MessageFormat};
use gog_gmail::send::{send_message, SendParams};
use gog_gmail::labels::list_labels;
use gog_gmail::thread::{get_thread, list_threads};

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

async fn execute_search(args: &GmailSearchArgs, flags: &GlobalFlags) -> Result<()> {
    let auth = crate::client::build_client(Service::Gmail, flags).await?;
    let params = SearchParams {
        query: args.query.clone(),
        max_results: Some(args.max_results),
        page_token: args.page_token.clone(),
        label_ids: vec![],
    };
    let result = search_messages(&auth.client, &auth.access_token, &params).await?;
    crate::client::output_json(flags, &result)?;
    Ok(())
}

async fn execute_get(args: &GmailGetArgs, flags: &GlobalFlags) -> Result<()> {
    let auth = crate::client::build_client(Service::Gmail, flags).await?;
    let format = if args.raw { MessageFormat::Raw } else { MessageFormat::Full };
    let message = get_message(&auth.client, &auth.access_token, &args.id, format).await?;
    crate::client::output_json(flags, &message)?;
    Ok(())
}

async fn execute_send(args: &GmailSendArgs, flags: &GlobalFlags) -> Result<()> {
    if args.to.is_empty() {
        anyhow::bail!("at least one recipient is required (--to)");
    }
    let auth = crate::client::build_client(Service::Gmail, flags).await?;
    let params = SendParams {
        to: args.to.clone(),
        cc: args.cc.clone(),
        bcc: args.bcc.clone(),
        subject: args.subject.clone().unwrap_or_default(),
        body: args.body.clone().unwrap_or_default(),
        html: false,
        reply_to: args.reply_to.clone(),
        thread_id: None,
    };

    if flags.dry_run {
        println!("Would send email:");
        println!("  To: {}", args.to.join(", "));
        if !args.cc.is_empty() { println!("  CC: {}", args.cc.join(", ")); }
        if !args.bcc.is_empty() { println!("  BCC: {}", args.bcc.join(", ")); }
        println!("  Subject: {}", params.subject);
        return Ok(());
    }

    let message = send_message(&auth.client, &auth.access_token, &auth.email, &params).await?;
    crate::client::output_json(flags, &message)?;
    Ok(())
}

async fn execute_labels(args: &GmailLabelsArgs, flags: &GlobalFlags) -> Result<()> {
    let auth = crate::client::build_client(Service::Gmail, flags).await?;
    let labels = list_labels(&auth.client, &auth.access_token).await?;

    if let Some(ref filter) = args.filter {
        let filter_lower = filter.to_lowercase();
        let filtered: Vec<_> = labels
            .into_iter()
            .filter(|l| l.name.to_lowercase().contains(&filter_lower))
            .collect();
        crate::client::output_json(flags, &filtered)?;
    } else {
        crate::client::output_json(flags, &labels)?;
    }
    Ok(())
}

async fn execute_thread(args: &GmailThreadArgs, flags: &GlobalFlags) -> Result<()> {
    let auth = crate::client::build_client(Service::Gmail, flags).await?;

    match &args.id {
        Some(thread_id) => {
            let thread = get_thread(&auth.client, &auth.access_token, thread_id).await?;
            crate::client::output_json(flags, &thread)?;
        }
        None => {
            let threads = list_threads(&auth.client, &auth.access_token, "", Some(20), None).await?;
            crate::client::output_json(flags, &threads)?;
        }
    }
    Ok(())
}
