// responses.rs - List form responses from the Google Forms API.
// Google Forms API v1: https://developers.google.com/forms/api/reference/rest/v1/forms.responses/list

use reqwest::Client;

use crate::types::{FormsError, ResponseList};

const FORMS_BASE: &str = "https://forms.googleapis.com/v1/forms";

// ---------------------------------------------------------------------------
// ListResponsesParams
// ---------------------------------------------------------------------------

/// Optional query parameters for listing form responses.
#[derive(Debug, Clone, Default)]
pub struct ListResponsesParams {
    /// Maximum number of responses to return per page (default: 5000, max: 5000).
    pub page_size: Option<u32>,

    /// Page token returned by a previous call.
    pub page_token: Option<String>,

    /// RFC 3339 timestamp; only responses submitted after this time are returned.
    pub filter: Option<String>,
}

// ---------------------------------------------------------------------------
// list_responses
// ---------------------------------------------------------------------------

/// List all responses for a form, with optional pagination and filtering.
///
/// Uses: `GET https://forms.googleapis.com/v1/forms/{formId}/responses`
pub async fn list_responses(
    client: &Client,
    access_token: &str,
    form_id: &str,
    params: &ListResponsesParams,
) -> Result<ResponseList, FormsError> {
    let url = format!("{FORMS_BASE}/{form_id}/responses");

    let mut query: Vec<(&str, String)> = Vec::new();

    if let Some(size) = params.page_size {
        query.push(("pageSize", size.to_string()));
    }
    if let Some(token) = &params.page_token {
        query.push(("pageToken", token.clone()));
    }
    if let Some(filter) = &params.filter {
        query.push(("filter", filter.clone()));
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
        return Err(FormsError::Api { status, message });
    }

    let list: ResponseList = resp.json().await?;
    Ok(list)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_responses_url_construction() {
        let form_id = "form_abc";
        let url = format!("{FORMS_BASE}/{form_id}/responses");
        assert_eq!(
            url,
            "https://forms.googleapis.com/v1/forms/form_abc/responses"
        );
    }

    #[test]
    fn test_list_responses_params_default() {
        let params = ListResponsesParams::default();
        assert!(params.page_size.is_none());
        assert!(params.page_token.is_none());
        assert!(params.filter.is_none());
    }

    #[test]
    fn test_list_responses_params_with_values() {
        let params = ListResponsesParams {
            page_size: Some(100),
            page_token: Some("tok_abc".to_string()),
            filter: Some("timestamp > 2024-01-01T00:00:00Z".to_string()),
        };
        assert_eq!(params.page_size, Some(100));
        assert_eq!(params.page_token.as_deref(), Some("tok_abc"));
        assert!(params.filter.is_some());
    }

    #[test]
    fn test_query_params_built_correctly() {
        // Verify query parameter construction logic mirrors what list_responses does.
        let params = ListResponsesParams {
            page_size: Some(50),
            page_token: Some("next_page".to_string()),
            filter: None,
        };

        let mut query: Vec<(&str, String)> = Vec::new();
        if let Some(size) = params.page_size {
            query.push(("pageSize", size.to_string()));
        }
        if let Some(token) = &params.page_token {
            query.push(("pageToken", token.clone()));
        }
        if let Some(filter) = &params.filter {
            query.push(("filter", filter.clone()));
        }

        assert_eq!(query.len(), 2);
        assert_eq!(query[0], ("pageSize", "50".to_string()));
        assert_eq!(query[1], ("pageToken", "next_page".to_string()));
    }

    #[test]
    fn test_query_params_empty_when_all_none() {
        let params = ListResponsesParams::default();

        let mut query: Vec<(&str, String)> = Vec::new();
        if let Some(size) = params.page_size {
            query.push(("pageSize", size.to_string()));
        }
        if let Some(token) = &params.page_token {
            query.push(("pageToken", token.clone()));
        }
        if let Some(filter) = &params.filter {
            query.push(("filter", filter.clone()));
        }

        assert!(query.is_empty());
    }
}
