// Google Keep API v1 data types
// Matches Google Keep API v1 JSON response shapes.

use serde::{Deserialize, Serialize};

/// A single Google Keep note.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    /// Resource name of the note (e.g. "notes/abc123").
    pub name: String,
    /// Title of the note.
    #[serde(default)]
    pub title: String,
    /// Body content of the note.
    pub body: Option<NoteBody>,
    /// Creation time (RFC 3339 timestamp string).
    pub create_time: Option<String>,
    /// Last update time (RFC 3339 timestamp string).
    pub update_time: Option<String>,
    /// Whether the note is in the trash.
    #[serde(default)]
    pub trashed: bool,
    /// Permissions attached to the note.
    #[serde(default)]
    pub permissions: Vec<Permission>,
}

impl Note {
    /// Return the note title, or "(Untitled)" if the title is empty.
    pub fn display_title(&self) -> &str {
        if self.title.is_empty() {
            "(Untitled)"
        } else {
            &self.title
        }
    }

    /// Extract plain text from the note body.
    ///
    /// - For text content: returns the text string.
    /// - For list content: joins checked/unchecked items with newlines.
    /// - Returns `None` if there is no body or no recognisable content.
    pub fn body_text(&self) -> Option<String> {
        let body = self.body.as_ref()?;

        if let Some(text_content) = &body.text {
            return Some(text_content.text.clone());
        }

        if let Some(list_content) = &body.list {
            if list_content.list_items.is_empty() {
                return None;
            }
            let lines: Vec<String> = list_content
                .list_items
                .iter()
                .map(|item| item.display_text())
                .collect();
            return Some(lines.join("\n"));
        }

        None
    }

    /// Returns `true` if the note body contains a list (rather than plain text).
    pub fn is_list(&self) -> bool {
        self.body
            .as_ref()
            .map(|b| b.list.is_some())
            .unwrap_or(false)
    }
}

/// Body of a note — contains either text or a list (mutually exclusive in the API).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteBody {
    /// Plain text content.
    pub text: Option<TextContent>,
    /// List content.
    pub list: Option<ListContent>,
}

/// Plain-text body content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextContent {
    pub text: String,
}

/// List body content containing ordered list items.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListContent {
    #[serde(default)]
    pub list_items: Vec<ListItem>,
}

/// A single item in a Keep list.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListItem {
    /// The text of this list item.
    pub text: TextContent,
    /// Whether this item is checked (completed).
    #[serde(default)]
    pub checked: bool,
    /// Nested child items.
    #[serde(default)]
    pub children_list_items: Vec<ListItem>,
}

impl ListItem {
    /// Format the item as "[x] text" (checked) or "[ ] text" (unchecked).
    pub fn display_text(&self) -> String {
        if self.checked {
            format!("[x] {}", self.text.text)
        } else {
            format!("[ ] {}", self.text.text)
        }
    }
}

/// A permission entry on a note.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Permission {
    /// Resource name of the permission.
    pub name: Option<String>,
    /// Role granted (e.g. "OWNER", "WRITER", "READER").
    pub role: Option<String>,
    /// Email address of the user this permission applies to.
    pub email: Option<String>,
}

