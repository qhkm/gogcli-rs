// gog-calendar
// Google Calendar API client library.
// Ported from the Go gogcli calendar package.

pub mod calendars;
pub mod create;
pub mod delete;
pub mod freebusy;
pub mod list;
pub mod types;

pub use calendars::list_calendars;
pub use create::{create_event, CreateParams};
pub use delete::delete_event;
pub use freebusy::query_freebusy;
pub use list::{list_events, ListParams};
pub use types::{
    Attendee, CalendarError, CalendarList, CalendarListEntry, Event, EventDateTime, EventList,
    EventPerson, FreeBusyResponse, ReminderOverride, Reminders,
};
