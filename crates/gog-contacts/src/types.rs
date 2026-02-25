// Google People API v1 data types
// Matches Google People API v1 JSON response shapes.
// https://developers.google.com/people/api/rest/v1/people

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum ContactsError {
    #[error("Contacts API error ({status}): {message}")]
    Api { status: u16, message: String },

    #[error(transparent)]
    Http(#[from] reqwest::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error("missing field: {0}")]
    MissingField(String),
}

// ---------------------------------------------------------------------------
// Person
// ---------------------------------------------------------------------------

/// A person in Google Contacts (People API resource).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Person {
    /// The resource name (e.g., "people/c12345").
    #[serde(default)]
    pub resource_name: String,

    /// An opaque server-provided entity tag used for update conflicts.
    #[serde(default)]
    pub etag: String,

    #[serde(default)]
    pub names: Vec<Name>,

    #[serde(default)]
    pub email_addresses: Vec<EmailAddress>,

    #[serde(default)]
    pub phone_numbers: Vec<PhoneNumber>,

    #[serde(default)]
    pub addresses: Vec<Address>,

    #[serde(default)]
    pub organizations: Vec<Organization>,

    #[serde(default)]
    pub birthdays: Vec<Birthday>,

    #[serde(default)]
    pub biographies: Vec<Biography>,

    #[serde(default)]
    pub urls: Vec<Url>,

    #[serde(default)]
    pub user_defined: Vec<UserDefined>,
}

impl Person {
    /// Returns the display name of the first name entry, or "(No name)" if absent.
    pub fn display_name(&self) -> &str {
        self.names
            .first()
            .map(|n| n.display_name.as_str())
            .filter(|s| !s.is_empty())
            .unwrap_or("(No name)")
    }

    /// Returns the value of the first email address, if any.
    pub fn primary_email(&self) -> Option<&str> {
        self.email_addresses.first().map(|e| e.value.as_str())
    }

    /// Returns the value of the first phone number, if any.
    pub fn primary_phone(&self) -> Option<&str> {
        self.phone_numbers.first().map(|p| p.value.as_str())
    }
}

// ---------------------------------------------------------------------------
// Name
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Name {
    #[serde(default)]
    pub display_name: String,

    #[serde(default)]
    pub given_name: String,

    #[serde(default)]
    pub family_name: String,
}

// ---------------------------------------------------------------------------
// EmailAddress
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct EmailAddress {
    #[serde(default)]
    pub value: String,

    /// The email type, e.g. "home", "work", "other".
    #[serde(rename = "type", default)]
    pub type_: String,

    #[serde(default)]
    pub display_name: String,
}

// ---------------------------------------------------------------------------
// PhoneNumber
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PhoneNumber {
    #[serde(default)]
    pub value: String,

    /// The phone type, e.g. "mobile", "home", "work".
    #[serde(rename = "type", default)]
    pub type_: String,
}

// ---------------------------------------------------------------------------
// Address
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Address {
    #[serde(default)]
    pub street_address: String,

    #[serde(default)]
    pub city: String,

    #[serde(default)]
    pub region: String,

    #[serde(default)]
    pub postal_code: String,

    #[serde(default)]
    pub country: String,

    #[serde(rename = "type", default)]
    pub type_: String,
}

// ---------------------------------------------------------------------------
// Organization
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Organization {
    #[serde(default)]
    pub name: String,

    #[serde(default)]
    pub title: String,

    #[serde(default)]
    pub department: String,
}

// ---------------------------------------------------------------------------
// Birthday
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Birthday {
    pub date: Option<DateValue>,

    pub text: Option<String>,
}

/// A structured date value. Year may be omitted for recurring birthdays.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DateValue {
    pub year: Option<i32>,
    pub month: Option<i32>,
    pub day: Option<i32>,
}

// ---------------------------------------------------------------------------
// Biography
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Biography {
    #[serde(default)]
    pub value: String,

    #[serde(default)]
    pub content_type: String,
}

