// Google People API - delete contact
// Ported from internal/contacts/delete.go (conceptual)
//
// Endpoint:
//   DELETE https://people.googleapis.com/v1/{resourceName}:deleteContact

use crate::types::ContactsError;

const PEOPLE_BASE: &str = "https://people.googleapis.com/v1";

/// Delete a contact by resource name (e.g., "people/c12345").
///
/// Returns `Ok(())` on success. On failure returns `ContactsError::Api`.
pub async fn delete_contact(
    client: &reqwest::Client,
    resource_name: &str,
) -> Result<(), ContactsError> {
    if resource_name.is_empty() {
        return Err(ContactsError::MissingField("resourceName".to_string()));
    }

    let url = format!("{PEOPLE_BASE}/{resource_name}:deleteContact");

    let resp = client.delete(&url).send().await?;

    let status = resp.status().as_u16();
    if status >= 400 {
        let message = resp.text().await.unwrap_or_default();
        return Err(ContactsError::Api { status, message });
    }

    Ok(())
}
