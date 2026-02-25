// gog-auth scopes module
// Ported from internal/googleauth/service.go

use serde::{Deserialize, Serialize};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("unknown service: {0}")]
    UnknownService(String),
    #[error("invalid drive scope: {0}")]
    InvalidDriveScope(String),
    #[error("OAuth flow failed: {0}")]
    OAuthFlow(String),
    #[error(transparent)]
    Secrets(#[from] gog_secrets::SecretsError),
    #[error(transparent)]
    Http(#[from] reqwest::Error),
}

// ---------------------------------------------------------------------------
// Service enum
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Service {
    Gmail,
    Calendar,
    Drive,
    Contacts,
    Chat,
    Keep,
    Forms,
}

impl Service {
    /// Parse a service from a string (case-insensitive).
    pub fn parse(s: &str) -> Result<Service, AuthError> {
        match s.trim().to_lowercase().as_str() {
            "gmail" => Ok(Service::Gmail),
            "calendar" => Ok(Service::Calendar),
            "drive" => Ok(Service::Drive),
            "contacts" => Ok(Service::Contacts),
            "chat" => Ok(Service::Chat),
            "keep" => Ok(Service::Keep),
            "forms" => Ok(Service::Forms),
            other => Err(AuthError::UnknownService(other.to_string())),
        }
    }

    /// Return the lowercase string name of this service.
    pub fn as_str(&self) -> &str {
        match self {
            Service::Gmail => "gmail",
            Service::Calendar => "calendar",
            Service::Drive => "drive",
            Service::Contacts => "contacts",
            Service::Chat => "chat",
            Service::Keep => "keep",
            Service::Forms => "forms",
        }
    }

    /// Return all 7 services in the canonical order.
    pub fn all() -> Vec<Service> {
        vec![
            Service::Gmail,
            Service::Calendar,
            Service::Drive,
            Service::Contacts,
            Service::Chat,
            Service::Keep,
            Service::Forms,
        ]
    }

    /// Return the services that use user OAuth (all except Keep).
    pub fn user_services() -> Vec<Service> {
        Service::all()
            .into_iter()
            .filter(|s| s.is_user_service())
            .collect()
    }

    /// Return true if this service uses user OAuth (not service-account-only).
    pub fn is_user_service(&self) -> bool {
        !matches!(self, Service::Keep)
    }

    /// Return the default OAuth scopes for this service (full URLs).
    pub fn scopes(&self) -> Vec<&'static str> {
        match self {
            Service::Gmail => vec![
                "https://www.googleapis.com/auth/gmail.modify",
                "https://www.googleapis.com/auth/gmail.settings.basic",
                "https://www.googleapis.com/auth/gmail.settings.sharing",
            ],
            Service::Calendar => vec!["https://www.googleapis.com/auth/calendar"],
            Service::Drive => vec!["https://www.googleapis.com/auth/drive"],
            Service::Contacts => vec![
                "https://www.googleapis.com/auth/contacts",
                "https://www.googleapis.com/auth/contacts.other.readonly",
                "https://www.googleapis.com/auth/directory.readonly",
            ],
            Service::Chat => vec![
                "https://www.googleapis.com/auth/chat.spaces",
                "https://www.googleapis.com/auth/chat.messages",
                "https://www.googleapis.com/auth/chat.memberships",
                "https://www.googleapis.com/auth/chat.users.readstate.readonly",
            ],
            Service::Keep => vec!["https://www.googleapis.com/auth/keep.readonly"],
            Service::Forms => vec![
                "https://www.googleapis.com/auth/forms.body",
                "https://www.googleapis.com/auth/forms.responses.readonly",
            ],
        }
    }

    /// Return API names for display.
    pub fn apis(&self) -> Vec<&'static str> {
        match self {
            Service::Gmail => vec!["Gmail API"],
            Service::Calendar => vec!["Calendar API"],
            Service::Drive => vec!["Drive API"],
            Service::Contacts => vec!["People API"],
            Service::Chat => vec!["Chat API"],
            Service::Keep => vec!["Keep API"],
            Service::Forms => vec!["Forms API"],
        }
    }
}

