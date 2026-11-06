use std::collections::HashMap;
use std::net::SocketAddr;
use axum::{
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use exaforge::modules::secrets::dto::SyncExaVaultRequest;
use exaforge::pkg::errors::AppError;
use exaforge::pkg::exavault::ExaVaultClient;
use serde_json::json;
use validator::Validate;

async fn mock_export_handler(headers: HeaderMap) -> impl IntoResponse {
    let auth = headers.get("authorization").and_then(|h| h.to_str().ok());
    if auth != Some("Bearer exv_valid_token_123") {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "Unauthorized" })),
        );
    }

    let mut secrets = HashMap::new();
    secrets.insert("DATABASE_URL".to_string(), "postgres://user:pass@host/db".to_string());
    secrets.insert("API_SECRET".to_string(), "supersecretvalue".to_string());
    secrets.insert("STRIPE_KEY".to_string(), "sk_test_123456789".to_string());

    (StatusCode::OK, Json(json!(secrets)))
}

#[tokio::test]
async fn test_exavault_client_validation() {
    let client = ExaVaultClient::default();
    let err = client.export_secrets_json(None, "   ").await.unwrap_err();
    match err {
        AppError::BadRequest(_, code) => assert_eq!(code, "INVALID_EXAVAULT_TOKEN"),
        other => panic!("Expected BadRequest error, got {:?}", other),
    }
}

#[tokio::test]
async fn test_exavault_client_export_success() {
    // Spin up local mock ExaVault server
    let app = Router::new().route("/api/v1/export", get(mock_export_handler));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr: SocketAddr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let client = ExaVaultClient::default();
    let mock_base_url = format!("http://{}", addr);

    let secrets = client
        .export_secrets_json(Some(&mock_base_url), "exv_valid_token_123")
        .await
        .expect("Should export secrets successfully");

    assert_eq!(secrets.len(), 3);
    assert_eq!(secrets.get("DATABASE_URL").unwrap(), "postgres://user:pass@host/db");
    assert_eq!(secrets.get("API_SECRET").unwrap(), "supersecretvalue");
    assert_eq!(secrets.get("STRIPE_KEY").unwrap(), "sk_test_123456789");
}

#[tokio::test]
async fn test_exavault_client_unauthorized() {
    let app = Router::new().route("/api/v1/export", get(mock_export_handler));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr: SocketAddr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let client = ExaVaultClient::default();
    let mock_base_url = format!("http://{}", addr);

    let err = client
        .export_secrets_json(Some(&mock_base_url), "exv_invalid_token")
        .await
        .unwrap_err();

    match err {
        AppError::Unauthorized(_, code) => assert_eq!(code, "EXAVAULT_UNAUTHORIZED"),
        other => panic!("Expected Unauthorized error, got {:?}", other),
    }
}

#[test]
fn test_sync_exavault_request_validation() {
    let invalid_req = SyncExaVaultRequest {
        scope_type: "project".to_string(),
        project_id: uuid::Uuid::new_v4(),
        service_id: None,
        exavault_url: Some("https://exavault.eka-dev.cloud".to_string()),
        exavault_token: "123".to_string(), // too short
    };
    assert!(invalid_req.validate().is_err());

    let valid_req = SyncExaVaultRequest {
        scope_type: "project".to_string(),
        project_id: uuid::Uuid::new_v4(),
        service_id: None,
        exavault_url: Some("https://exavault.eka-dev.cloud".to_string()),
        exavault_token: "exv_abc1234567890".to_string(),
    };
    assert!(valid_req.validate().is_ok());
}
