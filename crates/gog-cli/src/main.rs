// gog - Google Workspace CLI
// Rust port of the Go gogcli tool.
// Binary entry point with clap v4 derive-macro CLI definition.

mod commands;
mod client;
mod error;
mod output;

use std::process;

use anyhow::Result;
use clap::{Parser, Subcommand};

use commands::{auth, calendar, chat, completion, config, contacts, drive, forms, gmail, keep};

// ---------------------------------------------------------------------------
// Version
// ---------------------------------------------------------------------------

const VERSION: &str = env!("CARGO_PKG_VERSION");

// ---------------------------------------------------------------------------
// Global flags
// ---------------------------------------------------------------------------

/// Global flags shared across all subcommands.
#[derive(Debug, Clone, Parser)]
pub struct GlobalFlags {
    /// Account email for API commands.
    #[arg(long, short = 'a', global = true, env = "GOG_ACCOUNT")]
    pub account: Option<String>,

    /// OAuth client name (selects stored credentials + token bucket).
    #[arg(long, global = true, env = "GOG_CLIENT", default_value = "default")]
    pub client: String,

    /// Output JSON to stdout (best for scripting).
    #[arg(long, short = 'j', global = true, env = "GOG_JSON")]
    pub json: bool,

    /// Output stable, parseable text to stdout (TSV; no colors).
    #[arg(long, short = 'p', global = true, env = "GOG_PLAIN")]
    pub plain: bool,

    /// In JSON mode, emit only the primary result (drops envelope fields like nextPageToken).
    #[arg(long, global = true)]
    pub results_only: bool,

    /// In JSON mode, select comma-separated fields (supports dot paths).
    #[arg(long, global = true)]
    pub select: Option<String>,

    /// Do not make changes; print intended actions and exit successfully.
    #[arg(long, short = 'n', global = true)]
    pub dry_run: bool,

    /// Skip confirmations for destructive commands.
    #[arg(long, short = 'y', global = true)]
    pub force: bool,

    /// Never prompt; fail instead (useful for CI).
    #[arg(long, global = true)]
    pub no_input: bool,

    /// Enable verbose logging.
    #[arg(long, short = 'v', global = true)]
    pub verbose: bool,

    /// Color output: auto|always|never.
    #[arg(long, global = true, default_value = "auto", env = "GOG_COLOR")]
    pub color: String,
}

// ---------------------------------------------------------------------------
// Top-level CLI
// ---------------------------------------------------------------------------

/// gog - Google Workspace CLI for Gmail, Calendar, Drive, Contacts, Chat, Keep, and Forms.
#[derive(Debug, Parser)]
#[command(
    name = "gog",
    version = VERSION,
    about = "Google Workspace CLI",
    long_about = "Google CLI for Gmail/Calendar/Drive/Contacts/Chat/Keep/Forms\n\nUse 'gog <command> --help' for command-specific help.",
)]
pub struct Cli {
    #[command(flatten)]
    pub global: GlobalFlags,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    // -----------------------------------------------------------------------
    // Service commands
    // -----------------------------------------------------------------------

    /// Auth and credentials.
    Auth(auth::AuthCmd),

    /// Gmail.
    #[command(visible_aliases = &["mail", "email"])]
    Gmail(gmail::GmailCmd),

    /// Google Calendar.
    #[command(name = "calendar", visible_aliases = &["cal"])]
    Calendar(calendar::CalendarCmd),

    /// Google Drive.
    #[command(name = "drive", visible_aliases = &["drv"])]
    Drive(drive::DriveCmd),

    /// Google Contacts.
    #[command(name = "contacts", visible_aliases = &["contact"])]
    Contacts(contacts::ContactsCmd),

    /// Google Chat.
    Chat(chat::ChatCmd),

    /// Google Keep (Workspace only).
    Keep(keep::KeepCmd),

    /// Google Forms.
    #[command(name = "forms", visible_aliases = &["form"])]
    Forms(forms::FormsCmd),

