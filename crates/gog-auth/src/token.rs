// gog-auth token module
// Token validation helpers.
// Ported from internal/googleauth/token_check.go

use crate::scopes::AuthError;

const TOKEN_ENDPOINT: &str = "https://oauth2.googleapis.com/token";

/// Attempt to refresh a token using the provided refresh_token.
///
/// Returns `Ok(true)` if the refresh succeeds, `Ok(false)` if the
/// server rejects the token (e.g. 400 / 401), and `Err` for network errors.
pub async fn validate_refresh_token(
    client_id: &str,
    client_secret: &str,
    refresh_token: &str,
) -> Result<bool, AuthError> {
    let client = reqwest::Client::new();

    let params = [
        ("client_id", client_id),
        ("client_secret", client_secret),
        ("refresh_token", refresh_token),
        ("grant_type", "refresh_token"),
    ];

    let resp = client.post(TOKEN_ENDPOINT).form(&params).send().await?;

    Ok(resp.status().is_success())
}