// ---------------------------------------------------------------------------
// ScopeOptions
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct ScopeOptions {
    pub readonly: bool,
    pub drive_scope: DriveScopeMode,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum DriveScopeMode {
    #[default]
    Full,
    Readonly,
    File,
}

// ---------------------------------------------------------------------------
// Scope collection helpers
// ---------------------------------------------------------------------------

/// Collect unique scopes for the given services, sorted.
pub fn scopes_for_services(services: &[Service]) -> Vec<String> {
    let mut set = std::collections::BTreeSet::new();
    for svc in services {
        for scope in svc.scopes() {
            set.insert(scope.to_string());
        }
    }
    set.into_iter().collect()
}

/// Collect scopes for the given services, plus the management scopes
/// (openid, email, userinfo.email), sorted.
pub fn scopes_for_manage(services: &[Service]) -> Vec<String> {
    let mut set = std::collections::BTreeSet::new();
    for svc in services {
        for scope in svc.scopes() {
            set.insert(scope.to_string());
        }
    }
    set.insert("openid".to_string());
    set.insert("email".to_string());
    set.insert("https://www.googleapis.com/auth/userinfo.email".to_string());
    set.into_iter().collect()
}

/// Apply ScopeOptions (readonly flag, drive scope mode) to scope collection.
pub fn scopes_for_services_with_options(services: &[Service], opts: &ScopeOptions) -> Vec<String> {
    let mut set = std::collections::BTreeSet::new();
    for svc in services {
        let scopes = scopes_for_service_with_options(svc, opts);
        for scope in scopes {
            set.insert(scope);
        }
    }
    set.into_iter().collect()
}

fn drive_scope_value(opts: &ScopeOptions) -> &'static str {
    if opts.readonly {
        return "https://www.googleapis.com/auth/drive.readonly";
    }
    match opts.drive_scope {
        DriveScopeMode::File => "https://www.googleapis.com/auth/drive.file",
        DriveScopeMode::Readonly => "https://www.googleapis.com/auth/drive.readonly",
        DriveScopeMode::Full => "https://www.googleapis.com/auth/drive",
    }
}

fn scopes_for_service_with_options(service: &Service, opts: &ScopeOptions) -> Vec<String> {
    match service {
        Service::Gmail => {
            if opts.readonly {
                vec!["https://www.googleapis.com/auth/gmail.readonly".to_string()]
            } else {
                service.scopes().iter().map(|s| s.to_string()).collect()
            }
        }
        Service::Calendar => {
            if opts.readonly {
                vec!["https://www.googleapis.com/auth/calendar.readonly".to_string()]
            } else {
                service.scopes().iter().map(|s| s.to_string()).collect()
            }
        }
        Service::Drive => {
            vec![drive_scope_value(opts).to_string()]
        }
        Service::Contacts => {
            let contacts_scope = if opts.readonly {
                "https://www.googleapis.com/auth/contacts.readonly"
            } else {
                "https://www.googleapis.com/auth/contacts"
            };
            vec![
                contacts_scope.to_string(),
                "https://www.googleapis.com/auth/contacts.other.readonly".to_string(),
                "https://www.googleapis.com/auth/directory.readonly".to_string(),
            ]
        }
        Service::Chat => {
            if opts.readonly {
                vec![
                    "https://www.googleapis.com/auth/chat.spaces.readonly".to_string(),
                    "https://www.googleapis.com/auth/chat.messages.readonly".to_string(),
                    "https://www.googleapis.com/auth/chat.memberships.readonly".to_string(),
                    "https://www.googleapis.com/auth/chat.users.readstate.readonly".to_string(),
                ]
            } else {
                service.scopes().iter().map(|s| s.to_string()).collect()
            }
        }
        Service::Keep => service.scopes().iter().map(|s| s.to_string()).collect(),
        Service::Forms => {
            let body_scope = if opts.readonly {
                "https://www.googleapis.com/auth/forms.body.readonly"
            } else {
                "https://www.googleapis.com/auth/forms.body"
            };
            vec![
                body_scope.to_string(),
                "https://www.googleapis.com/auth/forms.responses.readonly".to_string(),
            ]
        }
    }
}