    // -----------------------------------------------------------------------
    // Utility commands
    // -----------------------------------------------------------------------

    /// Print version.
    Version(VersionArgs),

    /// Show/edit config.
    Config(config::ConfigCmd),

    /// Generate shell completions.
    Completion(completion::CompletionCmd),

    // -----------------------------------------------------------------------
    // Desire-path aliases (top-level shortcuts)
    // -----------------------------------------------------------------------

    /// Send an email (alias for 'gmail send').
    #[command(hide = true)]
    Send(gmail::GmailSendArgs),

    /// Authorize and store a refresh token (alias for 'auth add').
    #[command(hide = true)]
    Login(auth::AuthAddArgs),

    /// Remove a stored refresh token (alias for 'auth remove').
    #[command(hide = true)]
    Logout(auth::AuthRemoveArgs),

    /// Show auth/config status (alias for 'auth status').
    #[command(hide = true, aliases = &["st"])]
    Status(auth::AuthStatusArgs),
}

// ---------------------------------------------------------------------------
// Version command
// ---------------------------------------------------------------------------

#[derive(Debug, Parser)]
pub struct VersionArgs {}

fn print_version() {
    println!("gog version {VERSION}");
}

// ---------------------------------------------------------------------------
// Main entry point
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = run(cli).await;
    match result {
        Ok(()) => {}
        Err(err) => {
            let code = error::report_error(&err);
            process::exit(code);
        }
    }
}

