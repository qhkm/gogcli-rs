// Calendar command.
// Mirrors: internal/cmd/calendar.go

use clap::{Parser, Subcommand};
use anyhow::Result;

use gog_auth::Service;
use gog_calendar::{
    list_events, ListParams,
    create_event, CreateParams,
    delete_event,
    query_freebusy,
    list_calendars,
    EventDateTime,
};

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

async fn execute_list(args: &CalendarListArgs, flags: &GlobalFlags) -> Result<()> {
    let auth = crate::client::build_client(Service::Calendar, flags).await?;
    let params = ListParams {
        calendar_id: args.calendar.clone(),
        time_min: args.from.clone(),
        time_max: args.to.clone(),
        max_results: Some(args.max_results),
        query: args.query.clone(),
        page_token: None,
        single_events: true,
        order_by: Some("startTime".to_string()),
    };
    let result = list_events(&auth.client, &auth.access_token, &params).await?;
    crate::client::output(flags, &result, || {
        let mut rows = vec![vec![
            "ID".into(), "START".into(), "END".into(), "SUMMARY".into(),
        ]];
        for e in &result.items {
            rows.push(vec![
                e.id.clone().unwrap_or_default(),
                event_time_str(e.start.as_ref()),
                event_time_str(e.end.as_ref()),
                e.summary.clone().unwrap_or_default(),
            ]);
        }
        rows
    })?;
    Ok(())
}

async fn execute_create(args: &CalendarCreateArgs, flags: &GlobalFlags) -> Result<()> {
    let auth = crate::client::build_client(Service::Calendar, flags).await?;

    // Convert optional string times to EventDateTime.
    // If no time is provided, default to an all-day event starting today.
    let now = chrono::Utc::now();
    let start = match &args.start {
        Some(s) => EventDateTime::date_time(s, None),
        None => EventDateTime::date_only(&now.format("%Y-%m-%d").to_string()),
    };
    let end = match &args.end {
        Some(e) => EventDateTime::date_time(e, None),
        None => {
            // Default end: one hour after now for timed events, next day for all-day.
            if args.start.is_none() {
                let tomorrow = now + chrono::Duration::days(1);
                EventDateTime::date_only(&tomorrow.format("%Y-%m-%d").to_string())
            } else {
                let one_hour_later = now + chrono::Duration::hours(1);
                EventDateTime::date_time(&one_hour_later.to_rfc3339(), None)
            }
        }
    };

    let params = CreateParams {
        calendar_id: args.calendar.clone(),
        summary: args.title.clone(),
        description: args.description.clone(),
        location: args.location.clone(),
        start,
        end,
        attendees: args.attendees.clone(),
        recurrence: vec![],
    };

    if flags.dry_run {
        println!("Would create event: {}", args.title);
        return Ok(());
    }

    let event = create_event(&auth.client, &auth.access_token, &params).await?;
    crate::client::output_json(flags, &event)?;
    Ok(())
}

async fn execute_delete(args: &CalendarDeleteArgs, flags: &GlobalFlags) -> Result<()> {
    let auth = crate::client::build_client(Service::Calendar, flags).await?;

    if flags.dry_run {
        println!("Would delete event: {}", args.event_id);
        return Ok(());
    }

    delete_event(&auth.client, &auth.access_token, &args.calendar, &args.event_id).await?;
    println!("Deleted event {}", args.event_id);
    Ok(())
}

async fn execute_freebusy(args: &CalendarFreeBusyArgs, flags: &GlobalFlags) -> Result<()> {
    let auth = crate::client::build_client(Service::Calendar, flags).await?;

    let time_min = args.from.clone().unwrap_or_else(|| chrono::Utc::now().to_rfc3339());
    let time_max = args.to.clone().unwrap_or_else(|| {
        (chrono::Utc::now() + chrono::Duration::days(7)).to_rfc3339()
    });

    let calendars: Vec<String> = if args.emails.is_empty() {
        vec![auth.email.clone()]
    } else {
        args.emails.clone()
    };

    let result = query_freebusy(&auth.client, &auth.access_token, &calendars, &time_min, &time_max).await?;
    crate::client::output_json(flags, &result)?;
    Ok(())
}

async fn execute_calendars(_args: &CalendarCalendarsArgs, flags: &GlobalFlags) -> Result<()> {
    let auth = crate::client::build_client(Service::Calendar, flags).await?;
    let calendars = list_calendars(&auth.client, &auth.access_token).await?;
    crate::client::output(flags, &calendars, || {
        let mut rows = vec![vec!["ID".into(), "SUMMARY".into()]];
        for c in &calendars {
            rows.push(vec![
                c.id.clone(),
                c.summary.clone().unwrap_or_default(),
            ]);
        }
        rows
    })?;
    Ok(())
}

fn event_time_str(dt: Option<&EventDateTime>) -> String {
    match dt {
        Some(edt) => edt.date_time.clone()
            .or_else(|| edt.date.clone())
            .unwrap_or_default(),
        None => String::new(),
    }
}
