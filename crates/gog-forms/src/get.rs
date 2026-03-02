// get.rs - Fetch a Google Form or a single form response by ID.
// Google Forms API v1: https://developers.google.com/forms/api/reference/rest

use reqwest::Client;

use crate::types::{Form, FormResponse, FormsError};

const FORMS_BASE: &str = "https://forms.googleapis.com/v1/forms";

// ---------------------------------------------------------------------------
// get_form
// ---------------------------------------------------------------------------

/// Fetch a Google Form by its form ID.
///
/// Uses: `GET https://forms.googleapis.com/v1/forms/{formId}`
pub async fn get_form(
    client: &Client,
    access_token: &str,
    form_id: &str,
) -> Result<Form, FormsError> {
    let url = format!("{FORMS_BASE}/{form_id}");

    let resp = client.get(&url).bearer_auth(access_token).send().await?;

    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        let message = resp.text().await.unwrap_or_default();
        return Err(FormsError::Api { status, message });
    }

    let form: Form = resp.json().await?;
    Ok(form)
}

// ---------------------------------------------------------------------------
// get_response
// ---------------------------------------------------------------------------

/// Fetch a single form response by its response ID.
///
/// Uses: `GET https://forms.googleapis.com/v1/forms/{formId}/responses/{responseId}`
pub async fn get_response(
    client: &Client,
    access_token: &str,
    form_id: &str,
    response_id: &str,
) -> Result<FormResponse, FormsError> {
    let url = format!("{FORMS_BASE}/{form_id}/responses/{response_id}");

    let resp = client.get(&url).bearer_auth(access_token).send().await?;

    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        let message = resp.text().await.unwrap_or_default();
        return Err(FormsError::Api { status, message });
    }

    let response: FormResponse = resp.json().await?;
    Ok(response)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_forms_base_url() {
        assert_eq!(FORMS_BASE, "https://forms.googleapis.com/v1/forms");
    }

    #[test]
    fn test_get_form_url_construction() {
        let form_id = "1FAIpQLSe_abc123";
        let expected = format!("{FORMS_BASE}/{form_id}");
        assert_eq!(
            expected,
            "https://forms.googleapis.com/v1/forms/1FAIpQLSe_abc123"
        );
    }

    #[test]
    fn test_get_response_url_construction() {
        let form_id = "form_xyz";
        let response_id = "resp_001";
        let url = format!("{FORMS_BASE}/{form_id}/responses/{response_id}");
        assert_eq!(
            url,
            "https://forms.googleapis.com/v1/forms/form_xyz/responses/resp_001"
        );
    }
}
