// gog-auth oauth module
// Simplified OAuth 2.0 configuration and authorization URL builder.
// Ported from internal/googleauth/oauth_flow.go

use serde::Deserialize;

use crate::scopes::AuthError;

const AUTH_ENDPOINT: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const TOKEN_ENDPOINT: &str = "https://oauth2.googleapis.com/token";

// ---------------------------------------------------------------------------
// OAuthConfig
// ---------------------------------------------------------------------------

pub struct OAuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub scopes: Vec<String>,
    pub redirect_uri: String,
}

impl OAuthConfig {
    /// Build the Google OAuth authorization URL.
    pub fn auth_url(&self) -> String {
        let scope_str = self.scopes.join(" ");
        format!(
            "{}?client_id={}&redirect_uri={}&response_type=code&scope={}&access_type=offline&include_granted_scopes=true",
            AUTH_ENDPOINT,
            urlencoding::encode(&self.client_id),
            urlencoding::encode(&self.redirect_uri),
            urlencoding::encode(&scope_str),
        )
    }

    /// Exchange an authorization code for tokens.
    pub async fn exchange_code(&self, code: &str) -> Result<TokenResponse, AuthError> {
        let client = reqwest::Client::new();

        let params = [
            ("code", code),
            ("client_id", &self.client_id),
            ("client_secret", &self.client_secret),
            ("redirect_uri", &self.redirect_uri),
            ("grant_type", "authorization_code"),
        ];

        let resp = client
            .post(TOKEN_ENDPOINT)
            .form(&params)
            .send()
            .await?
            .json::<TokenResponse>()
            .await?;

        Ok(resp)
    }
}

// ---------------------------------------------------------------------------
// TokenResponse
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: Option<u64>,
    pub token_type: String,
}

// ---------------------------------------------------------------------------
// URL encoding helper (minimal, avoids extra dependency bloat)
// ---------------------------------------------------------------------------

mod urlencoding {
    pub fn encode(s: &str) -> String {
        url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_config() -> OAuthConfig {
        OAuthConfig {
            client_id: "test-client-id".to_string(),
            client_secret: "test-secret".to_string(),
            scopes: vec![
                "https://www.googleapis.com/auth/gmail.modify".to_string(),
                "openid".to_string(),
            ],
            redirect_uri: "http://127.0.0.1:8080/callback".to_string(),
        }
    }

    #[test]
    fn test_auth_url_contains_scopes() {
        let config = make_config();
        let url = config.auth_url();
        assert!(url.contains("scope="), "URL should contain scope parameter");
        assert!(url.contains("gmail"), "URL should contain gmail scope");
    }

    #[test]
    fn test_auth_url_contains_client_id() {
        let config = make_config();
        let url = config.auth_url();
        assert!(
            url.contains("test-client-id"),
            "URL should contain the client_id"
        );
    }

    #[test]
    fn test_auth_url_has_redirect_uri() {
        let config = make_config();
        let url = config.auth_url();
        assert!(
            url.contains("redirect_uri="),
            "URL should contain redirect_uri parameter"
        );
        assert!(
            url.contains("127.0.0.1"),
            "URL should contain the redirect URI host"
        );
    }
}
