// gog-calendar types module
// Google Calendar API data types.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Event types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub id: Option<String>,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub location: Option<String>,
    pub start: Option<EventDateTime>,
    pub end: Option<EventDateTime>,
    /// "confirmed", "tentative", or "cancelled"
    pub status: Option<String>,
    pub html_link: Option<String>,
    pub created: Option<String>,
    pub updated: Option<String>,
    pub creator: Option<EventPerson>,
    pub organizer: Option<EventPerson>,
    #[serde(default)]
    pub attendees: Vec<Attendee>,
    /// RRULE strings
    pub recurrence: Option<Vec<String>>,
    pub recurring_event_id: Option<String>,
    pub color_id: Option<String>,
    pub conference_data: Option<serde_json::Value>,
    #[serde(default)]
    pub reminders: Option<Reminders>,
    /// "default", "focusTime", "outOfOffice", "workingLocation"
    pub event_type: Option<String>,
    pub visibility: Option<String>,
    /// "opaque" or "transparent"
    pub transparency: Option<String>,
}

impl Event {
    /// Return the event summary or "(No title)" when missing.
    pub fn display_summary(&self) -> &str {
        self.summary
            .as_deref()
            .filter(|s| !s.is_empty())
            .unwrap_or("(No title)")
    }

    /// Return true when the event is all-day (date-only start).
    pub fn is_all_day(&self) -> bool {
        self.start.as_ref().map(|s| s.is_all_day()).unwrap_or(false)
    }

    /// Return true when conference_data is present (e.g. Google Meet).
    pub fn has_conference(&self) -> bool {
        self.conference_data.is_some()
    }
}

// ---------------------------------------------------------------------------
// EventDateTime
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventDateTime {
    /// Date-only (all-day): "2026-01-15"
    pub date: Option<String>,
    /// Date-time: "2026-01-15T10:00:00-05:00"
    pub date_time: Option<String>,
    pub time_zone: Option<String>,
}

impl EventDateTime {
    /// Create a timed EventDateTime from an RFC3339 string and optional timezone.
    pub fn date_time(dt: &str, tz: Option<&str>) -> Self {
        Self {
            date: None,
            date_time: Some(dt.to_string()),
            time_zone: tz.map(str::to_string),
        }
    }

    /// Create a date-only (all-day) EventDateTime.
    pub fn date_only(date: &str) -> Self {
        Self {
            date: Some(date.to_string()),
            date_time: None,
            time_zone: None,
        }
    }

    /// Return true when this represents an all-day event (date only, no time).
    pub fn is_all_day(&self) -> bool {
        self.date.is_some() && self.date_time.is_none()
    }
}

