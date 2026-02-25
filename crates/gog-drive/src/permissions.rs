// gog-drive permissions module
// Manage sharing permissions on Drive files.
// Ported from internal/drive/permissions.go

use reqwest::Client;
use serde_json::json;

use crate::types::{DriveError, Permission};

const FILES_ENDPOINT: &str = "https://www.googleapis.com/drive/v3/files";

// ---------------------------------------------------------------------------
// list_permissions
// ---------------------------------------------------------------------------

/// List all permissions on a Drive file or folder.
///
/// Calls `GET /drive/v3/files/{fileId}/permissions`.
pub async fn list_permissions(
    client: &Client,
    access_token: &str,
    file_id: &str,
) -> Result<Vec<Permission>, DriveError> {
    let url = format!("{}/{}/permissions", FILES_ENDPOINT, file_id);

    let resp = client
        .get(&url)
        .bearer_auth(access_token)
        .query(&[("fields", "permissions(id,role,type,emailAddress,displayName)")])
        .send()
        .await?;

    let status = resp.status().as_u16();
    if !resp.status().is_success() {
        let msg = resp.text().await.unwrap_or_default();
        return Err(DriveError::Api {
            status,
            message: msg,
        });
    }

    #[derive(serde::Deserialize)]
    struct PermissionList {
        #[serde(default)]
        permissions: Vec<Permission>,
    }

    let list: PermissionList = resp.json().await?;
    Ok(list.permissions)
}

// ---------------------------------------------------------------------------
// add_permission
// ---------------------------------------------------------------------------

/// Add a sharing permission to a Drive file.
///
/// Calls `POST /drive/v3/files/{fileId}/permissions`.
///
/// Common values:
/// - `role`: `"reader"`, `"commenter"`, `"writer"`, `"fileOrganizer"`, `"organizer"`, `"owner"`
/// - `type_`: `"user"`, `"group"`, `"domain"`, `"anyone"`
/// - `email_address`: required when `type_` is `"user"` or `"group"`
pub async fn add_permission(
    client: &Client,
    access_token: &str,
    file_id: &str,
    role: &str,
    type_: &str,
    email_address: Option<&str>,
) -> Result<Permission, DriveError> {
    let url = format!("{}/{}/permissions", FILES_ENDPOINT, file_id);

    let body = match email_address {
        Some(email) => json!({
            "role": role,
            "type": type_,
            "emailAddress": email,
        }),
        None => json!({
            "role": role,
            "type": type_,
        }),
    };

    let resp = client
        .post(&url)
        .bearer_auth(access_token)
        .query(&[("fields", "id,role,type,emailAddress,displayName")])
        .json(&body)
        .send()
        .await?;

    let status = resp.status().as_u16();
    if !resp.status().is_success() {
        let msg = resp.text().await.unwrap_or_default();
        return Err(DriveError::Api {
            status,
            message: msg,
        });
    }

    let perm: Permission = resp.json().await?;
    Ok(perm)
}

// ---------------------------------------------------------------------------
// remove_permission
// ---------------------------------------------------------------------------

/// Remove a permission from a Drive file.
///
/// Calls `DELETE /drive/v3/files/{fileId}/permissions/{permissionId}`.
pub async fn remove_permission(
    client: &Client,
    access_token: &str,
    file_id: &str,
    permission_id: &str,
) -> Result<(), DriveError> {
    let url = format!(
        "{}/{}/permissions/{}",
        FILES_ENDPOINT, file_id, permission_id
    );

    let resp = client
        .delete(&url)
        .bearer_auth(access_token)
        .send()
        .await?;

    let status = resp.status().as_u16();
    if !resp.status().is_success() {
        let msg = resp.text().await.unwrap_or_default();
        return Err(DriveError::Api {
            status,
            message: msg,
        });
    }

    Ok(())
}
