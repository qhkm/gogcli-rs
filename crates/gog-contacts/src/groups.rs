// Google People API - contact groups
// Ported from internal/contacts/groups.go (conceptual)
//
// Endpoint:
//   GET https://people.googleapis.com/v1/contactGroups

use crate::types::{ContactGroup, ContactGroupList, ContactsError};

const PEOPLE_BASE: &str = "https://people.googleapis.com/v1";

/// List all contact groups for the authenticated user.
///
/// Returns the first page; use `next_page_token` to paginate.
pub async fn list_contact_groups(
    client: &reqwest::Client,
    page_token: Option<&str>,
    page_size: Option<u32>,
) -> Result<ContactGroupList, ContactsError> {
    let url = format!("{PEOPLE_BASE}/contactGroups");

    let mut query: Vec<(&str, String)> = Vec::new();
    if let Some(token) = page_token {
        query.push(("pageToken", token.to_string()));
    }
    if let Some(size) = page_size {
        query.push(("pageSize", size.to_string()));
    }

    let resp = client.get(&url).query(&query).send().await?;

    let status = resp.status().as_u16();
    if status >= 400 {
        let message = resp.text().await.unwrap_or_default();
        return Err(ContactsError::Api { status, message });
    }

    let list: ContactGroupList = resp.json().await?;
    Ok(list)
}

/// Get a single contact group by resource name (e.g., "contactGroups/friends").
pub async fn get_contact_group(
    client: &reqwest::Client,
    resource_name: &str,
) -> Result<ContactGroup, ContactsError> {
    if resource_name.is_empty() {
        return Err(ContactsError::MissingField("resourceName".to_string()));
    }

    let url = format!("{PEOPLE_BASE}/{resource_name}");

    let resp = client.get(&url).send().await?;

    let status = resp.status().as_u16();
    if status >= 400 {
        let message = resp.text().await.unwrap_or_default();
        return Err(ContactsError::Api { status, message });
    }

    let group: ContactGroup = resp.json().await?;
    Ok(group)
}
