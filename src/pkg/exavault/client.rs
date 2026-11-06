use std::collections::HashMap;
use std::time::Duration;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, USER_AGENT};
use tracing::{debug, error, info, warn};

use crate::pkg::errors::AppError;

pub const DEFAULT_EXAVAULT_URL: &str = "https://exavault.eka-dev.cloud";

#[derive(Debug, Clone)]
pub struct ExaVaultClient {
    client: reqwest::Client,
    default_base_url: String,
}

impl Default for ExaVaultClient {
    fn default() -> Self {
        Self::new(None)
    }
}

impl ExaVaultClient {
    pub fn new(default_url: Option<String>) -> Self {
        let base_url = default_url
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| {
                std::env::var("EXAVAULT_URL")
                    .unwrap_or_else(|_| DEFAULT_EXAVAULT_URL.to_string())
            });

        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("ExaForge-CICD/1.0"));

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap_or_default();

        Self {
            client,
            default_base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    /// Fetches decrypted secrets from ExaVault using the provided Service Token (`exv_...`).
    /// Calls `GET /api/v1/export?format=json` with `Authorization: Bearer <token>`.
    pub async fn export_secrets_json(
        &self,
        base_url_override: Option<&str>,
        token: &str,
    ) -> Result<HashMap<String, String>, AppError> {
        let clean_token = token.trim();
        if clean_token.is_empty() {
            return Err(AppError::bad_request("ExaVault token cannot be empty", "INVALID_EXAVAULT_TOKEN"));
        }

        let base_url = base_url_override
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.trim_end_matches('/'))
            .unwrap_or(&self.default_base_url);

        let endpoint = format!("{}/api/v1/export?format=json", base_url);

        debug!(endpoint = %endpoint, "Fetching secrets from ExaVault...");

        let res = self
            .client
            .get(&endpoint)
            .header(AUTHORIZATION, format!("Bearer {}", clean_token))
            .send()
            .await
            .map_err(|e| {
                error!(error = %e, endpoint = %endpoint, "Failed to connect to ExaVault server");
                AppError::internal(format!("ExaVault network connection failed: {}", e))
            })?;

        let status = res.status();
        if !status.is_success() {
            let error_text = res.text().await.unwrap_or_default();
            warn!(status = %status, response = %error_text, "ExaVault returned error status");
            if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
                return Err(AppError::unauthorized(
                    "ExaVault authorization failed: invalid, expired, or revoked service token",
                    "EXAVAULT_UNAUTHORIZED",
                ));
            }
            return Err(AppError::bad_request(
                format!("ExaVault export failed (HTTP {}): {}", status, error_text),
                "EXAVAULT_EXPORT_FAILED",
            ));
        }

        let secrets: HashMap<String, String> = res.json().await.map_err(|e| {
            AppError::internal(format!("Failed to parse JSON secrets from ExaVault: {}", e))
        })?;

        info!(count = secrets.len(), "Successfully fetched secrets from ExaVault");
        Ok(secrets)
    }
}

