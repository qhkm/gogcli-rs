// Google Chat API v1 data types
// Matches Google Chat API v1 JSON response shapes.
// Reference: https://chat.googleapis.com/$discovery/rest?version=v1

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Space
// ---------------------------------------------------------------------------

/// A Google Chat space (room, DM, or group).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Space {
    /// Resource name of the space, e.g. `spaces/AAAABBBBBBB`.
    pub name: String,
    /// Human-readable display name of the space (absent for DMs).
    #[serde(default)]
    pub display_name: String,
    /// Type of the space: `ROOM`, `DM`, or `GROUP_CHAT`.
    #[serde(default)]
    pub space_type: String,
    /// True when the space is a 1:1 DM with a bot.
    #[serde(default)]
    pub single_user_bot_dm: bool,
    /// True when the space supports threaded replies.
    #[serde(default)]
    pub threaded: bool,
    /// True when the space was created by a Google Workspace admin.
    #[serde(default)]
    pub admin_installed: bool,
}

impl Space {
    /// Return the display name if non-empty, otherwise return the resource
    /// name (the last path segment, e.g. `AAAABBBBBBB`).
    pub fn display_name_or_id(&self) -> &str {
        if !self.display_name.is_empty() {
            &self.display_name
        } else {
            // Fall back to the short ID portion of the resource name.
            self.name.rsplit('/').next().unwrap_or(self.name.as_str())
        }
    }

    /// Return true when this space is a direct-message conversation.
    pub fn is_dm(&self) -> bool {
        self.space_type == "DM" || self.single_user_bot_dm
    }
}

// ---------------------------------------------------------------------------
// SpaceList
// ---------------------------------------------------------------------------

/// Paginated list of spaces returned by the Chat API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpaceList {
    #[serde(default)]
    pub spaces: Vec<Space>,
    pub next_page_token: Option<String>,
}

// ---------------------------------------------------------------------------
// Sender / User
// ---------------------------------------------------------------------------

/// The user or bot that sent a Chat message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Sender {
    /// Resource name of the user, e.g. `users/1234567890`.
    pub name: String,
    /// Human-readable display name.
    #[serde(default)]
    pub display_name: String,
    /// Either `"HUMAN"` or `"BOT"`.
    #[serde(rename = "type", default)]
    pub type_: String,
}

// ---------------------------------------------------------------------------
// ChatThread
// ---------------------------------------------------------------------------

/// A thread within a Chat space.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatThread {
    /// Resource name of the thread, e.g. `spaces/AAA/threads/BBB`.
    pub name: String,
    /// Optional client-assigned thread key.
    #[serde(default)]
    pub thread_key: String,
}

// ---------------------------------------------------------------------------
// ChatMessage
// ---------------------------------------------------------------------------

/// A message in a Google Chat space.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    /// Resource name, e.g. `spaces/AAA/messages/BBB`.
    pub name: String,
    /// Who sent the message.
    pub sender: Option<Sender>,
    /// RFC 3339 creation timestamp.
    #[serde(default)]
    pub create_time: String,
    /// Plain-text body of the message.
    #[serde(default)]
    pub text: String,
    /// Thread this message belongs to.
    pub thread: Option<ChatThread>,
    /// The space this message was posted in.
    pub space: Option<SpaceRef>,
    /// HTML-formatted version of the body (server-populated).
    #[serde(default)]
    pub formatted_text: String,
    /// Attached card widgets (JSON blob; kept as raw value for flexibility).
    pub cards: Option<serde_json::Value>,
}

impl ChatMessage {
    /// Return a truncated preview of the message text.
    ///
    /// If `text` is shorter than or equal to `max_len`, it is returned
    /// unchanged. Otherwise the first `max_len` characters are returned
    /// followed by `"..."`.
    pub fn text_preview(&self, max_len: usize) -> String {
        let chars: Vec<char> = self.text.chars().collect();
        if chars.len() <= max_len {
            self.text.clone()
        } else {
            let truncated: String = chars[..max_len].iter().collect();
            format!("{}...", truncated)
        }
    }
}

/// Lightweight space reference embedded inside a message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpaceRef {
    pub name: String,
}

// ---------------------------------------------------------------------------
// MessageList
// ---------------------------------------------------------------------------

/// Paginated list of messages returned by the Chat API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageList {
    #[serde(default)]
    pub messages: Vec<ChatMessage>,
    pub next_page_token: Option<String>,
}

// ---------------------------------------------------------------------------
// Member / MemberUser
// ---------------------------------------------------------------------------

/// A user record embedded in a membership resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemberUser {
    /// Resource name of the user, e.g. `users/1234567890`.
    pub name: String,
    /// Human-readable display name.
    #[serde(default)]
    pub display_name: String,
    /// Either `"HUMAN"` or `"BOT"`.
    #[serde(rename = "type", default)]
    pub type_: String,
}