// ---------------------------------------------------------------------------
// Url
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Url {
    #[serde(default)]
    pub value: String,

    #[serde(rename = "type", default)]
    pub type_: String,
}

// ---------------------------------------------------------------------------
// UserDefined
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UserDefined {
    #[serde(default)]
    pub key: String,

    #[serde(default)]
    pub value: String,
}

// ---------------------------------------------------------------------------
// PersonList
// ---------------------------------------------------------------------------

/// Response from the people.connections.list endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PersonList {
    #[serde(default)]
    pub connections: Vec<Person>,

    pub next_page_token: Option<String>,

    /// Total number of people in the list, without pagination.
    pub total_people: Option<i32>,

    /// Total number of items in the list (may differ from `total_people`).
    pub total_items: Option<i32>,
}

// ---------------------------------------------------------------------------
// SearchResponse
// ---------------------------------------------------------------------------

/// Response from the people.searchContacts endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SearchResponse {
    #[serde(default)]
    pub results: Vec<SearchResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub person: Person,
}

// ---------------------------------------------------------------------------
// ContactGroup
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ContactGroup {
    #[serde(default)]
    pub resource_name: String,

    #[serde(default)]
    pub name: String,

    pub member_count: Option<i32>,

    /// The contact group type, e.g. "USER_CONTACT_GROUP" or "SYSTEM_CONTACT_GROUP".
    #[serde(default)]
    pub group_type: String,
}

