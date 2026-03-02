// Auth command - Auth and credentials management.
// Mirrors: internal/cmd/auth.go

use anyhow::{bail, Result};
use clap::{Parser, Subcommand};

use gog_auth::scopes::{scopes_for_services, Service};
use gog_core::config;
use gog_secrets::Token;

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
pub async fn execute(cmd: &AuthCmd, flags: &GlobalFlags) -> Result<()> {
    match &cmd.subcommand {
        AuthSubcommand::Add(args) => execute_add(args, flags).await,
        AuthSubcommand::Remove(args) => execute_remove(args, flags).await,
        AuthSubcommand::Status(args) => execute_status(args, flags).await,
        AuthSubcommand::List(args) => execute_list(args, flags).await,
    }
}

async fn execute_add(args: &AuthAddArgs, flags: &GlobalFlags) -> Result<()> {
    // Resolve services to authorize
    let services: Vec<Service> = if args.services.is_empty() {
        Service::user_services()
    } else {
        args.services
            .iter()
            .map(|s| Service::parse(s))
            .collect::<Result<Vec<_>, _>>()?
    };

    // Read client credentials
    let creds = config::read_client_credentials_for(&flags.client)?;

    // Build scopes
    let scopes = scopes_for_services(&services);

    // Run the browser OAuth flow (starts local server, opens browser, waits for callback)
    let timeout = std::time::Duration::from_secs(120);
    let (_oauth, token_resp) = gog_auth::oauth::authorize_with_browser(
        creds.client_id,
        creds.client_secret,
        scopes.clone(),
        timeout,
    )
    .await?;

    let refresh_token = token_resp
        .refresh_token
        .ok_or_else(|| anyhow::anyhow!("no refresh token in response (account may already be authorized; revoke and re-authorize)"))?;

    // Determine email
    let email = match &args.email {
        Some(e) => e.clone(),
        None => {
            println!("Enter the account email:");
            let mut email_input = String::new();
            std::io::stdin().read_line(&mut email_input)?;
            let email_input = email_input.trim().to_string();
            if email_input.is_empty() {
                bail!("no email provided");
            }
            email_input
        }
    };

    // Store token
    let store = gog_secrets::open_store()?;
    let service_names: Vec<String> = services.iter().map(|s| s.as_str().to_string()).collect();
    let token = Token {
        client: flags.client.clone(),
        email: email.clone(),
        services: service_names,
        scopes,
        created_at: chrono::Utc::now(),
        refresh_token,
    };
    store.set_token(&flags.client, &email, &token)?;
    store.set_default_account(&flags.client, &email)?;

    println!("Authorized {} (client: {})", email, flags.client);
    Ok(())
}

async fn execute_remove(args: &AuthRemoveArgs, flags: &GlobalFlags) -> Result<()> {
    let email = match &args.email {
        Some(e) => e.clone(),
        None => crate::client::require_account(flags)?,
    };

    let store = gog_secrets::open_store()?;
    store.delete_token(&flags.client, &email)?;

    println!(
        "Removed credentials for {} (client: {})",
        email, flags.client
    );
    Ok(())
}

async fn execute_status(args: &AuthStatusArgs, flags: &GlobalFlags) -> Result<()> {
    let email = match &args.email {
        Some(e) => e.clone(),
        None => crate::client::require_account(flags)?,
    };

    let store = gog_secrets::open_store()?;
    match store.get_token(&flags.client, &email) {
        Ok(token) => {
            if flags.json {
                crate::client::output_json(flags, &token)?;
            } else {
                println!("Account:  {}", token.email);
                println!("Client:   {}", token.client);
                println!("Created:  {}", token.created_at);
                if !token.services.is_empty() {
                    println!("Services: {}", token.services.join(", "));
                }
                if !token.scopes.is_empty() {
                    println!("Scopes:   {} scope(s)", token.scopes.len());
                }
            }
        }
        Err(e) => {
            bail!(
                "no credentials found for {} (client: {}): {}",
                email,
                flags.client,
                e
            );
        }
    }
    Ok(())
}

async fn execute_list(_args: &AuthListArgs, flags: &GlobalFlags) -> Result<()> {
    let store = gog_secrets::open_store()?;
    let tokens = store.list_tokens()?;

    if flags.json {
        crate::client::output_json(flags, &tokens)?;
    } else if tokens.is_empty() {
        println!("No authenticated accounts found.");
        println!("Run 'gog auth add' to authorize an account.");
    } else {
        for token in &tokens {
            println!(
                "{} (client: {}, services: {})",
                token.email,
                token.client,
                if token.services.is_empty() {
                    "all".to_string()
                } else {
                    token.services.join(",")
                }
            );
        }
    }
    Ok(())
}