/// A membership entry in a Chat space.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Member {
    /// Resource name of the membership, e.g. `spaces/AAA/members/BBB`.
    pub name: String,
    /// The member's user record.
    pub member: Option<MemberUser>,
    /// Role of the member: `"ROLE_MEMBER"` or `"ROLE_MANAGER"`.
    #[serde(default)]
    pub role: String,
    /// Membership state: `"JOINED"`, `"INVITED"`, or `"NOT_A_MEMBER"`.
    #[serde(default)]
    pub state: String,
    /// RFC 3339 timestamp when the membership was created.
    #[serde(default)]
    pub create_time: String,
}

// ---------------------------------------------------------------------------
// MemberList
// ---------------------------------------------------------------------------

/// Paginated list of memberships returned by the Chat API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemberList {
    #[serde(default)]
    pub memberships: Vec<Member>,
    pub next_page_token: Option<String>,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Space tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_space_deserialize() {
        let json = r#"{
            "name": "spaces/AAAABBBBBBB",
            "displayName": "Engineering",
            "spaceType": "ROOM",
            "singleUserBotDm": false,
            "threaded": true,
            "adminInstalled": false
        }"#;

        let space: Space = serde_json::from_str(json).expect("deserialize space");
        assert_eq!(space.name, "spaces/AAAABBBBBBB");
        assert_eq!(space.display_name, "Engineering");
        assert_eq!(space.space_type, "ROOM");
        assert!(!space.single_user_bot_dm);
        assert!(space.threaded);
        assert!(!space.admin_installed);
    }

    #[test]
    fn test_space_is_dm() {
        let dm_space = Space {
            name: "spaces/DM123".to_string(),
            display_name: String::new(),
            space_type: "DM".to_string(),
            single_user_bot_dm: false,
            threaded: false,
            admin_installed: false,
        };
        assert!(dm_space.is_dm(), "space_type DM should report is_dm=true");

        let bot_dm = Space {
            name: "spaces/BOT456".to_string(),
            display_name: String::new(),
            space_type: String::new(),
            single_user_bot_dm: true,
            threaded: false,
            admin_installed: false,
        };
        assert!(
            bot_dm.is_dm(),
            "single_user_bot_dm=true should report is_dm=true"
        );

        let room = Space {
            name: "spaces/ROOM789".to_string(),
            display_name: "Team Room".to_string(),
            space_type: "ROOM".to_string(),
            single_user_bot_dm: false,
            threaded: true,
            admin_installed: false,
        };
        assert!(!room.is_dm(), "ROOM should not be a DM");
    }

    #[test]
    fn test_space_display_name() {
        // Has a display name - should return it.
        let named = Space {
            name: "spaces/AAA".to_string(),
            display_name: "My Space".to_string(),
            space_type: "ROOM".to_string(),
            single_user_bot_dm: false,
            threaded: false,
            admin_installed: false,
        };
        assert_eq!(named.display_name_or_id(), "My Space");

        // No display name (DM) - should fall back to the ID segment.
        let unnamed = Space {
            name: "spaces/XYZID".to_string(),
            display_name: String::new(),
            space_type: "DM".to_string(),
            single_user_bot_dm: false,
            threaded: false,
            admin_installed: false,
        };
        assert_eq!(unnamed.display_name_or_id(), "XYZID");
    }

    // -----------------------------------------------------------------------
    // SpaceList tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_space_list_deserialize() {
        let json = r#"{
            "spaces": [
                {
                    "name": "spaces/AAA",
                    "displayName": "Alpha",
                    "spaceType": "ROOM"
                },
                {
                    "name": "spaces/BBB",
                    "displayName": "",
                    "spaceType": "DM"
                }
            ],
            "nextPageToken": "tok_xyz"
        }"#;

        let list: SpaceList = serde_json::from_str(json).expect("deserialize space list");
        assert_eq!(list.spaces.len(), 2);
        assert_eq!(list.spaces[0].name, "spaces/AAA");
        assert_eq!(list.spaces[0].display_name, "Alpha");
        assert_eq!(list.spaces[1].space_type, "DM");
        assert_eq!(list.next_page_token.as_deref(), Some("tok_xyz"));
    }

    #[test]
    fn test_space_list_empty() {
        let json = r#"{}"#;
        let list: SpaceList = serde_json::from_str(json).expect("deserialize empty space list");
        assert!(list.spaces.is_empty());
        assert!(list.next_page_token.is_none());
    }

    // -----------------------------------------------------------------------
    // Sender tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_sender_deserialize() {
        // Human sender
        let json = r#"{
            "name": "users/987654321",
            "displayName": "Alice Smith",
            "type": "HUMAN"
        }"#;

        let sender: Sender = serde_json::from_str(json).expect("deserialize human sender");
        assert_eq!(sender.name, "users/987654321");
        assert_eq!(sender.display_name, "Alice Smith");
        assert_eq!(sender.type_, "HUMAN");

        // Bot sender
        let bot_json = r#"{
            "name": "users/123",
            "displayName": "MyBot",
            "type": "BOT"
        }"#;

        let bot: Sender = serde_json::from_str(bot_json).expect("deserialize bot sender");
        assert_eq!(bot.type_, "BOT");
        assert_eq!(bot.display_name, "MyBot");
    }

    // -----------------------------------------------------------------------
    // ChatMessage tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_message_deserialize() {
        let json = r#"{
            "name": "spaces/AAA/messages/MSG001",
            "sender": {
                "name": "users/111",
                "displayName": "Bob",
                "type": "HUMAN"
            },
            "createTime": "2024-01-15T10:30:00Z",
            "text": "Hello, world!",
            "thread": {
                "name": "spaces/AAA/threads/T001",
                "threadKey": "my-thread"
            },
            "space": {
                "name": "spaces/AAA"
            },
            "formattedText": "<b>Hello</b>, world!"
        }"#;

        let msg: ChatMessage = serde_json::from_str(json).expect("deserialize message");
        assert_eq!(msg.name, "spaces/AAA/messages/MSG001");
        assert_eq!(msg.text, "Hello, world!");
        assert_eq!(msg.create_time, "2024-01-15T10:30:00Z");
        assert_eq!(msg.formatted_text, "<b>Hello</b>, world!");

        let sender = msg.sender.expect("sender present");
        assert_eq!(sender.display_name, "Bob");
        assert_eq!(sender.type_, "HUMAN");

        let thread = msg.thread.expect("thread present");
        assert_eq!(thread.name, "spaces/AAA/threads/T001");
        assert_eq!(thread.thread_key, "my-thread");

        let space = msg.space.expect("space ref present");
        assert_eq!(space.name, "spaces/AAA");
    }

    #[test]
    fn test_message_text_preview() {
        let msg = ChatMessage {
            name: "spaces/AAA/messages/1".to_string(),
            sender: None,
            create_time: String::new(),
            text: "This is a longer message that should be truncated by the preview function."
                .to_string(),
            thread: None,
            space: None,
            formatted_text: String::new(),
            cards: None,
        };

        let preview = msg.text_preview(20);
        assert_eq!(preview, "This is a longer mes...");
        assert!(
            preview.ends_with("..."),
            "truncated preview must end with '...'"
        );
    }

    #[test]
    fn test_message_text_preview_short() {
        let msg = ChatMessage {
            name: "spaces/AAA/messages/2".to_string(),
            sender: None,
            create_time: String::new(),
            text: "Hi".to_string(),
            thread: None,
            space: None,
            formatted_text: String::new(),
            cards: None,
        };

        // Text shorter than max_len - should not be truncated.
        let preview = msg.text_preview(50);
        assert_eq!(preview, "Hi");
        assert!(
            !preview.ends_with("..."),
            "short text must NOT be truncated"
        );
    }

    // -----------------------------------------------------------------------
    // ChatThread tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_thread_deserialize() {
        let json = r#"{
            "name": "spaces/SPACE123/threads/THREAD456",
            "threadKey": "ticket-42"
        }"#;

        let thread: ChatThread = serde_json::from_str(json).expect("deserialize thread");
        assert_eq!(thread.name, "spaces/SPACE123/threads/THREAD456");
        assert_eq!(thread.thread_key, "ticket-42");
    }

    #[test]
    fn test_thread_deserialize_minimal() {
        // threadKey is optional; should default to empty string.
        let json = r#"{"name": "spaces/S/threads/T"}"#;
        let thread: ChatThread = serde_json::from_str(json).expect("deserialize minimal thread");
        assert_eq!(thread.name, "spaces/S/threads/T");
        assert_eq!(thread.thread_key, "");
    }

    // -----------------------------------------------------------------------
    // Member tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_member_deserialize() {
        let json = r#"{
            "name": "spaces/AAA/members/USER999",
            "member": {
                "name": "users/999",
                "displayName": "Charlie",
                "type": "HUMAN"
            },
            "role": "ROLE_MANAGER",
            "state": "JOINED",
            "createTime": "2023-06-01T08:00:00Z"
        }"#;

        let member: Member = serde_json::from_str(json).expect("deserialize member");
        assert_eq!(member.name, "spaces/AAA/members/USER999");
        assert_eq!(member.role, "ROLE_MANAGER");
        assert_eq!(member.state, "JOINED");
        assert_eq!(member.create_time, "2023-06-01T08:00:00Z");

        let user = member.member.expect("member user present");
        assert_eq!(user.name, "users/999");
        assert_eq!(user.display_name, "Charlie");
        assert_eq!(user.type_, "HUMAN");
    }

    #[test]
    fn test_member_list_deserialize() {
        let json = r#"{
            "memberships": [
                {
                    "name": "spaces/S/members/M1",
                    "role": "ROLE_MEMBER",
                    "state": "JOINED"
                },
                {
                    "name": "spaces/S/members/M2",
                    "role": "ROLE_MANAGER",
                    "state": "JOINED"
                }
            ],
            "nextPageToken": "page2"
        }"#;

        let list: MemberList = serde_json::from_str(json).expect("deserialize member list");
        assert_eq!(list.memberships.len(), 2);
        assert_eq!(list.memberships[0].name, "spaces/S/members/M1");
        assert_eq!(list.memberships[0].role, "ROLE_MEMBER");
        assert_eq!(list.memberships[1].role, "ROLE_MANAGER");
        assert_eq!(list.next_page_token.as_deref(), Some("page2"));
    }
}
