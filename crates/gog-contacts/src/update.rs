// Google People API - update contact
// Ported from internal/contacts/update.go (conceptual)
//
// Endpoint:
//   PATCH https://people.googleapis.com/v1/{resourceName}:updateContact?updatePersonFields=...

use crate::types::{ContactsError, Person};

const PEOPLE_BASE: &str = "https://people.googleapis.com/v1";

/// Default set of fields to update when no specific fields are provided.
const DEFAULT_UPDATE_FIELDS: &str =
    "names,emailAddresses,phoneNumbers,organizations,birthdays,addresses,biographies,urls,userDefined";

/// Update an existing contact.
///
/// `update_person_fields` is a comma-separated list of fields to update.
/// If `None`, a sensible default set is used.
///
/// The `person` must have its `resource_name` and `etag` populated.
pub async fn update_contact(
    client: &reqwest::Client,
    person: &Person,
    update_person_fields: Option<&str>,
) -> Result<Person, ContactsError> {
    if person.resource_name.is_empty() {
        return Err(ContactsError::MissingField("resourceName".to_string()));
    }

    let resource_name = &person.resource_name;
    let url = format!("{PEOPLE_BASE}/{resource_name}:updateContact");
    let fields = update_person_fields.unwrap_or(DEFAULT_UPDATE_FIELDS);

    let resp = client
        .patch(&url)
        .query(&[("updatePersonFields", fields)])
        .json(person)
        .send()
        .await?;

    let status = resp.status().as_u16();
    if status >= 400 {
        let message = resp.text().await.unwrap_or_default();
        return Err(ContactsError::Api { status, message });
    }

    let updated: Person = resp.json().await?;
    Ok(updated)
}