/// Response from the `notes.list` endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteList {
    /// Notes returned in this page.
    #[serde(default)]
    pub notes: Vec<Note>,
    /// Token to retrieve the next page; absent on the last page.
    pub next_page_token: Option<String>,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------------------------------------------------
    // test_note_deserialize
    // ------------------------------------------------------------------
    #[test]
    fn test_note_deserialize() {
        let json = r#"{
            "name": "notes/abc123",
            "title": "My Note",
            "body": {
                "text": { "text": "Hello Keep" }
            },
            "createTime": "2024-01-01T00:00:00Z",
            "updateTime": "2024-01-02T12:00:00Z",
            "trashed": false,
            "permissions": [
                { "name": "notes/abc123/permissions/p1", "role": "OWNER", "email": "user@example.com" }
            ]
        }"#;

        let note: Note = serde_json::from_str(json).expect("deserialize note");
        assert_eq!(note.name, "notes/abc123");
        assert_eq!(note.title, "My Note");
        assert!(!note.trashed);
        assert_eq!(note.create_time.as_deref(), Some("2024-01-01T00:00:00Z"));
        assert_eq!(note.update_time.as_deref(), Some("2024-01-02T12:00:00Z"));
        assert_eq!(note.permissions.len(), 1);
        assert_eq!(note.permissions[0].role.as_deref(), Some("OWNER"));

        let body = note.body.expect("body present");
        let text = body.text.expect("text content present");
        assert_eq!(text.text, "Hello Keep");
    }

    // ------------------------------------------------------------------
    // test_note_display_title
    // ------------------------------------------------------------------
    #[test]
    fn test_note_display_title() {
        let note = Note {
            name: "notes/x".to_string(),
            title: "Shopping list".to_string(),
            body: None,
            create_time: None,
            update_time: None,
            trashed: false,
            permissions: vec![],
        };
        assert_eq!(note.display_title(), "Shopping list");
    }

    // ------------------------------------------------------------------
    // test_note_display_title_missing
    // ------------------------------------------------------------------
    #[test]
    fn test_note_display_title_missing() {
        let note = Note {
            name: "notes/y".to_string(),
            title: String::new(),
            body: None,
            create_time: None,
            update_time: None,
            trashed: false,
            permissions: vec![],
        };
        assert_eq!(note.display_title(), "(Untitled)");
    }

    // ------------------------------------------------------------------
    // test_note_body_text
    // ------------------------------------------------------------------
    #[test]
    fn test_note_body_text() {
        let note = Note {
            name: "notes/t1".to_string(),
            title: "Text note".to_string(),
            body: Some(NoteBody {
                text: Some(TextContent {
                    text: "Plain text content".to_string(),
                }),
                list: None,
            }),
            create_time: None,
            update_time: None,
            trashed: false,
            permissions: vec![],
        };
        assert_eq!(note.body_text(), Some("Plain text content".to_string()));
    }

    // ------------------------------------------------------------------
    // test_note_body_list
    // ------------------------------------------------------------------
    #[test]
    fn test_note_body_list() {
        let note = Note {
            name: "notes/l1".to_string(),
            title: "List note".to_string(),
            body: Some(NoteBody {
                text: None,
                list: Some(ListContent {
                    list_items: vec![
                        ListItem {
                            text: TextContent {
                                text: "Buy milk".to_string(),
                            },
                            checked: true,
                            children_list_items: vec![],
                        },
                        ListItem {
                            text: TextContent {
                                text: "Buy eggs".to_string(),
                            },
                            checked: false,
                            children_list_items: vec![],
                        },
                    ],
                }),
            }),
            create_time: None,
            update_time: None,
            trashed: false,
            permissions: vec![],
        };
        let text = note.body_text().expect("body_text for list");
        assert_eq!(text, "[x] Buy milk\n[ ] Buy eggs");
    }

    // ------------------------------------------------------------------
    // test_note_is_list
    // ------------------------------------------------------------------
    #[test]
    fn test_note_is_list() {
        let list_note = Note {
            name: "notes/l2".to_string(),
            title: "List".to_string(),
            body: Some(NoteBody {
                text: None,
                list: Some(ListContent { list_items: vec![] }),
            }),
            create_time: None,
            update_time: None,
            trashed: false,
            permissions: vec![],
        };
        assert!(list_note.is_list());

        let text_note = Note {
            name: "notes/t2".to_string(),
            title: "Text".to_string(),
            body: Some(NoteBody {
                text: Some(TextContent {
                    text: "hi".to_string(),
                }),
                list: None,
            }),
            create_time: None,
            update_time: None,
            trashed: false,
            permissions: vec![],
        };
        assert!(!text_note.is_list());

        let no_body_note = Note {
            name: "notes/nb".to_string(),
            title: "No body".to_string(),
            body: None,
            create_time: None,
            update_time: None,
            trashed: false,
            permissions: vec![],
        };
        assert!(!no_body_note.is_list());
    }

    // ------------------------------------------------------------------
    // test_list_item_display_checked
    // ------------------------------------------------------------------
    #[test]
    fn test_list_item_display_checked() {
        let item = ListItem {
            text: TextContent {
                text: "Done task".to_string(),
            },
            checked: true,
            children_list_items: vec![],
        };
        assert_eq!(item.display_text(), "[x] Done task");
    }

    // ------------------------------------------------------------------
    // test_list_item_display_unchecked
    // ------------------------------------------------------------------
    #[test]
    fn test_list_item_display_unchecked() {
        let item = ListItem {
            text: TextContent {
                text: "Pending task".to_string(),
            },
            checked: false,
            children_list_items: vec![],
        };
        assert_eq!(item.display_text(), "[ ] Pending task");
    }

    // ------------------------------------------------------------------
    // test_note_list_deserialize
    // ------------------------------------------------------------------
    #[test]
    fn test_note_list_deserialize() {
        let json = r#"{
            "notes": [
                {
                    "name": "notes/n1",
                    "title": "First note",
                    "body": { "text": { "text": "Content one" } },
                    "createTime": "2024-03-01T10:00:00Z",
                    "updateTime": "2024-03-01T11:00:00Z",
                    "trashed": false,
                    "permissions": []
                },
                {
                    "name": "notes/n2",
                    "title": "",
                    "trashed": true,
                    "permissions": []
                }
            ],
            "nextPageToken": "page_xyz"
        }"#;

        let list: NoteList = serde_json::from_str(json).expect("deserialize note list");
        assert_eq!(list.notes.len(), 2);
        assert_eq!(list.notes[0].name, "notes/n1");
        assert_eq!(list.notes[0].title, "First note");
        assert!(!list.notes[0].trashed);
        assert_eq!(list.notes[1].name, "notes/n2");
        assert!(list.notes[1].trashed);
        assert_eq!(list.next_page_token.as_deref(), Some("page_xyz"));
    }
}
