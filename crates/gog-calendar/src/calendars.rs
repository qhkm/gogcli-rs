// gog-calendar calendars module
// List the user's calendars.
// API reference: GET https://www.googleapis.com/calendar/v3/users/me/calendarList

use crate::types::{CalendarError, CalendarList, CalendarListEntry};

const CALENDAR_LIST_URL: &str = "https://www.googleapis.com/calendar/v3/users/me/calendarList";

// ---------------------------------------------------------------------------
// list_calendars
// ---------------------------------------------------------------------------

/// Fetch all calendars in the user's calendar list, following pagination.
pub async fn list_calendars(
    client: &reqwest::Client,
    access_token: &str,
) -> Result<Vec<CalendarListEntry>, CalendarError> {
    let mut all_calendars: Vec<CalendarListEntry> = vec![];
    let mut page_token: Option<String> = None;

    loop {
        let mut req = client.get(CALENDAR_LIST_URL).bearer_auth(access_token);

        if let Some(ref token) = page_token {
            req = req.query(&[("pageToken", token)]);
        }

        let resp = req.send().await?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let message = resp.text().await.unwrap_or_default();
            return Err(CalendarError::Api { status, message });
        }

        let page: CalendarList = resp.json().await?;
        all_calendars.extend(page.items);

        match page.next_page_token {
            Some(token) => page_token = Some(token),
            None => break,
        }
    }

    Ok(all_calendars)
}
