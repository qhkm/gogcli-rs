// gog-calendar delete module
// Delete an event from a Google Calendar.
// API reference: DELETE https://www.googleapis.com/calendar/v3/calendars/{calendarId}/events/{eventId}

use crate::types::CalendarError;

const CALENDAR_EVENTS_URL: &str = "https://www.googleapis.com/calendar/v3/calendars";

// ---------------------------------------------------------------------------
// delete_event
// ---------------------------------------------------------------------------

/// Delete a single event from the specified calendar.
///
/// Returns `Ok(())` on success (HTTP 204 No Content).
pub async fn delete_event(
    client: &reqwest::Client,
    access_token: &str,
    calendar_id: &str,
    event_id: &str,
) -> Result<(), CalendarError> {
    let url = format!("{}/{}/events/{}", CALENDAR_EVENTS_URL, calendar_id, event_id);

    let resp = client
        .delete(&url)
        .bearer_auth(access_token)
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        let message = resp.text().await.unwrap_or_default();
        return Err(CalendarError::Api { status, message });
    }

    Ok(())
}
