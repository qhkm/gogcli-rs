// Contacts command.
// Mirrors: internal/cmd/contacts.go

use clap::{Parser, Subcommand};
use anyhow::Result;

use crate::GlobalFlags;

/// Google Contacts operations.
#[derive(Debug, Parser)]
#[command(about = "Google Contacts")]
pub struct ContactsCmd {
    #[command(subcommand)]
    pub subcommand: ContactsSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum ContactsSubcommand {
    /// Search contacts.
    #[command(aliases = ["find", "query"])]
    Search(ContactsSearchArgs),

    /// Create a new contact.
    #[command(aliases = ["add", "new"])]
    Create(ContactsCreateArgs),

    /// Update an existing contact.
    #[command(aliases = ["edit", "modify"])]
    Update(ContactsUpdateArgs),

    /// Delete a contact.
    #[command(aliases = ["rm", "remove"])]
    Delete(ContactsDeleteArgs),

    /// List contact groups.
    #[command(aliases = ["group"])]
    Groups(ContactsGroupsArgs),
}

#[derive(Debug, Parser)]
pub struct ContactsSearchArgs {
    /// Search query.
    pub query: String,

    /// Maximum number of results.
    #[arg(long, default_value = "10")]
    pub max_results: u32,
}

#[derive(Debug, Parser)]
pub struct ContactsCreateArgs {
    /// Contact's first name.
    #[arg(long)]
    pub first_name: Option<String>,

    /// Contact's last name.
    #[arg(long)]
    pub last_name: Option<String>,

    /// Contact's email address.
    #[arg(long, short = 'e')]
    pub email: Option<String>,

    /// Contact's phone number.
    #[arg(long)]
    pub phone: Option<String>,
}

#[derive(Debug, Parser)]
pub struct ContactsUpdateArgs {
    /// Contact resource name (e.g. "people/c12345").
    pub resource_name: String,

    /// New email address.
    #[arg(long, short = 'e')]
    pub email: Option<String>,

    /// New phone number.
    #[arg(long)]
    pub phone: Option<String>,

    /// New first name.
    #[arg(long)]
    pub first_name: Option<String>,

    /// New last name.
    #[arg(long)]
    pub last_name: Option<String>,
}

#[derive(Debug, Parser)]
pub struct ContactsDeleteArgs {
    /// Contact resource name.
    pub resource_name: String,
}

#[derive(Debug, Parser)]
pub struct ContactsGroupsArgs {
    /// Maximum number of groups to return.
    #[arg(long, default_value = "20")]
    pub max_results: u32,
}

/// Execute the contacts command.
pub async fn execute(cmd: &ContactsCmd, flags: &GlobalFlags) -> Result<()> {
    match &cmd.subcommand {
        ContactsSubcommand::Search(args) => execute_search(args, flags).await,
        ContactsSubcommand::Create(args) => execute_create(args, flags).await,
        ContactsSubcommand::Update(args) => execute_update(args, flags).await,
        ContactsSubcommand::Delete(args) => execute_delete(args, flags).await,
        ContactsSubcommand::Groups(args) => execute_groups(args, flags).await,
    }
}

async fn execute_search(_args: &ContactsSearchArgs, _flags: &GlobalFlags) -> Result<()> {
    todo!("implement contacts search")
}

async fn execute_create(_args: &ContactsCreateArgs, _flags: &GlobalFlags) -> Result<()> {
    todo!("implement contacts create")
}

async fn execute_update(_args: &ContactsUpdateArgs, _flags: &GlobalFlags) -> Result<()> {
    todo!("implement contacts update")
}

async fn execute_delete(_args: &ContactsDeleteArgs, _flags: &GlobalFlags) -> Result<()> {
    todo!("implement contacts delete")
}

async fn execute_groups(_args: &ContactsGroupsArgs, _flags: &GlobalFlags) -> Result<()> {
    todo!("implement contacts groups")
}
