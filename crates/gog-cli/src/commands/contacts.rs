// Contacts command.
// Mirrors: internal/cmd/contacts.go

use clap::{Parser, Subcommand};
use anyhow::Result;

use gog_auth::Service;
use gog_contacts::types::{EmailAddress, Name, Person, PhoneNumber};

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

async fn execute_search(args: &ContactsSearchArgs, flags: &GlobalFlags) -> Result<()> {
    let auth = crate::client::build_client(Service::Contacts, flags).await?;

    // The Contacts API functions use auth.client which already has the Bearer
    // header set as a default header.
    let persons = gog_contacts::search::search_contacts(&auth.client, &args.query).await?;
    crate::client::output_json(flags, &persons)
}

async fn execute_create(args: &ContactsCreateArgs, flags: &GlobalFlags) -> Result<()> {
    let auth = crate::client::build_client(Service::Contacts, flags).await?;

    // Build the Name entry when at least one name component is supplied.
    let names = if args.first_name.is_some() || args.last_name.is_some() {
        let given_name = args.first_name.clone().unwrap_or_default();
        let family_name = args.last_name.clone().unwrap_or_default();
        let display_name = format!("{} {}", given_name, family_name).trim().to_string();
        vec![Name {
            given_name,
            family_name,
            display_name,
        }]
    } else {
        vec![]
    };

    let email_addresses = match &args.email {
        Some(email) => vec![EmailAddress {
            value: email.clone(),
            type_: "other".to_string(),
            display_name: String::new(),
        }],
        None => vec![],
    };

    let phone_numbers = match &args.phone {
        Some(phone) => vec![PhoneNumber {
            value: phone.clone(),
            type_: "mobile".to_string(),
        }],
        None => vec![],
    };

    let person = Person {
        names,
        email_addresses,
        phone_numbers,
        ..Default::default()
    };

    let created = gog_contacts::create::create_contact(&auth.client, &person).await?;
    crate::client::output_json(flags, &created)
}

async fn execute_update(args: &ContactsUpdateArgs, flags: &GlobalFlags) -> Result<()> {
    let auth = crate::client::build_client(Service::Contacts, flags).await?;

    // Fetch the existing contact first so we have the current etag and fields.
    let mut person =
        gog_contacts::search::get_contact(&auth.client, &args.resource_name).await?;

    // Merge name updates.
    if args.first_name.is_some() || args.last_name.is_some() {
        let existing = person.names.first().cloned().unwrap_or_default();
        let given_name = args
            .first_name
            .clone()
            .unwrap_or_else(|| existing.given_name.clone());
        let family_name = args
            .last_name
            .clone()
            .unwrap_or_else(|| existing.family_name.clone());
        let display_name = format!("{} {}", given_name, family_name).trim().to_string();
        person.names = vec![Name {
            given_name,
            family_name,
            display_name,
        }];
    }

    // Merge email update: replace the first entry (or append if none exist).
    if let Some(email) = &args.email {
        if let Some(first) = person.email_addresses.first_mut() {
            first.value = email.clone();
        } else {
            person.email_addresses.push(EmailAddress {
                value: email.clone(),
                type_: "other".to_string(),
                display_name: String::new(),
            });
        }
    }

    // Merge phone update: replace the first entry (or append if none exist).
    if let Some(phone) = &args.phone {
        if let Some(first) = person.phone_numbers.first_mut() {
            first.value = phone.clone();
        } else {
            person.phone_numbers.push(PhoneNumber {
                value: phone.clone(),
                type_: "mobile".to_string(),
            });
        }
    }

    let updated = gog_contacts::update::update_contact(&auth.client, &person, None).await?;
    crate::client::output_json(flags, &updated)
}

async fn execute_delete(args: &ContactsDeleteArgs, flags: &GlobalFlags) -> Result<()> {
    let auth = crate::client::build_client(Service::Contacts, flags).await?;

    gog_contacts::delete::delete_contact(&auth.client, &args.resource_name).await?;

    let result = serde_json::json!({
        "deleted": true,
        "resource_name": args.resource_name,
    });
    crate::client::output_json(flags, &result)
}

async fn execute_groups(args: &ContactsGroupsArgs, flags: &GlobalFlags) -> Result<()> {
    let auth = crate::client::build_client(Service::Contacts, flags).await?;

    let group_list = gog_contacts::groups::list_contact_groups(
        &auth.client,
        None,
        Some(args.max_results),
    )
    .await?;

    crate::client::output_json(flags, &group_list)
}