// ---------------------------------------------------------------------------
// People / attendees
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventPerson {
    pub email: Option<String>,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    #[serde(rename = "self")]
    pub is_self: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attendee {
    pub email: Option<String>,
    pub display_name: Option<String>,
    /// "needsAction", "accepted", "declined", or "tentative"
    pub response_status: Option<String>,
    pub organizer: Option<bool>,
    #[serde(rename = "self")]
    pub is_self: Option<bool>,
    pub optional: Option<bool>,
}

// ---------------------------------------------------------------------------
// Reminders
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reminders {
    #[serde(rename = "useDefault")]
    pub use_default: Option<bool>,
    #[serde(default)]
    pub overrides: Vec<ReminderOverride>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReminderOverride {
    /// "email" or "popup"
    pub method: String,
    pub minutes: i32,
}

// ---------------------------------------------------------------------------
// Lists
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventList {
    #[serde(default)]
    pub items: Vec<Event>,
    pub next_page_token: Option<String>,
    pub summary: Option<String>,
    pub time_zone: Option<String>,
}

// ---------------------------------------------------------------------------
// Calendar list
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarListEntry {
    pub id: String,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub time_zone: Option<String>,
    pub color_id: Option<String>,
    pub background_color: Option<String>,
    pub foreground_color: Option<String>,
    pub primary: Option<bool>,
    pub access_role: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarList {
    #[serde(default)]
    pub items: Vec<CalendarListEntry>,
    pub next_page_token: Option<String>,
}

// ---------------------------------------------------------------------------
// FreeBusy
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FreeBusyResponse {
    pub calendars: Option<serde_json::Value>,
    pub time_min: Option<String>,
    pub time_max: Option<String>,
}

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

#[derive(thiserror::Error, Debug)]
pub enum CalendarError {
    #[error("Calendar API error ({status}): {message}")]
    Api { status: u16, message: String },

    #[error("invalid time: {0}")]
    InvalidTime(String),

    #[error("missing field: {0}")]
    MissingField(String),

    #[error(transparent)]
    Http(#[from] reqwest::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // test_event_deserialize
    // -----------------------------------------------------------------------
    #[test]
    fn test_event_deserialize() {
        let json = r#"{
            "id": "abc123",
            "summary": "Team Standup",
            "description": "Daily sync",
            "location": "Room 1",
            "start": {"dateTime": "2026-01-15T09:00:00-05:00", "timeZone": "America/New_York"},
            "end":   {"dateTime": "2026-01-15T09:30:00-05:00", "timeZone": "America/New_York"},
            "status": "confirmed",
            "htmlLink": "https://calendar.google.com/event?eid=abc123",
            "created": "2026-01-01T00:00:00Z",
            "updated": "2026-01-10T08:00:00Z",
            "creator":  {"email": "alice@example.com", "displayName": "Alice", "self": true},
            "organizer":{"email": "alice@example.com", "displayName": "Alice"},
            "attendees": [
                {"email": "bob@example.com", "responseStatus": "accepted"},
                {"email": "carol@example.com", "responseStatus": "needsAction"}
            ],
            "recurrence": ["RRULE:FREQ=DAILY"],
            "colorId": "1",
            "conferenceData": {"conferenceSolution": {"key": {"type": "hangoutsMeet"}}},
            "reminders": {"useDefault": false, "overrides": [{"method": "popup", "minutes": 10}]},
            "eventType": "default",
            "visibility": "public",
            "transparency": "opaque"
        }"#;

        let event: Event = serde_json::from_str(json).expect("deserialize event");
        assert_eq!(event.id.as_deref(), Some("abc123"));
        assert_eq!(event.summary.as_deref(), Some("Team Standup"));
        assert_eq!(event.status.as_deref(), Some("confirmed"));
        assert_eq!(event.attendees.len(), 2);
        assert!(event
            .recurrence
            .as_ref()
            .map(|r| !r.is_empty())
            .unwrap_or(false));
        assert!(event.has_conference());
    }

    // -----------------------------------------------------------------------
    // test_event_all_day
    // -----------------------------------------------------------------------
    #[test]
    fn test_event_all_day() {
        let event = Event {
            id: None,
            summary: Some("Holiday".to_string()),
            description: None,
            location: None,
            start: Some(EventDateTime::date_only("2026-12-25")),
            end: Some(EventDateTime::date_only("2026-12-26")),
            status: None,
            html_link: None,
            created: None,
            updated: None,
            creator: None,
            organizer: None,
            attendees: vec![],
            recurrence: None,
            recurring_event_id: None,
            color_id: None,
            conference_data: None,
            reminders: None,
            event_type: None,
            visibility: None,
            transparency: None,
        };
        assert!(event.is_all_day(), "holiday should be all-day");
    }

    // -----------------------------------------------------------------------
    // test_event_timed
    // -----------------------------------------------------------------------
    #[test]
    fn test_event_timed() {
        let event = Event {
            id: None,
            summary: Some("Meeting".to_string()),
            description: None,
            location: None,
            start: Some(EventDateTime::date_time("2026-01-15T10:00:00Z", None)),
            end: Some(EventDateTime::date_time("2026-01-15T11:00:00Z", None)),
            status: None,
            html_link: None,
            created: None,
            updated: None,
            creator: None,
            organizer: None,
            attendees: vec![],
            recurrence: None,
            recurring_event_id: None,
            color_id: None,
            conference_data: None,
            reminders: None,
            event_type: None,
            visibility: None,
            transparency: None,
        };
        assert!(!event.is_all_day(), "timed event must not be all-day");
    }

    // -----------------------------------------------------------------------
    // test_event_display_summary
    // -----------------------------------------------------------------------
    #[test]
    fn test_event_display_summary() {
        let event = Event {
            id: None,
            summary: Some("Sprint Planning".to_string()),
            description: None,
            location: None,
            start: None,
            end: None,
            status: None,
            html_link: None,
            created: None,
            updated: None,
            creator: None,
            organizer: None,
            attendees: vec![],
            recurrence: None,
            recurring_event_id: None,
            color_id: None,
            conference_data: None,
            reminders: None,
            event_type: None,
            visibility: None,
            transparency: None,
        };
        assert_eq!(event.display_summary(), "Sprint Planning");
    }

    // -----------------------------------------------------------------------
    // test_event_display_summary_missing
    // -----------------------------------------------------------------------
    #[test]
    fn test_event_display_summary_missing() {
        let event = Event {
            id: None,
            summary: None,
            description: None,
            location: None,
            start: None,
            end: None,
            status: None,
            html_link: None,
            created: None,
            updated: None,
            creator: None,
            organizer: None,
            attendees: vec![],
            recurrence: None,
            recurring_event_id: None,
            color_id: None,
            conference_data: None,
            reminders: None,
            event_type: None,
            visibility: None,
            transparency: None,
        };
        assert_eq!(event.display_summary(), "(No title)");
    }

    // -----------------------------------------------------------------------
    // test_event_datetime_date_only
    // -----------------------------------------------------------------------
    #[test]
    fn test_event_datetime_date_only() {
        let edt = EventDateTime::date_only("2026-07-04");
        assert_eq!(edt.date.as_deref(), Some("2026-07-04"));
        assert!(edt.date_time.is_none());
        assert!(edt.is_all_day());
    }

    // -----------------------------------------------------------------------
    // test_event_datetime_with_tz
    // -----------------------------------------------------------------------
    #[test]
    fn test_event_datetime_with_tz() {
        let edt = EventDateTime::date_time("2026-07-04T14:00:00-04:00", Some("America/New_York"));
        assert!(edt.date.is_none());
        assert_eq!(edt.date_time.as_deref(), Some("2026-07-04T14:00:00-04:00"));
        assert_eq!(edt.time_zone.as_deref(), Some("America/New_York"));
        assert!(!edt.is_all_day());
    }

    // -----------------------------------------------------------------------
    // test_event_list_deserialize
    // -----------------------------------------------------------------------
    #[test]
    fn test_event_list_deserialize() {
        let json = r#"{
            "summary": "My Calendar",
            "timeZone": "UTC",
            "nextPageToken": "token_xyz",
            "items": [
                {"id": "ev1", "summary": "Event One"},
                {"id": "ev2", "summary": "Event Two"}
            ]
        }"#;

        let list: EventList = serde_json::from_str(json).expect("deserialize event list");
        assert_eq!(list.items.len(), 2);
        assert_eq!(list.next_page_token.as_deref(), Some("token_xyz"));
        assert_eq!(list.summary.as_deref(), Some("My Calendar"));
        assert_eq!(list.time_zone.as_deref(), Some("UTC"));
    }

    // -----------------------------------------------------------------------
    // test_calendar_list_entry_deserialize
    // -----------------------------------------------------------------------
    #[test]
    fn test_calendar_list_entry_deserialize() {
        let json = r##"{
            "id": "primary",
            "summary": "John Doe",
            "description": "Primary calendar",
            "timeZone": "America/Chicago",
            "colorId": "14",
            "backgroundColor": "#9fc6e7",
            "foregroundColor": "#000000",
            "primary": true,
            "accessRole": "owner"
        }"##;

        let entry: CalendarListEntry =
            serde_json::from_str(json).expect("deserialize calendar entry");
        assert_eq!(entry.id, "primary");
        assert_eq!(entry.primary, Some(true));
        assert_eq!(entry.access_role.as_deref(), Some("owner"));
        assert_eq!(entry.time_zone.as_deref(), Some("America/Chicago"));
    }

    // -----------------------------------------------------------------------
    // test_attendee_deserialize
    // -----------------------------------------------------------------------
    #[test]
    fn test_attendee_deserialize() {
        let json = r#"{
            "email": "bob@example.com",
            "displayName": "Bob",
            "responseStatus": "accepted",
            "organizer": false,
            "self": false,
            "optional": true
        }"#;

        let attendee: Attendee = serde_json::from_str(json).expect("deserialize attendee");
        assert_eq!(attendee.email.as_deref(), Some("bob@example.com"));
        assert_eq!(attendee.response_status.as_deref(), Some("accepted"));
        assert_eq!(attendee.optional, Some(true));
    }

    // -----------------------------------------------------------------------
    // test_event_has_conference
    // -----------------------------------------------------------------------
    #[test]
    fn test_event_has_conference() {
        let json = r#"{
            "id": "conf_event",
            "summary": "Video Call",
            "conferenceData": {
                "conferenceSolution": {"key": {"type": "hangoutsMeet"}},
                "entryPoints": [{"entryPointType": "video", "uri": "https://meet.google.com/abc-defg-hij"}]
            }
        }"#;

        let event: Event = serde_json::from_str(json).expect("deserialize conference event");
        assert!(
            event.has_conference(),
            "event with conferenceData must report has_conference=true"
        );

        let json_no_conf = r#"{"id": "plain_event", "summary": "Call"}"#;
        let event_no_conf: Event =
            serde_json::from_str(json_no_conf).expect("deserialize plain event");
        assert!(
            !event_no_conf.has_conference(),
            "event without conferenceData must report has_conference=false"
        );
    }
}
