use serde_json::{Value, json};

use crate::common::TestApp;

#[tokio::test]
async fn register_success() {
    let app = TestApp::spawn().await;
    let res = app
        .client
        .post(app.url("/auth/register"))
        .json(&json!({ "email": "test@example.com", "password": "password123" }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 201);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["user"]["email"], "test@example.com");
    assert!(body["access_token"].is_string());
    assert!(body["refresh_token"].is_string());
}

#[tokio::test]
async fn register_duplicate_email() {
    let app = TestApp::spawn().await;
    let payload = json!({ "email": "dup@example.com", "password": "password123" });

    app.client
        .post(app.url("/auth/register"))
        .json(&payload)
        .send()
        .await
        .unwrap();

    let res = app
        .client
        .post(app.url("/auth/register"))
        .json(&payload)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 409);
}

#[tokio::test]
async fn register_invalid_email() {
    let app = TestApp::spawn().await;
    let res = app
        .client
        .post(app.url("/auth/register"))
        .json(&json!({ "email": "not-an-email", "password": "password123" }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 422);
}

#[tokio::test]
async fn register_short_password() {
    let app = TestApp::spawn().await;
    let res = app
        .client
        .post(app.url("/auth/register"))
        .json(&json!({ "email": "test@example.com", "password": "short" }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 422);
}

#[tokio::test]
async fn login_success() {
    let app = TestApp::spawn().await;
    app.client
        .post(app.url("/auth/register"))
        .json(&json!({ "email": "login@example.com", "password": "password123" }))
        .send()
        .await
        .unwrap();

    let res = app
        .client
        .post(app.url("/auth/login"))
        .json(&json!({ "email": "login@example.com", "password": "password123" }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    assert!(body["access_token"].is_string());
    assert!(body["refresh_token"].is_string());
}

#[tokio::test]
async fn login_wrong_password() {
    let app = TestApp::spawn().await;
    app.client
        .post(app.url("/auth/register"))
        .json(&json!({ "email": "wrong@example.com", "password": "password123" }))
        .send()
        .await
        .unwrap();

    let res = app
        .client
        .post(app.url("/auth/login"))
        .json(&json!({ "email": "wrong@example.com", "password": "wrongpassword" }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 401);
}

#[tokio::test]
async fn login_nonexistent_user() {
    let app = TestApp::spawn().await;
    let res = app
        .client
        .post(app.url("/auth/login"))
        .json(&json!({ "email": "nobody@example.com", "password": "password123" }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 401);
}

#[tokio::test]
async fn refresh_token_success() {
    let app = TestApp::spawn().await;
    let res = app
        .client
        .post(app.url("/auth/register"))
        .json(&json!({ "email": "refresh@example.com", "password": "password123" }))
        .send()
        .await
        .unwrap();

    let body: Value = res.json().await.unwrap();
    let refresh_token = body["refresh_token"].as_str().unwrap();

    let res = app
        .client
        .post(app.url("/auth/refresh"))
        .json(&json!({ "refresh_token": refresh_token }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    assert!(body["access_token"].is_string());
    assert!(body["refresh_token"].is_string());
}

#[tokio::test]
async fn refresh_token_reuse_rejected() {
    let app = TestApp::spawn().await;
    let res = app
        .client
        .post(app.url("/auth/register"))
        .json(&json!({ "email": "reuse@example.com", "password": "password123" }))
        .send()
        .await
        .unwrap();

    let body: Value = res.json().await.unwrap();
    let refresh_token = body["refresh_token"].as_str().unwrap();

    // First refresh should succeed
    app.client
        .post(app.url("/auth/refresh"))
        .json(&json!({ "refresh_token": refresh_token }))
        .send()
        .await
        .unwrap();

    // Second use of the same token should fail (revoked by rotation)
    let res = app
        .client
        .post(app.url("/auth/refresh"))
        .json(&json!({ "refresh_token": refresh_token }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 401);
}

#[tokio::test]
async fn forgot_password_returns_success() {
    let app = TestApp::spawn().await;
    // Should return 200 even for non-existent email
    let res = app
        .client
        .post(app.url("/auth/forgot-password"))
        .json(&json!({ "email": "noone@example.com" }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
}

#[tokio::test]
async fn logout_revokes_refresh() {
    let app = TestApp::spawn().await;
    let res = app
        .client
        .post(app.url("/auth/register"))
        .json(&json!({ "email": "logout@example.com", "password": "password123" }))
        .send()
        .await
        .unwrap();

    let body: Value = res.json().await.unwrap();
    let access_token = body["access_token"].as_str().unwrap();
    let refresh_token = body["refresh_token"].as_str().unwrap();

    // Logout
    let res = app
        .client
        .post(app.url("/auth/logout"))
        .header("Authorization", format!("Bearer {}", access_token))
        .json(&json!({ "refresh_token": refresh_token }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);

    // Refresh should now fail
    let res = app
        .client
        .post(app.url("/auth/refresh"))
        .json(&json!({ "refresh_token": refresh_token }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 401);
}
