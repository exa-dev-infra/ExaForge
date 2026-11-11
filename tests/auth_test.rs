use exaforge::pkg::auth::{
    jwt::{generate_token_pair, verify_access_token},
    password::{hash_password, verify_password},
    token_hash::hash_token,
};
use uuid::Uuid;

#[test]
fn test_password_hashing_and_verification() {
    let password = "SuperSecretPassword2026!";
    let hashed = hash_password(password).expect("hashing should succeed");

    assert_ne!(password, hashed);
    assert!(verify_password(password, &hashed).expect("verification should succeed"));
    assert!(!verify_password("WrongPassword123", &hashed).expect("verification should succeed"));
}

#[test]
fn test_jwt_token_generation_and_validation() {
    let user_id = Uuid::new_v4();
    let email = "test@example.com";
    let role = "admin";
    let secret = "test-jwt-secret-key-12345678901234567890";

    let token_pair = generate_token_pair(&user_id, email, role, secret, 15)
        .expect("token generation should succeed");

    assert!(!token_pair.access_token.is_empty());
    assert!(!token_pair.refresh_token.is_empty());
    assert_eq!(token_pair.token_type, "Bearer");

    let claims = verify_access_token(&token_pair.access_token, secret)
        .expect("token verification should succeed");

    assert_eq!(claims.sub, user_id.to_string());
    assert_eq!(claims.email, email);
    assert_eq!(claims.role, role);

    // Corrupted secret should fail
    let invalid_claims = verify_access_token(&token_pair.access_token, "wrong-secret-key");
    assert!(invalid_claims.is_err());
}

#[test]
fn test_sha256_token_hashing() {
    let token = "raw-refresh-token-abc-123";
    let digest1 = hash_token(token);
    let digest2 = hash_token(token);

    assert_eq!(digest1, digest2);
    assert_eq!(digest1.len(), 64); // SHA-256 produces 64 hex characters
    assert_ne!(token, digest1);
}

#[tokio::test]
async fn test_authenticated_user_from_cookie_and_bearer() {
    use axum::extract::FromRequestParts;
    use exaforge::pkg::auth::jwt::AuthenticatedUser;

    let user_id = Uuid::new_v4();
    let email = "admin@exaforge.local";
    let role = "super_admin";
    let secret = "default_secret_key_change_in_production";
    std::env::set_var("JWT_SECRET", secret);

    let token_pair = generate_token_pair(&user_id, email, role, secret, 15).unwrap();

    // 1. Test Cookie header extraction
    let req = axum::http::Request::builder()
        .header(axum::http::header::COOKIE, format!("access_token={}; other=123", token_pair.access_token))
        .body(())
        .unwrap();

    let (mut parts, _) = req.into_parts();
    let auth_user = AuthenticatedUser::from_request_parts(&mut parts, &()).await.unwrap();

    assert_eq!(auth_user.id, user_id);
    assert_eq!(auth_user.email, email);
    assert_eq!(auth_user.role, role);

    // 2. Test Authorization Bearer header extraction
    let req2 = axum::http::Request::builder()
        .header(axum::http::header::AUTHORIZATION, format!("Bearer {}", token_pair.access_token))
        .body(())
        .unwrap();

    let (mut parts2, _) = req2.into_parts();
    let auth_user2 = AuthenticatedUser::from_request_parts(&mut parts2, &()).await.unwrap();

    assert_eq!(auth_user2.id, user_id);

    // 3. Test missing credentials returns 401 Unauthorized
    let req3 = axum::http::Request::builder()
        .body(())
        .unwrap();

    let (mut parts3, _) = req3.into_parts();
    let err = AuthenticatedUser::from_request_parts(&mut parts3, &()).await.unwrap_err();
    use axum::response::IntoResponse;
    assert_eq!(err.into_response().status(), axum::http::StatusCode::UNAUTHORIZED);
}