/// Return a comma-separated string of user service names.
pub fn user_service_csv() -> String {
    Service::user_services()
        .iter()
        .map(|s| s.as_str())
        .collect::<Vec<_>>()
        .join(",")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_from_str() {
        assert_eq!(Service::parse("gmail").unwrap(), Service::Gmail);
        assert_eq!(Service::parse("CALENDAR").unwrap(), Service::Calendar);
        assert_eq!(Service::parse("Drive").unwrap(), Service::Drive);
        assert_eq!(Service::parse("contacts").unwrap(), Service::Contacts);
        assert_eq!(Service::parse("chat").unwrap(), Service::Chat);
        assert_eq!(Service::parse("keep").unwrap(), Service::Keep);
        assert_eq!(Service::parse("forms").unwrap(), Service::Forms);
    }

    #[test]
    fn test_service_from_str_invalid() {
        assert!(matches!(
            Service::parse("unknown"),
            Err(AuthError::UnknownService(_))
        ));
    }

    #[test]
    fn test_service_as_str() {
        assert_eq!(Service::Gmail.as_str(), "gmail");
        assert_eq!(Service::Calendar.as_str(), "calendar");
        assert_eq!(Service::Drive.as_str(), "drive");
        assert_eq!(Service::Contacts.as_str(), "contacts");
        assert_eq!(Service::Chat.as_str(), "chat");
        assert_eq!(Service::Keep.as_str(), "keep");
        assert_eq!(Service::Forms.as_str(), "forms");
    }

    #[test]
    fn test_service_all_count() {
        assert_eq!(Service::all().len(), 7);
    }

    #[test]
    fn test_service_user_services_excludes_keep() {
        let user_svcs = Service::user_services();
        assert!(!user_svcs.contains(&Service::Keep));
        assert_eq!(user_svcs.len(), 6);
    }

    #[test]
    fn test_gmail_scopes() {
        let scopes = Service::Gmail.scopes();
        assert!(scopes.contains(&"https://www.googleapis.com/auth/gmail.modify"));
        assert!(scopes.contains(&"https://www.googleapis.com/auth/gmail.settings.basic"));
        assert!(scopes.contains(&"https://www.googleapis.com/auth/gmail.settings.sharing"));
    }

    #[test]
    fn test_calendar_scopes() {
        let scopes = Service::Calendar.scopes();
        assert_eq!(scopes.len(), 1);
        assert_eq!(scopes[0], "https://www.googleapis.com/auth/calendar");
    }

    #[test]
    fn test_contacts_scopes() {
        let scopes = Service::Contacts.scopes();
        assert_eq!(scopes.len(), 3);
        assert!(scopes.contains(&"https://www.googleapis.com/auth/contacts"));
        assert!(scopes.contains(&"https://www.googleapis.com/auth/contacts.other.readonly"));
        assert!(scopes.contains(&"https://www.googleapis.com/auth/directory.readonly"));
    }

    #[test]
    fn test_chat_scopes() {
        let scopes = Service::Chat.scopes();
        assert_eq!(scopes.len(), 4);
        assert!(scopes.contains(&"https://www.googleapis.com/auth/chat.spaces"));
        assert!(scopes.contains(&"https://www.googleapis.com/auth/chat.messages"));
        assert!(scopes.contains(&"https://www.googleapis.com/auth/chat.memberships"));
        assert!(scopes.contains(
            &"https://www.googleapis.com/auth/chat.users.readstate.readonly"
        ));
    }

    #[test]
    fn test_scopes_for_services_deduplication() {
        // Providing the same service twice must not duplicate scopes.
        let scopes_once = scopes_for_services(&[Service::Gmail]);
        let scopes_twice = scopes_for_services(&[Service::Gmail, Service::Gmail]);
        assert_eq!(scopes_once, scopes_twice);
    }

    #[test]
    fn test_scopes_for_services_sorted() {
        let scopes = scopes_for_services(&[Service::Gmail, Service::Calendar, Service::Chat]);
        let mut sorted = scopes.clone();
        sorted.sort();
        assert_eq!(scopes, sorted, "scopes_for_services should return sorted output");
    }

    #[test]
    fn test_scopes_for_manage_includes_openid() {
        let scopes = scopes_for_manage(&[Service::Gmail]);
        assert!(scopes.contains(&"openid".to_string()), "should include openid");
        assert!(scopes.contains(&"email".to_string()), "should include email");
        assert!(
            scopes.contains(&"https://www.googleapis.com/auth/userinfo.email".to_string()),
            "should include userinfo.email"
        );
    }

    #[test]
    fn test_user_service_csv() {
        let csv = user_service_csv();
        // Should contain all user services (6 total), comma-separated.
        let parts: Vec<&str> = csv.split(',').collect();
        assert_eq!(parts.len(), 6, "expected 6 user services in CSV");
        assert!(csv.contains("gmail"));
        assert!(csv.contains("calendar"));
        assert!(!csv.contains("keep"), "Keep should NOT be in user_service_csv");
    }

    #[test]
    fn test_scope_options_readonly_gmail() {
        let opts = ScopeOptions {
            readonly: true,
            drive_scope: DriveScopeMode::Full,
        };
        let scopes = scopes_for_services_with_options(&[Service::Gmail], &opts);
        assert_eq!(scopes, vec!["https://www.googleapis.com/auth/gmail.readonly"]);
    }

    #[test]
    fn test_scope_options_drive_file() {
        let opts = ScopeOptions {
            readonly: false,
            drive_scope: DriveScopeMode::File,
        };
        let scopes = scopes_for_services_with_options(&[Service::Drive], &opts);
        assert_eq!(scopes, vec!["https://www.googleapis.com/auth/drive.file"]);
    }
}