/// Response from the contactGroups.list endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ContactGroupList {
    #[serde(default)]
    pub contact_groups: Vec<ContactGroup>,

    pub next_page_token: Option<String>,

    pub total_items: Option<i32>,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Person deserialization
    // -----------------------------------------------------------------------

    #[test]
    fn test_person_deserialize() {
        let json = r#"{
            "resourceName": "people/c1234567",
            "etag": "some-etag",
            "names": [
                {
                    "displayName": "Jane Doe",
                    "givenName": "Jane",
                    "familyName": "Doe"
                }
            ],
            "emailAddresses": [
                {
                    "value": "jane@example.com",
                    "type": "work",
                    "displayName": ""
                }
            ],
            "phoneNumbers": [
                {
                    "value": "+1-555-0100",
                    "type": "mobile"
                }
            ]
        }"#;

        let person: Person = serde_json::from_str(json).expect("deserialize person");
        assert_eq!(person.resource_name, "people/c1234567");
        assert_eq!(person.etag, "some-etag");
        assert_eq!(person.names.len(), 1);
        assert_eq!(person.names[0].display_name, "Jane Doe");
        assert_eq!(person.names[0].given_name, "Jane");
        assert_eq!(person.names[0].family_name, "Doe");
        assert_eq!(person.email_addresses.len(), 1);
        assert_eq!(person.email_addresses[0].value, "jane@example.com");
        assert_eq!(person.email_addresses[0].type_, "work");
        assert_eq!(person.phone_numbers.len(), 1);
        assert_eq!(person.phone_numbers[0].value, "+1-555-0100");
        assert_eq!(person.phone_numbers[0].type_, "mobile");
    }

    // -----------------------------------------------------------------------
    // Person::display_name
    // -----------------------------------------------------------------------

    #[test]
    fn test_person_display_name() {
        let person = Person {
            names: vec![Name {
                display_name: "Alice Smith".to_string(),
                given_name: "Alice".to_string(),
                family_name: "Smith".to_string(),
            }],
            ..Default::default()
        };
        assert_eq!(person.display_name(), "Alice Smith");
    }

    #[test]
    fn test_person_display_name_missing() {
        let person = Person::default();
        assert_eq!(person.display_name(), "(No name)");
    }

    #[test]
    fn test_person_display_name_empty_string() {
        let person = Person {
            names: vec![Name {
                display_name: "".to_string(),
                given_name: "".to_string(),
                family_name: "".to_string(),
            }],
            ..Default::default()
        };
        assert_eq!(person.display_name(), "(No name)");
    }

    // -----------------------------------------------------------------------
    // Person::primary_email
    // -----------------------------------------------------------------------

    #[test]
    fn test_person_primary_email() {
        let person = Person {
            email_addresses: vec![
                EmailAddress {
                    value: "first@example.com".to_string(),
                    type_: "work".to_string(),
                    display_name: "".to_string(),
                },
                EmailAddress {
                    value: "second@example.com".to_string(),
                    type_: "home".to_string(),
                    display_name: "".to_string(),
                },
            ],
            ..Default::default()
        };
        assert_eq!(person.primary_email(), Some("first@example.com"));
    }

    #[test]
    fn test_person_primary_email_none() {
        let person = Person::default();
        assert_eq!(person.primary_email(), None);
    }

    // -----------------------------------------------------------------------
    // Person::primary_phone
    // -----------------------------------------------------------------------

    #[test]
    fn test_person_primary_phone() {
        let person = Person {
            phone_numbers: vec![
                PhoneNumber {
                    value: "+1-800-555-0100".to_string(),
                    type_: "mobile".to_string(),
                },
                PhoneNumber {
                    value: "+1-800-555-0200".to_string(),
                    type_: "home".to_string(),
                },
            ],
            ..Default::default()
        };
        assert_eq!(person.primary_phone(), Some("+1-800-555-0100"));
    }

    #[test]
    fn test_person_primary_phone_none() {
        let person = Person::default();
        assert_eq!(person.primary_phone(), None);
    }

    // -----------------------------------------------------------------------
    // EmailAddress deserialization
    // -----------------------------------------------------------------------

    #[test]
    fn test_email_address_deserialize() {
        let json = r#"{
            "value": "user@work.com",
            "type": "work",
            "displayName": "Work Email"
        }"#;

        let email: EmailAddress = serde_json::from_str(json).expect("deserialize email");
        assert_eq!(email.value, "user@work.com");
        assert_eq!(email.type_, "work");
        assert_eq!(email.display_name, "Work Email");
    }

    // -----------------------------------------------------------------------
    // PersonList deserialization
    // -----------------------------------------------------------------------

    #[test]
    fn test_person_list_deserialize() {
        let json = r#"{
            "connections": [
                {
                    "resourceName": "people/c1",
                    "etag": "etag1",
                    "names": [{"displayName": "Alice", "givenName": "Alice", "familyName": ""}]
                },
                {
                    "resourceName": "people/c2",
                    "etag": "etag2",
                    "names": [{"displayName": "Bob", "givenName": "Bob", "familyName": ""}]
                }
            ],
            "nextPageToken": "page_token_xyz",
            "totalPeople": 2,
            "totalItems": 2
        }"#;

        let list: PersonList = serde_json::from_str(json).expect("deserialize person list");
        assert_eq!(list.connections.len(), 2);
        assert_eq!(list.connections[0].resource_name, "people/c1");
        assert_eq!(list.connections[0].names[0].display_name, "Alice");
        assert_eq!(list.connections[1].resource_name, "people/c2");
        assert_eq!(list.next_page_token.as_deref(), Some("page_token_xyz"));
        assert_eq!(list.total_people, Some(2));
        assert_eq!(list.total_items, Some(2));
    }

    #[test]
    fn test_person_list_empty() {
        let json = r#"{}"#;
        let list: PersonList = serde_json::from_str(json).expect("deserialize empty list");
        assert!(list.connections.is_empty());
        assert!(list.next_page_token.is_none());
        assert!(list.total_people.is_none());
        assert!(list.total_items.is_none());
    }

    // -----------------------------------------------------------------------
    // ContactGroup deserialization
    // -----------------------------------------------------------------------

    #[test]
    fn test_contact_group_deserialize() {
        let json = r#"{
            "resourceName": "contactGroups/friends",
            "name": "Friends",
            "memberCount": 42,
            "groupType": "USER_CONTACT_GROUP"
        }"#;

        let group: ContactGroup = serde_json::from_str(json).expect("deserialize contact group");
        assert_eq!(group.resource_name, "contactGroups/friends");
        assert_eq!(group.name, "Friends");
        assert_eq!(group.member_count, Some(42));
        assert_eq!(group.group_type, "USER_CONTACT_GROUP");
    }

    #[test]
    fn test_contact_group_deserialize_no_member_count() {
        let json = r#"{
            "resourceName": "contactGroups/starred",
            "name": "Starred in Android",
            "groupType": "SYSTEM_CONTACT_GROUP"
        }"#;

        let group: ContactGroup = serde_json::from_str(json).expect("deserialize contact group");
        assert_eq!(group.resource_name, "contactGroups/starred");
        assert_eq!(group.name, "Starred in Android");
        assert!(group.member_count.is_none());
        assert_eq!(group.group_type, "SYSTEM_CONTACT_GROUP");
    }

    // -----------------------------------------------------------------------
    // Birthday deserialization
    // -----------------------------------------------------------------------

    #[test]
    fn test_birthday_deserialize_with_year() {
        let json = r#"{
            "date": {
                "year": 1990,
                "month": 6,
                "day": 15
            },
            "text": "June 15, 1990"
        }"#;

        let bday: Birthday = serde_json::from_str(json).expect("deserialize birthday");
        let date = bday.date.expect("date present");
        assert_eq!(date.year, Some(1990));
        assert_eq!(date.month, Some(6));
        assert_eq!(date.day, Some(15));
        assert_eq!(bday.text.as_deref(), Some("June 15, 1990"));
    }

    #[test]
    fn test_birthday_deserialize_without_year() {
        let json = r#"{
            "date": {
                "month": 3,
                "day": 22
            }
        }"#;

        let bday: Birthday = serde_json::from_str(json).expect("deserialize birthday without year");
        let date = bday.date.expect("date present");
        assert!(date.year.is_none(), "year should be absent");
        assert_eq!(date.month, Some(3));
        assert_eq!(date.day, Some(22));
        assert!(bday.text.is_none());
    }

    #[test]
    fn test_birthday_deserialize_text_only() {
        let json = r#"{"text": "Sometime in spring"}"#;
        let bday: Birthday = serde_json::from_str(json).expect("deserialize text-only birthday");
        assert!(bday.date.is_none());
        assert_eq!(bday.text.as_deref(), Some("Sometime in spring"));
    }

    // -----------------------------------------------------------------------
    // Organization deserialization
    // -----------------------------------------------------------------------

    #[test]
    fn test_organization_deserialize() {
        let json = r#"{
            "name": "Acme Corp",
            "title": "Senior Engineer",
            "department": "Platform"
        }"#;

        let org: Organization = serde_json::from_str(json).expect("deserialize organization");
        assert_eq!(org.name, "Acme Corp");
        assert_eq!(org.title, "Senior Engineer");
        assert_eq!(org.department, "Platform");
    }

    #[test]
    fn test_organization_deserialize_minimal() {
        let json = r#"{"name": "Startup Inc"}"#;
        let org: Organization = serde_json::from_str(json).expect("deserialize minimal org");
        assert_eq!(org.name, "Startup Inc");
        assert!(org.title.is_empty());
        assert!(org.department.is_empty());
    }

    // -----------------------------------------------------------------------
    // ContactsError display
    // -----------------------------------------------------------------------

    #[test]
    fn test_contacts_error_api_display() {
        let err = ContactsError::Api { status: 404, message: "Not Found".to_string() };
        let msg = err.to_string();
        assert!(msg.contains("404"), "got: {msg}");
        assert!(msg.contains("Not Found"), "got: {msg}");
    }

    #[test]
    fn test_contacts_error_missing_field_display() {
        let err = ContactsError::MissingField("resourceName".to_string());
        let msg = err.to_string();
        assert!(msg.contains("resourceName"), "got: {msg}");
    }
}
