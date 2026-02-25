// gog-calendar list module
// List events from a Google Calendar.
// API reference: GET https://www.googleapis.com/calendar/v3/calendars/{calendarId}/events

use crate::types::{CalendarError, EventList};

const CALENDAR_EVENTS_URL: &str = "https://www.googleapis.com/calendar/v3/calendars";

// ---------------------------------------------------------------------------
// Parameters
// ---------------------------------------------------------------------------

pub struct ListParams {
    /// Calendar identifier. Use "primary" for the user's primary calendar.
    pub calendar_id: String,
    /// Lower bound (inclusive) for an event's end time (RFC3339).
    pub time_min: Option<String>,
    /// Upper bound (exclusive) for an event's start time (RFC3339).
    pub time_max: Option<String>,
    /// Maximum number of events to return.
    pub max_results: Option<u32>,
    /// Token for the next page of results.
    pub page_token: Option<String>,
    /// When true, recurring events are expanded into individual instances.
    pub single_events: bool,
    /// Order of the events: "startTime" or "updated".
    pub order_by: Option<String>,
    /// Free text search over event fields.
    pub query: Option<String>,
}

impl Default for ListParams {
    fn default() -> Self {
        Self {
            calendar_id: "primary".to_string(),
            time_min: None,
            time_max: None,
            max_results: None,
            page_token: None,
            single_events: true,
            order_by: None,
            query: None,
        }
    }
}

// ---------------------------------------------------------------------------
// list_events
// ---------------------------------------------------------------------------

/// Fetch a page of events from the specified calendar.
pub async fn list_events(
    client: &reqwest::Client,
    access_token: &str,
    params: &ListParams,
) -> Result<EventList, CalendarError> {
    let url = format!("{}/{}/events", CALENDAR_EVENTS_URL, params.calendar_id);

    let mut query: Vec<(&str, String)> = vec![];

    if let Some(ref v) = params.time_min {
        query.push(("timeMin", v.clone()));
    }
    if let Some(ref v) = params.time_max {
        query.push(("timeMax", v.clone()));
    }
    if let Some(n) = params.max_results {
        query.push(("maxResults", n.to_string()));
    }
    if let Some(ref v) = params.page_token {
        query.push(("pageToken", v.clone()));
    }
    if params.single_events {
        query.push(("singleEvents", "true".to_string()));
    }
    if let Some(ref v) = params.order_by {
        query.push(("orderBy", v.clone()));
    }
    if let Some(ref v) = params.query {
        query.push(("q", v.clone()));
    }

    let resp = client
        .get(&url)
        .bearer_auth(access_token)
        .query(&query)
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        let message = resp.text().await.unwrap_or_default();
        return Err(CalendarError::Api { status, message });
    }

    let list: EventList = resp.json().await?;
    Ok(list)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // test_list_params_default_calendar
    // -----------------------------------------------------------------------
    #[test]
    fn test_list_params_default_calendar() {
        let params = ListParams::default();
        assert_eq!(params.calendar_id, "primary");
        assert!(params.single_events, "single_events should default to true");
        assert!(params.time_min.is_none());
        assert!(params.time_max.is_none());
    }
}
