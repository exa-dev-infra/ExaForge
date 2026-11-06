use axum::{
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, Request},
};
use exaforge::{
    modules::rbac::dto::{ProjectRole, ServiceRole, SystemRole},
    pkg::auth::jwt::{generate_token_pair, AdminUser, OptionalAuthUser, SuperAdminUser},
};
use uuid::Uuid;

#[tokio::test]
async fn test_admin_user_extractor() {
    let secret = "test-secret-key-12345678901234567890";
    std::env::set_var("JWT_SECRET", secret);

    let admin_id = Uuid::new_v4();
    let admin_tokens =
        generate_token_pair(&admin_id, "admin@example.com", "admin", secret, 15).unwrap();

    let super_admin_id = Uuid::new_v4();
    let super_admin_tokens =
        generate_token_pair(&super_admin_id, "super@example.com", "super_admin", secret, 15).unwrap();

    let user_id = Uuid::new_v4();
    let user_tokens =
        generate_token_pair(&user_id, "user@example.com", "user", secret, 15).unwrap();

    // 1. Admin token should succeed
    let req = Request::builder()
        .header(
            AUTHORIZATION,
            format!("Bearer {}", admin_tokens.access_token),
        )
        .body(())
        .unwrap();
    let (mut parts, _) = req.into_parts();
    let admin = AdminUser::from_request_parts(&mut parts, &()).await;
    assert!(admin.is_ok());
    assert_eq!(admin.unwrap().role, "admin");

    // 2. Super Admin token should also satisfy AdminUser
    let req_sa = Request::builder()
        .header(
            AUTHORIZATION,
            format!("Bearer {}", super_admin_tokens.access_token),
        )
        .body(())
        .unwrap();
    let (mut parts_sa, _) = req_sa.into_parts();
    let admin_from_sa = AdminUser::from_request_parts(&mut parts_sa, &()).await;
    assert!(admin_from_sa.is_ok());
    assert_eq!(admin_from_sa.unwrap().role, "super_admin");

    // 3. Regular user token should fail with forbidden
    let req_user = Request::builder()
        .header(
            AUTHORIZATION,
            format!("Bearer {}", user_tokens.access_token),
        )
        .body(())
        .unwrap();
    let (mut parts_user, _) = req_user.into_parts();
    let user_res = AdminUser::from_request_parts(&mut parts_user, &()).await;
    assert!(user_res.is_err());
}

#[tokio::test]
async fn test_super_admin_user_extractor() {
    let secret = "test-secret-key-12345678901234567890";
    std::env::set_var("JWT_SECRET", secret);

    let super_admin_id = Uuid::new_v4();
    let super_admin_tokens =
        generate_token_pair(&super_admin_id, "super@example.com", "super_admin", secret, 15).unwrap();

    let regular_admin_id = Uuid::new_v4();
    let regular_admin_tokens =
        generate_token_pair(&regular_admin_id, "admin@example.com", "admin", secret, 15).unwrap();

    // 1. Super Admin should pass SuperAdminUser
    let req_sa = Request::builder()
        .header(
            AUTHORIZATION,
            format!("Bearer {}", super_admin_tokens.access_token),
        )
        .body(())
        .unwrap();
    let (mut parts_sa, _) = req_sa.into_parts();
    let sa = SuperAdminUser::from_request_parts(&mut parts_sa, &()).await;
    assert!(sa.is_ok());
    assert_eq!(sa.unwrap().role, "super_admin");

    // 2. Regular admin should fail SuperAdminUser
    let req_admin = Request::builder()
        .header(
            AUTHORIZATION,
            format!("Bearer {}", regular_admin_tokens.access_token),
        )
        .body(())
        .unwrap();
    let (mut parts_admin, _) = req_admin.into_parts();
    let admin_res = SuperAdminUser::from_request_parts(&mut parts_admin, &()).await;
    assert!(admin_res.is_err());
}

#[tokio::test]
async fn test_rbac_role_enums_and_strings() {
    assert_eq!(SystemRole::SuperAdmin.to_string(), "super_admin");
    assert_eq!(SystemRole::Admin.to_string(), "admin");
    assert_eq!(SystemRole::Developer.to_string(), "developer");
    assert_eq!(SystemRole::Viewer.to_string(), "viewer");

    assert_eq!(ProjectRole::Maintainer.to_string(), "maintainer");
    assert_eq!(ProjectRole::Viewer.to_string(), "viewer");

    assert_eq!(ServiceRole::Developer.to_string(), "developer");
    assert_eq!(ServiceRole::Viewer.to_string(), "viewer");
}

#[tokio::test]
async fn test_optional_auth_user_extractor() {
    let secret = "test-secret-key-12345678901234567890";
    std::env::set_var("JWT_SECRET", secret);

    let user_id = Uuid::new_v4();
    let tokens = generate_token_pair(&user_id, "opt@example.com", "user", secret, 15).unwrap();

    // 1. Present valid token
    let req = Request::builder()
        .header(AUTHORIZATION, format!("Bearer {}", tokens.access_token))
        .body(())
        .unwrap();
    let (mut parts, _) = req.into_parts();
    let opt = OptionalAuthUser::from_request_parts(&mut parts, &())
        .await
        .unwrap();
    assert!(opt.is_some());
    assert_eq!(opt.0.unwrap().email, "opt@example.com");

    // 2. Absent token -> returns None without error
    let req_absent = Request::builder().body(()).unwrap();
    let (mut parts_absent, _) = req_absent.into_parts();
    let opt_absent = OptionalAuthUser::from_request_parts(&mut parts_absent, &())
        .await
        .unwrap();
    assert!(opt_absent.is_none());
}
