// Google People API - search contacts
// Ported from internal/contacts/search.go (conceptual)
//
// Endpoints:
//   Search:  GET https://people.googleapis.com/v1/people:searchContacts?query={q}&readMask=...
//   List:    GET https://people.googleapis.com/v1/people/me/connections?personFields=...
//   Get:     GET https://people.googleapis.com/v1/{resourceName}?personFields=...

use crate::types::{ContactsError, Person, PersonList, SearchResponse};

const PEOPLE_BASE: &str = "https://people.googleapis.com/v1";

/// Default read mask used for search results.
const DEFAULT_READ_MASK: &str =
    "names,emailAddresses,phoneNumbers,organizations,birthdays,addresses";

/// Search contacts by a free-text query.
///
/// Calls `people:searchContacts` and returns the matching persons.
pub async fn search_contacts(
    client: &reqwest::Client,
    query: &str,
) -> Result<Vec<Person>, ContactsError> {
    let url = format!("{PEOPLE_BASE}/people:searchContacts");
    let resp = client
        .get(&url)
        .query(&[("query", query), ("readMask", DEFAULT_READ_MASK)])
        .send()
        .await?;

    let status = resp.status().as_u16();
    if status >= 400 {
        let message = resp.text().await.unwrap_or_default();
        return Err(ContactsError::Api { status, message });
    }

    let search_resp: SearchResponse = resp.json().await?;
    let persons = search_resp.results.into_iter().map(|r| r.person).collect();
    Ok(persons)
}

/// List all connections (contacts) for the authenticated user.
///
/// Returns the first page of results; iterate using `next_page_token` for additional pages.
pub async fn list_contacts(
    client: &reqwest::Client,
    page_token: Option<&str>,
    page_size: Option<u32>,
) -> Result<PersonList, ContactsError> {
    let url = format!("{PEOPLE_BASE}/people/me/connections");
    let person_fields = "names,emailAddresses,phoneNumbers,organizations,birthdays,addresses";

    let mut query: Vec<(&str, String)> = vec![("personFields", person_fields.to_string())];
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

    let list: PersonList = resp.json().await?;
    Ok(list)
}

/// Get a single contact by resource name (e.g., "people/c12345").
pub async fn get_contact(
    client: &reqwest::Client,
    resource_name: &str,
) -> Result<Person, ContactsError> {
    let url = format!("{PEOPLE_BASE}/{resource_name}");
    let person_fields = "names,emailAddresses,phoneNumbers,organizations,birthdays,addresses,biographies,urls,userDefined";

    let resp = client
        .get(&url)
        .query(&[("personFields", person_fields)])
        .send()
        .await?;

    let status = resp.status().as_u16();
    if status >= 400 {
        let message = resp.text().await.unwrap_or_default();
        return Err(ContactsError::Api { status, message });
    }

    let person: Person = resp.json().await?;
    Ok(person)
}
