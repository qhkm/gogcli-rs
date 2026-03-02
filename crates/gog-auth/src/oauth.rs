// gog-auth oauth module
// OAuth 2.0 configuration, authorization URL builder, and local callback server.
// Ported from internal/googleauth/oauth_flow.go

use std::time::Duration;

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use rand::RngCore;
use serde::Deserialize;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;

use crate::scopes::AuthError;

const AUTH_ENDPOINT: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const TOKEN_ENDPOINT: &str = "https://oauth2.googleapis.com/token";
const CALLBACK_PATH: &str = "/oauth2/callback";

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
    /// Build the Google OAuth authorization URL with a state parameter.
    pub fn auth_url_with_state(&self, state: &str) -> String {
        let scope_str = self.scopes.join(" ");
        format!(
            "{}?client_id={}&redirect_uri={}&response_type=code&scope={}&access_type=offline&include_granted_scopes=true&prompt=consent&state={}",
            AUTH_ENDPOINT,
            urlencoding::encode(&self.client_id),
            urlencoding::encode(&self.redirect_uri),
            urlencoding::encode(&scope_str),
            urlencoding::encode(state),
        )
    }

    /// Build the Google OAuth authorization URL (without state).
    pub fn auth_url(&self) -> String {
        let scope_str = self.scopes.join(" ");
        format!(
            "{}?client_id={}&redirect_uri={}&response_type=code&scope={}&access_type=offline&include_granted_scopes=true&prompt=consent",
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
// Local callback server (mirrors Go's authorizeServer)
// ---------------------------------------------------------------------------

/// Result of a successful OAuth callback server flow.
#[derive(Debug)]
pub struct OAuthCallbackResult {
    pub code: String,
}

/// Generate a cryptographically random state string (32 bytes, base64url-encoded).
pub fn generate_state() -> String {
    let mut buf = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut buf);
    URL_SAFE_NO_PAD.encode(buf)
}

/// Run the full OAuth browser flow:
/// 1. Bind a local TCP listener on 127.0.0.1 (random port)
/// 2. Build the redirect URI and auth URL
/// 3. Open the browser
/// 4. Wait for the callback with the authorization code
/// 5. Exchange the code for tokens
///
/// Returns the OAuthConfig (with the actual redirect_uri) and the TokenResponse.
pub async fn authorize_with_browser(
    client_id: String,
    client_secret: String,
    scopes: Vec<String>,
    timeout: Duration,
) -> Result<(OAuthConfig, TokenResponse), AuthError> {
    // Bind on random port
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|e| AuthError::OAuthFlow(format!("failed to bind local server: {e}")))?;

    let local_addr = listener
        .local_addr()
        .map_err(|e| AuthError::OAuthFlow(format!("failed to get local address: {e}")))?;

    let port = local_addr.port();
    let redirect_uri = format!("http://127.0.0.1:{}{}", port, CALLBACK_PATH);

    let state = generate_state();

    let oauth = OAuthConfig {
        client_id,
        client_secret,
        scopes,
        redirect_uri,
    };

    let auth_url = oauth.auth_url_with_state(&state);

    // Open browser
    eprintln!("Opening browser for authorization...");
    eprintln!("If the browser doesn't open, visit this URL:\n");
    eprintln!("  {}\n", auth_url);

    if let Err(e) = open::that(&auth_url) {
        eprintln!("Warning: could not open browser: {e}");
        eprintln!("Please open the URL above manually.");
    }

    eprintln!("Waiting for authorization callback on port {port}...");

    // Wait for callback
    let result = tokio::time::timeout(timeout, accept_callback(&listener, &state)).await;

    match result {
        Ok(Ok(callback)) => {
            // Exchange code for tokens
            let token_resp = oauth.exchange_code(&callback.code).await?;
            Ok((oauth, token_resp))
        }
        Ok(Err(e)) => Err(e),
        Err(_) => Err(AuthError::OAuthFlow(
            "timed out waiting for authorization callback (2 minutes)".to_string(),
        )),
    }
}

