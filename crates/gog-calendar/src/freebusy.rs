// gog-calendar freebusy module
// Query free/busy status for a set of calendars.
// API reference: POST https://www.googleapis.com/calendar/v3/freeBusy

use crate::types::{CalendarError, FreeBusyResponse};

const FREEBUSY_URL: &str = "https://www.googleapis.com/calendar/v3/freeBusy";

// ---------------------------------------------------------------------------
// query_freebusy
// ---------------------------------------------------------------------------

/// Query free/busy information for the given calendars within a time window.
///
/// `time_min` and `time_max` must be RFC3339 timestamps, e.g.
/// `"2026-02-25T00:00:00Z"`.
pub async fn query_freebusy(
    client: &reqwest::Client,
    access_token: &str,
    calendars: &[String],
    time_min: &str,
    time_max: &str,
) -> Result<FreeBusyResponse, CalendarError> {
    // Build the request body according to the FreeBusy API spec.
    let items: Vec<serde_json::Value> = calendars
        .iter()
        .map(|id| serde_json::json!({"id": id}))
        .collect();

    let body = serde_json::json!({
        "timeMin": time_min,
        "timeMax": time_max,
        "items": items,
    });

    let resp = client
        .post(FREEBUSY_URL)
        .bearer_auth(access_token)
        .json(&body)
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        let message = resp.text().await.unwrap_or_default();
        return Err(CalendarError::Api { status, message });
    }

    let freebusy: FreeBusyResponse = resp.json().await?;
    Ok(freebusy)
}