async fn run(cli: Cli) -> Result<()> {
    let flags = &cli.global;

    match cli.command {
        Command::Auth(cmd) => auth::execute(&cmd, flags).await,
        Command::Gmail(cmd) => gmail::execute(&cmd, flags).await,
        Command::Calendar(cmd) => calendar::execute(&cmd, flags).await,
        Command::Drive(cmd) => drive::execute(&cmd, flags).await,
        Command::Contacts(cmd) => contacts::execute(&cmd, flags).await,
        Command::Chat(cmd) => chat::execute(&cmd, flags).await,
        Command::Keep(cmd) => keep::execute(&cmd, flags).await,
        Command::Forms(cmd) => forms::execute(&cmd, flags).await,
        Command::Config(cmd) => config::execute(&cmd, flags).await,

        Command::Version(_) => {
            print_version();
            Ok(())
        }

        Command::Completion(cmd) => {
            cmd.execute();
            Ok(())
        }

        // Desire-path aliases - delegate to service commands
        Command::Send(args) => {
            let cmd = gmail::GmailCmd {
                subcommand: gmail::GmailSubcommand::Send(args),
            };
            gmail::execute(&cmd, flags).await
        }

        Command::Login(args) => {
            let cmd = auth::AuthCmd {
                subcommand: auth::AuthSubcommand::Add(args),
            };
            auth::execute(&cmd, flags).await
        }

        Command::Logout(args) => {
            let cmd = auth::AuthCmd {
                subcommand: auth::AuthSubcommand::Remove(args),
            };
            auth::execute(&cmd, flags).await
        }

        Command::Status(args) => {
            let cmd = auth::AuthCmd {
                subcommand: auth::AuthSubcommand::Status(args),
            };
            auth::execute(&cmd, flags).await
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    // Verify the CLI structure is valid (clap's built-in verification).
    fn get_cmd() -> clap::Command {
        Cli::command()
    }

    #[test]
    fn test_cli_version_flag() {
        // --version should be recognized by clap (it's a built-in flag).
        let cmd = get_cmd();
        // Verify the command has a version set.
        assert_eq!(cmd.get_version(), Some(VERSION));
    }

    #[test]
    fn test_cli_help_output() {
        // Parse --help and verify it doesn't panic.
        // We use try_parse to avoid process::exit.
        let result = Cli::try_parse_from(["gog", "--help"]);
        // --help returns an Err with clap's DisplayHelp error.
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), clap::error::ErrorKind::DisplayHelp);
    }

    #[test]
    fn test_cli_gmail_parse() {
        // Verify "gmail search <query>" parses correctly.
        let cli = Cli::try_parse_from(["gog", "gmail", "search", "from:alice@example.com"])
            .expect("gmail search should parse");

        match cli.command {
            Command::Gmail(gmail_cmd) => {
                match gmail_cmd.subcommand {
                    gmail::GmailSubcommand::Search(args) => {
                        assert_eq!(args.query, "from:alice@example.com");
                    }
                    other => panic!("expected GmailSubcommand::Search, got: {other:?}"),
                }
            }
            other => panic!("expected Command::Gmail, got: {other:?}"),
        }
    }

    #[test]
    fn test_cli_global_flags() {
        // Verify --json and --account parse correctly.
        let cli = Cli::try_parse_from([
            "gog",
            "--json",
            "--account",
            "user@example.com",
            "version",
        ])
        .expect("global flags should parse");

        assert!(cli.global.json);
        assert_eq!(cli.global.account.as_deref(), Some("user@example.com"));
    }

    #[test]
    fn test_cli_aliases_cal() {
        // Verify "cal list" maps to calendar list.
        let cli = Cli::try_parse_from(["gog", "cal", "list"])
            .expect("cal alias should parse");

        assert!(matches!(cli.command, Command::Calendar(_)));
    }

    #[test]
    fn test_cli_aliases_mail() {
        // Verify "mail search <query>" maps to gmail search.
        let cli = Cli::try_parse_from(["gog", "mail", "search", "is:unread"])
            .expect("mail alias should parse");

        assert!(matches!(cli.command, Command::Gmail(_)));
    }

    #[test]
    fn test_cli_aliases_email() {
        // Verify "email search <query>" maps to gmail search.
        let cli = Cli::try_parse_from(["gog", "email", "search", "is:starred"])
            .expect("email alias should parse");

        assert!(matches!(cli.command, Command::Gmail(_)));
    }

    #[test]
    fn test_cli_aliases_drv() {
        // Verify "drv ls" maps to drive ls.
        let cli = Cli::try_parse_from(["gog", "drv", "ls"])
            .expect("drv alias should parse");

        assert!(matches!(cli.command, Command::Drive(_)));
    }

    #[test]
    fn test_cli_desire_path_send() {
        // Verify top-level "send --to ... --subject ..." parses.
        let cli = Cli::try_parse_from([
            "gog",
            "send",
            "--to",
            "bob@example.com",
            "--subject",
            "Hello",
        ])
        .expect("desire-path 'send' should parse");

        assert!(matches!(cli.command, Command::Send(_)));
    }

    #[test]
    fn test_cli_desire_path_login() {
        // Verify top-level "login" parses.
        let cli = Cli::try_parse_from(["gog", "login"])
            .expect("desire-path 'login' should parse");

        assert!(matches!(cli.command, Command::Login(_)));
    }

    #[test]
    fn test_cli_desire_path_logout() {
        // Verify top-level "logout" parses.
        let cli = Cli::try_parse_from(["gog", "logout"])
            .expect("desire-path 'logout' should parse");

        assert!(matches!(cli.command, Command::Logout(_)));
    }

    #[test]
    fn test_cli_desire_path_status() {
        // Verify top-level "status" parses.
        let cli = Cli::try_parse_from(["gog", "status"])
            .expect("desire-path 'status' should parse");

        assert!(matches!(cli.command, Command::Status(_)));
    }

    #[test]
    fn test_cli_calendar_subcommands() {
        // calendar create
        let cli = Cli::try_parse_from(["gog", "calendar", "create", "Team meeting"])
            .expect("calendar create should parse");

        match cli.command {
            Command::Calendar(cmd) => {
                assert!(matches!(cmd.subcommand, calendar::CalendarSubcommand::Create(_)));
            }
            other => panic!("expected Calendar, got: {other:?}"),
        }
    }

    #[test]
    fn test_cli_drive_subcommands() {
        // drive search
        let cli = Cli::try_parse_from(["gog", "drive", "search", "my document"])
            .expect("drive search should parse");

        match cli.command {
            Command::Drive(cmd) => {
                assert!(matches!(cmd.subcommand, drive::DriveSubcommand::Search(_)));
            }
            other => panic!("expected Drive, got: {other:?}"),
        }
    }

    #[test]
    fn test_cli_dry_run_flag() {
        let cli = Cli::try_parse_from(["gog", "--dry-run", "version"])
            .expect("--dry-run should parse");

        assert!(cli.global.dry_run);
    }

    #[test]
    fn test_cli_force_flag() {
        let cli = Cli::try_parse_from(["gog", "--force", "version"])
            .expect("--force should parse");

        assert!(cli.global.force);
    }

    #[test]
    fn test_cli_verbose_flag() {
        let cli = Cli::try_parse_from(["gog", "--verbose", "version"])
            .expect("--verbose should parse");

        assert!(cli.global.verbose);
    }

    #[test]
    fn test_cli_plain_flag() {
        let cli = Cli::try_parse_from(["gog", "--plain", "version"])
            .expect("--plain should parse");

        assert!(cli.global.plain);
    }

    #[test]
    fn test_cli_client_default() {
        let cli = Cli::try_parse_from(["gog", "version"])
            .expect("version should parse");

        assert_eq!(cli.global.client, "default");
    }

    #[test]
    fn test_cli_client_override() {
        let cli = Cli::try_parse_from(["gog", "--client", "work", "version"])
            .expect("--client work should parse");

        assert_eq!(cli.global.client, "work");
    }

    #[test]
    fn test_cli_select_flag() {
        let cli = Cli::try_parse_from([
            "gog",
            "--select",
            "id,subject",
            "gmail",
            "search",
            "is:unread",
        ])
        .expect("--select should parse");

        assert_eq!(cli.global.select.as_deref(), Some("id,subject"));
    }

    #[test]
    fn test_cli_results_only_flag() {
        let cli = Cli::try_parse_from(["gog", "--results-only", "--json", "gmail", "search", "test"])
            .expect("--results-only should parse");

        assert!(cli.global.results_only);
        assert!(cli.global.json);
    }

    #[test]
    fn test_cli_no_input_flag() {
        let cli = Cli::try_parse_from(["gog", "--no-input", "version"])
            .expect("--no-input should parse");

        assert!(cli.global.no_input);
    }

    #[test]
    fn test_cli_color_flag() {
        let cli = Cli::try_parse_from(["gog", "--color", "never", "version"])
            .expect("--color never should parse");

        assert_eq!(cli.global.color, "never");
    }

    #[test]
    fn test_cli_auth_subcommands() {
        // auth add
        let cli = Cli::try_parse_from(["gog", "auth", "add", "user@example.com"])
            .expect("auth add should parse");

        match cli.command {
            Command::Auth(cmd) => {
                match cmd.subcommand {
                    auth::AuthSubcommand::Add(args) => {
                        assert_eq!(args.email.as_deref(), Some("user@example.com"));
                    }
                    other => panic!("expected Add, got: {other:?}"),
                }
            }
            other => panic!("expected Auth, got: {other:?}"),
        }
    }

    #[test]
    fn test_cli_auth_login_alias() {
        // auth login (alias for auth add)
        let cli = Cli::try_parse_from(["gog", "auth", "login"])
            .expect("auth login should parse");

        assert!(matches!(
            cli.command,
            Command::Auth(auth::AuthCmd {
                subcommand: auth::AuthSubcommand::Add(_),
                ..
            })
        ));
    }

    #[test]
    fn test_cli_keep_subcommands() {
        let cli = Cli::try_parse_from(["gog", "keep", "list"])
            .expect("keep list should parse");

        match cli.command {
            Command::Keep(cmd) => {
                assert!(matches!(cmd.subcommand, keep::KeepSubcommand::List(_)));
            }
            other => panic!("expected Keep, got: {other:?}"),
        }
    }

    #[test]
    fn test_cli_forms_subcommands() {
        let cli = Cli::try_parse_from(["gog", "forms", "get", "1BxiMVs0XRA5nFMdKvBdBZjgmUUqptlbs74OgVE2upms"])
            .expect("forms get should parse");

        match cli.command {
            Command::Forms(cmd) => {
                assert!(matches!(cmd.subcommand, forms::FormsSubcommand::Get(_)));
            }
            other => panic!("expected Forms, got: {other:?}"),
        }
    }

    #[test]
    fn test_cli_chat_subcommands() {
        let cli = Cli::try_parse_from(["gog", "chat", "spaces"])
            .expect("chat spaces should parse");

        match cli.command {
            Command::Chat(cmd) => {
                assert!(matches!(cmd.subcommand, chat::ChatSubcommand::Spaces(_)));
            }
            other => panic!("expected Chat, got: {other:?}"),
        }
    }

    #[test]
    fn test_cli_contacts_alias() {
        // "contact search" (singular alias)
        let cli = Cli::try_parse_from(["gog", "contact", "search", "alice"])
            .expect("contact alias should parse");

        assert!(matches!(cli.command, Command::Contacts(_)));
    }

    #[test]
    fn test_cli_form_alias() {
        // "form get" (singular alias)
        let cli = Cli::try_parse_from(["gog", "form", "get", "some-form-id"])
            .expect("form alias should parse");

        assert!(matches!(cli.command, Command::Forms(_)));
    }

    #[test]
    fn test_cli_gmail_send_args() {
        let cli = Cli::try_parse_from([
            "gog",
            "gmail",
            "send",
            "--to",
            "alice@example.com",
            "--subject",
            "Test subject",
            "--body",
            "Hello world",
        ])
        .expect("gmail send args should parse");

        match cli.command {
            Command::Gmail(cmd) => {
                match cmd.subcommand {
                    gmail::GmailSubcommand::Send(args) => {
                        assert_eq!(args.to, vec!["alice@example.com"]);
                        assert_eq!(args.subject.as_deref(), Some("Test subject"));
                        assert_eq!(args.body.as_deref(), Some("Hello world"));
                    }
                    other => panic!("expected Send, got: {other:?}"),
                }
            }
            other => panic!("expected Gmail, got: {other:?}"),
        }
    }

    #[test]
    fn test_cli_short_flags() {
        // Verify short flags work: -j for --json, -a for --account, -v for --verbose
        let cli = Cli::try_parse_from(["gog", "-j", "-a", "me@test.com", "-v", "version"])
            .expect("short flags should parse");

        assert!(cli.global.json);
        assert_eq!(cli.global.account.as_deref(), Some("me@test.com"));
        assert!(cli.global.verbose);
    }

    #[test]
    fn test_cli_gmail_search_max_results() {
        let cli = Cli::try_parse_from(["gog", "gmail", "search", "test", "--max-results", "50"])
            .expect("gmail search --max-results should parse");

        match cli.command {
            Command::Gmail(cmd) => {
                match cmd.subcommand {
                    gmail::GmailSubcommand::Search(args) => {
                        assert_eq!(args.max_results, 50);
                    }
                    other => panic!("expected Search, got: {other:?}"),
                }
            }
            other => panic!("expected Gmail, got: {other:?}"),
        }
    }

    #[test]
    fn test_output_config_from_global_flags() {
        // Verify output config construction from GlobalFlags works.
        use crate::output::make_output_config;

        let cfg = make_output_config(true, false, true, Some("id,name"))
            .expect("output config should build");

        assert!(cfg.is_json());
        assert!(cfg.results_only);
        assert_eq!(cfg.select_fields, vec!["id", "name"]);
    }

    #[test]
    fn test_output_config_conflict() {
        use crate::output::make_output_config;

        let err = make_output_config(true, true, false, None)
            .expect_err("json+plain should conflict");

        assert!(err.contains("cannot combine"));
    }
}
