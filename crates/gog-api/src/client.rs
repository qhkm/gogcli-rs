// gog-api client module
// Authenticated client builder.
// Ported from internal/googleapi/client.go

use crate::error::ApiError;
use gog_auth::Service;
use gog_core::config;

const TOKEN_ENDPOINT: &str = "https://oauth2.googleapis.com/token";

/// An authenticated reqwest client for a Google service.
pub struct AuthenticatedClient {
    pub client: reqwest::Client,
    pub access_token: String,
    pub email: String,
}

impl AuthenticatedClient {
    /// Create an authenticated client for the given service and account.
    ///
    /// Steps:
    /// 1. Read client credentials from config.
    /// 2. Get refresh token from keyring.
    /// 3. Exchange refresh token for access token via OAuth2 token endpoint.
    /// 4. Return client with auth header.
    pub async fn new(service: Service, email: &str, oauth_client: &str) -> Result<Self, ApiError> {
        // 1. Read client credentials from config
        let creds = config::read_client_credentials_for(oauth_client)?;

        // 2. Get refresh token from store
        let store = gog_secrets::open_store()?;
        let token = store.get_token(oauth_client, email)?;
        let refresh_token = token.refresh_token;

        // 3. Exchange refresh token for access token
        let scopes = service.scopes().join(" ");
        let http_client = reqwest::Client::new();
        let params = [
            ("client_id", creds.client_id.as_str()),
            ("client_secret", creds.client_secret.as_str()),
            ("refresh_token", refresh_token.as_str()),
            ("grant_type", "refresh_token"),
            ("scope", scopes.as_str()),
        ];

        let resp = http_client
            .post(TOKEN_ENDPOINT)
            .form(&params)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let msg = resp.text().await.unwrap_or_default();
            return Err(ApiError::GoogleApi {
                status,
                message: msg,
            });
        }

        let token_resp: serde_json::Value = resp.json().await?;
        let access_token = token_resp["access_token"]
            .as_str()
            .ok_or_else(|| ApiError::GoogleApi {
                status: 0,
                message: "missing access_token in response".to_string(),
            })?
            .to_string();

        // 4. Build client with Authorization header
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", access_token)
                .parse()
                .expect("valid header value"),
        );

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;

        Ok(AuthenticatedClient {
            client,
            access_token,
            email: email.to_string(),
        })
    }
}
