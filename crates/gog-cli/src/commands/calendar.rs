// Calendar command.
// Mirrors: internal/cmd/calendar.go

use clap::{Parser, Subcommand};
use anyhow::Result;

use crate::GlobalFlags;

/// Google Calendar operations.
#[derive(Debug, Parser)]
#[command(about = "Google Calendar")]
pub struct CalendarCmd {
    #[command(subcommand)]
    pub subcommand: CalendarSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum CalendarSubcommand {
    /// List calendar events.
    #[command(aliases = ["events", "ls"])]
    List(CalendarListArgs),

    /// Create a calendar event.
    #[command(aliases = ["add", "new"])]
    Create(CalendarCreateArgs),

    /// Delete a calendar event.
    #[command(aliases = ["rm", "remove"])]
    Delete(CalendarDeleteArgs),

    /// Query free/busy times.
    FreeBusy(CalendarFreeBusyArgs),

    /// List all calendars.
    #[command(name = "calendars", aliases = ["cals"])]
    Calendars(CalendarCalendarsArgs),
}

#[derive(Debug, Parser)]
pub struct CalendarListArgs {
    /// Calendar ID (defaults to "primary").
    #[arg(long, default_value = "primary")]
    pub calendar: String,

    /// Start time (ISO 8601 or relative like "today", "tomorrow").
    #[arg(long)]
    pub from: Option<String>,

    /// End time (ISO 8601 or relative).
    #[arg(long)]
    pub to: Option<String>,

    /// Maximum number of events to return.
    #[arg(long, default_value = "10")]
    pub max_results: u32,

    /// Filter by search query.
    #[arg(long, short = 'q')]
    pub query: Option<String>,
}

#[derive(Debug, Parser)]
pub struct CalendarCreateArgs {
    /// Event summary/title.
    pub title: String,

    /// Start time.
    #[arg(long)]
    pub start: Option<String>,

    /// End time.
    #[arg(long)]
    pub end: Option<String>,

    /// Event description.
    #[arg(long, short = 'd')]
    pub description: Option<String>,

    /// Event location.
    #[arg(long, short = 'l')]
    pub location: Option<String>,

    /// Attendee email(s).
    #[arg(long, value_delimiter = ',')]
    pub attendees: Vec<String>,

    /// Calendar ID (defaults to "primary").
    #[arg(long, default_value = "primary")]
    pub calendar: String,
}

#[derive(Debug, Parser)]
pub struct CalendarDeleteArgs {
    /// Event ID to delete.
    pub event_id: String,

    /// Calendar ID (defaults to "primary").
    #[arg(long, default_value = "primary")]
    pub calendar: String,
}

#[derive(Debug, Parser)]
pub struct CalendarFreeBusyArgs {
    /// Start time.
    #[arg(long)]
    pub from: Option<String>,

    /// End time.
    #[arg(long)]
    pub to: Option<String>,

    /// Email(s) to query free/busy for.
    #[arg(value_delimiter = ',')]
    pub emails: Vec<String>,
}

#[derive(Debug, Parser)]
pub struct CalendarCalendarsArgs {
    /// Include hidden calendars.
    #[arg(long)]
    pub show_hidden: bool,
}

/// Execute the calendar command.
pub async fn execute(cmd: &CalendarCmd, flags: &GlobalFlags) -> Result<()> {
    match &cmd.subcommand {
        CalendarSubcommand::List(args) => execute_list(args, flags).await,
        CalendarSubcommand::Create(args) => execute_create(args, flags).await,
        CalendarSubcommand::Delete(args) => execute_delete(args, flags).await,
        CalendarSubcommand::FreeBusy(args) => execute_freebusy(args, flags).await,
        CalendarSubcommand::Calendars(args) => execute_calendars(args, flags).await,
    }
}

async fn execute_list(_args: &CalendarListArgs, _flags: &GlobalFlags) -> Result<()> {
    todo!("implement calendar list")
}

async fn execute_create(_args: &CalendarCreateArgs, _flags: &GlobalFlags) -> Result<()> {
    todo!("implement calendar create")
}

async fn execute_delete(_args: &CalendarDeleteArgs, _flags: &GlobalFlags) -> Result<()> {
    todo!("implement calendar delete")
}

async fn execute_freebusy(_args: &CalendarFreeBusyArgs, _flags: &GlobalFlags) -> Result<()> {
    todo!("implement calendar freebusy")
}

async fn execute_calendars(_args: &CalendarCalendarsArgs, _flags: &GlobalFlags) -> Result<()> {
    todo!("implement calendar calendars")
}
