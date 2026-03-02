// gog-calendar create module
// Create an event in a Google Calendar.
// API reference: POST https://www.googleapis.com/calendar/v3/calendars/{calendarId}/events

use crate::types::{Attendee, CalendarError, Event, EventDateTime};

const CALENDAR_EVENTS_URL: &str = "https://www.googleapis.com/calendar/v3/calendars";

// ---------------------------------------------------------------------------
// Parameters
// ---------------------------------------------------------------------------

pub struct CreateParams {
    /// Calendar to insert the event into. Use "primary" for the primary calendar.
    pub calendar_id: String,
    /// Short human-readable title.
    pub summary: String,
    /// Extended description / notes.
    pub description: Option<String>,
    /// Geographic location.
    pub location: Option<String>,
    /// Event start time.
    pub start: EventDateTime,
    /// Event end time.
    pub end: EventDateTime,
    /// Attendee email addresses.
    pub attendees: Vec<String>,
    /// RRULE/EXRULE/RDATE/EXDATE strings for recurring events.
    pub recurrence: Vec<String>,
}

impl CreateParams {
    /// Convert `CreateParams` into an `Event` body ready for serialisation.
    pub fn to_event(&self) -> Event {
        let attendees: Vec<Attendee> = self
            .attendees
            .iter()
            .map(|email| Attendee {
                email: Some(email.clone()),
                display_name: None,
                response_status: None,
                organizer: None,
                is_self: None,
                optional: None,
            })
            .collect();

        let recurrence = if self.recurrence.is_empty() {
            None
        } else {
            Some(self.recurrence.clone())
        };

        Event {
            id: None,
            summary: Some(self.summary.clone()),
            description: self.description.clone(),
            location: self.location.clone(),
            start: Some(self.start.clone()),
            end: Some(self.end.clone()),
            status: None,
            html_link: None,
            created: None,
            updated: None,
            creator: None,
            organizer: None,
            attendees,
            recurrence,
            recurring_event_id: None,
            color_id: None,
            conference_data: None,
            reminders: None,
            event_type: None,
            visibility: None,
            transparency: None,
        }
    }
}

// ---------------------------------------------------------------------------
// create_event
// ---------------------------------------------------------------------------

/// Create an event in the specified calendar and return the created Event.
pub async fn create_event(
    client: &reqwest::Client,
    access_token: &str,
    params: &CreateParams,
) -> Result<Event, CalendarError> {
    let url = format!("{}/{}/events", CALENDAR_EVENTS_URL, params.calendar_id);
    let body = params.to_event();

    let resp = client
        .post(&url)
        .bearer_auth(access_token)
        .json(&body)
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        let message = resp.text().await.unwrap_or_default();
        return Err(CalendarError::Api { status, message });
    }

    let event: Event = resp.json().await?;
    Ok(event)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::EventDateTime;

    // -----------------------------------------------------------------------
    // test_create_params_to_event
    // -----------------------------------------------------------------------
    #[test]
    fn test_create_params_to_event() {
        let params = CreateParams {
            calendar_id: "primary".to_string(),
            summary: "Team Meeting".to_string(),
            description: Some("Agenda: Q1 review".to_string()),
            location: Some("Conference Room A".to_string()),
            start: EventDateTime::date_time("2026-02-25T10:00:00-05:00", Some("America/New_York")),
            end: EventDateTime::date_time("2026-02-25T11:00:00-05:00", Some("America/New_York")),
            attendees: vec![
                "alice@example.com".to_string(),
                "bob@example.com".to_string(),
            ],
            recurrence: vec!["RRULE:FREQ=WEEKLY;BYDAY=TU".to_string()],
        };

        let event = params.to_event();

        assert_eq!(event.summary.as_deref(), Some("Team Meeting"));
        assert_eq!(event.description.as_deref(), Some("Agenda: Q1 review"));
        assert_eq!(event.location.as_deref(), Some("Conference Room A"));
        assert_eq!(event.attendees.len(), 2);
        assert_eq!(
            event.attendees[0].email.as_deref(),
            Some("alice@example.com")
        );
        assert_eq!(event.attendees[1].email.as_deref(), Some("bob@example.com"));
        assert!(event
            .recurrence
            .as_ref()
            .map(|r| r.len() == 1)
            .unwrap_or(false));
        assert_eq!(
            event.recurrence.as_ref().unwrap()[0],
            "RRULE:FREQ=WEEKLY;BYDAY=TU"
        );
        // The ID should be None – the server assigns it.
        assert!(event.id.is_none());
    }

    // -----------------------------------------------------------------------
    // test_create_params_to_event_no_attendees
    // -----------------------------------------------------------------------
    #[test]
    fn test_create_params_to_event_no_attendees() {
        let params = CreateParams {
            calendar_id: "primary".to_string(),
            summary: "Focus time".to_string(),
            description: None,
            location: None,
            start: EventDateTime::date_only("2026-03-01"),
            end: EventDateTime::date_only("2026-03-02"),
            attendees: vec![],
            recurrence: vec![],
        };

        let event = params.to_event();

        assert!(event.attendees.is_empty());
        assert!(event.recurrence.is_none());
        assert!(event.is_all_day());
    }
}
