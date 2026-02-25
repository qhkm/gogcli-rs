// Google People API - create contact
// Ported from internal/contacts/create.go (conceptual)
//
// Endpoint:
//   POST https://people.googleapis.com/v1/people:createContact

use crate::types::{ContactsError, Person};

const PEOPLE_BASE: &str = "https://people.googleapis.com/v1";

/// Create a new contact with the given person data.
///
/// Returns the newly created `Person` including its assigned `resource_name`.
pub async fn create_contact(
    client: &reqwest::Client,
    person: &Person,
) -> Result<Person, ContactsError> {
    let url = format!("{PEOPLE_BASE}/people:createContact");

    let resp = client.post(&url).json(person).send().await?;

    let status = resp.status().as_u16();
    if status >= 400 {
        let message = resp.text().await.unwrap_or_default();
        return Err(ContactsError::Api { status, message });
    }

    let created: Person = resp.json().await?;
    Ok(created)
}