/// Accept a single HTTP connection, parse the callback, validate state, extract code.
async fn accept_callback(
    listener: &TcpListener,
    expected_state: &str,
) -> Result<OAuthCallbackResult, AuthError> {
    loop {
        let (mut stream, _addr) = listener
            .accept()
            .await
            .map_err(|e| AuthError::OAuthFlow(format!("failed to accept connection: {e}")))?;

        // Read the HTTP request line
        let (reader, mut writer) = stream.split();
        let mut buf_reader = BufReader::new(reader);
        let mut request_line = String::new();
        buf_reader
            .read_line(&mut request_line)
            .await
            .map_err(|e| AuthError::OAuthFlow(format!("failed to read request: {e}")))?;

        // Parse: GET /oauth2/callback?code=xxx&state=yyy HTTP/1.1
        let path = request_line
            .split_whitespace()
            .nth(1)
            .unwrap_or("")
            .to_string();

        // Only handle the callback path
        if !path.starts_with(CALLBACK_PATH) {
            let response = "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n";
            let _ = writer.write_all(response.as_bytes()).await;
            continue;
        }

        // Parse query parameters
        let query_str = path.split_once('?').map(|(_, q)| q).unwrap_or("");

        let params: std::collections::HashMap<String, String> =
            url::form_urlencoded::parse(query_str.as_bytes())
                .into_owned()
                .collect();

        // Check for error from Google
        if let Some(err) = params.get("error") {
            let body = format!(
                "<html><body><h1>Authorization Failed</h1><p>Error: {}</p><p>You can close this tab.</p></body></html>",
                html_escape(err)
            );
            let response = format!(
                "HTTP/1.1 400 Bad Request\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = writer.write_all(response.as_bytes()).await;
            return Err(AuthError::OAuthFlow(format!("authorization denied: {err}")));
        }

        // Validate state (CSRF protection)
        let got_state = params.get("state").map(|s| s.as_str()).unwrap_or("");
        if got_state != expected_state {
            let body = "<html><body><h1>Error</h1><p>State mismatch. Possible CSRF attack.</p></body></html>";
            let response = format!(
                "HTTP/1.1 400 Bad Request\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = writer.write_all(response.as_bytes()).await;
            return Err(AuthError::OAuthFlow(
                "state mismatch in callback".to_string(),
            ));
        }

        // Extract code
        let code = params
            .get("code")
            .ok_or_else(|| AuthError::OAuthFlow("no authorization code in callback".to_string()))?
            .clone();

        // Send success response
        let body = "<html><body><h1>Authorization Successful</h1><p>You can close this tab and return to the terminal.</p></body></html>";
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n{}",
            body.len(),
            body
        );
        let _ = writer.write_all(response.as_bytes()).await;

        return Ok(OAuthCallbackResult { code });
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

// ---------------------------------------------------------------------------
// URL encoding helper
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
            redirect_uri: "http://127.0.0.1:8080/oauth2/callback".to_string(),
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

    #[test]
    fn test_auth_url_with_state() {
        let config = make_config();
        let url = config.auth_url_with_state("test-state-abc");
        assert!(
            url.contains("state=test-state-abc"),
            "URL should contain the state parameter"
        );
        assert!(url.contains("prompt=consent"), "URL should request consent");
    }

    #[test]
    fn test_generate_state_unique() {
        let s1 = generate_state();
        let s2 = generate_state();
        assert_ne!(s1, s2, "generated states should be unique");
        assert!(s1.len() > 20, "state should be sufficiently long");
    }

    #[test]
    fn test_generate_state_is_base64url() {
        let s = generate_state();
        // base64url has no +, /, or =
        assert!(!s.contains('+'), "should not contain +");
        assert!(!s.contains('/'), "should not contain /");
        assert!(!s.contains('='), "should not contain = (no padding)");
    }

    #[tokio::test]
    async fn test_callback_server_captures_code() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let state = "test-state-123";

        // Spawn the callback acceptor
        let state_owned = state.to_string();
        let handle = tokio::spawn(async move { accept_callback(&listener, &state_owned).await });

        // Simulate the browser redirect
        tokio::time::sleep(Duration::from_millis(50)).await;
        let url = format!(
            "http://127.0.0.1:{}/oauth2/callback?code=test-code-456&state={}",
            port, state
        );
        let resp = reqwest::get(&url).await.unwrap();
        assert_eq!(resp.status(), 200);

        let result = handle.await.unwrap().unwrap();
        assert_eq!(result.code, "test-code-456");
    }

    #[tokio::test]
    async fn test_callback_server_rejects_bad_state() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        let handle = tokio::spawn(async move { accept_callback(&listener, "correct-state").await });

        tokio::time::sleep(Duration::from_millis(50)).await;
        let url = format!(
            "http://127.0.0.1:{}/oauth2/callback?code=test-code&state=wrong-state",
            port
        );
        let resp = reqwest::get(&url).await.unwrap();
        assert_eq!(resp.status(), 400);

        let result = handle.await.unwrap();
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("state mismatch"),
            "expected state mismatch error, got: {err}"
        );
    }

    #[tokio::test]
    async fn test_callback_server_ignores_wrong_path() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let state = "test-state";

        let state_owned = state.to_string();
        let handle = tokio::spawn(async move { accept_callback(&listener, &state_owned).await });

        tokio::time::sleep(Duration::from_millis(50)).await;

        // Hit wrong path — should get 404, server keeps waiting
        let wrong_url = format!("http://127.0.0.1:{}/favicon.ico", port);
        let resp = reqwest::get(&wrong_url).await.unwrap();
        assert_eq!(resp.status(), 404);

        // Now hit the correct path
        let correct_url = format!(
            "http://127.0.0.1:{}/oauth2/callback?code=real-code&state={}",
            port, state
        );
        let resp = reqwest::get(&correct_url).await.unwrap();
        assert_eq!(resp.status(), 200);

        let result = handle.await.unwrap().unwrap();
        assert_eq!(result.code, "real-code");
    }
}
