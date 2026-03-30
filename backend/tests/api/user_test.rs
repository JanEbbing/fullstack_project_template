use serde_json::{Value, json};

use crate::common::TestApp;

async fn register_and_get_token(app: &TestApp, email: &str) -> String {
    let res = app
        .client
        .post(app.url("/auth/register"))
        .json(&json!({ "email": email, "password": "password123" }))
        .send()
        .await
        .unwrap();
    let body: Value = res.json().await.unwrap();
    body["access_token"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn get_me_authenticated() {
    let app = TestApp::spawn().await;
    let token = register_and_get_token(&app, "me@example.com").await;

    let res = app
        .client
        .get(app.url("/user/me"))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["user"]["email"], "me@example.com");
}

#[tokio::test]
async fn get_me_unauthenticated() {
    let app = TestApp::spawn().await;
    let res = app.client.get(app.url("/user/me")).send().await.unwrap();
    assert_eq!(res.status(), 401);
}

#[tokio::test]
async fn create_and_list_user_data() {
    let app = TestApp::spawn().await;
    let token = register_and_get_token(&app, "data@example.com").await;

    // Create an item
    let res = app
        .client
        .post(app.url("/user/data"))
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({ "title": "Test Item", "content": "Some content" }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 201);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["data"]["title"], "Test Item");

    // List items
    let res = app
        .client
        .get(app.url("/user/data"))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    let data = body["data"].as_array().unwrap();
    assert_eq!(data.len(), 1);
    assert_eq!(data[0]["title"], "Test Item");
}

#[tokio::test]
async fn user_data_isolation() {
    let app = TestApp::spawn().await;
    let token_a = register_and_get_token(&app, "alice@example.com").await;
    let token_b = register_and_get_token(&app, "bob@example.com").await;

    // Alice creates data
    app.client
        .post(app.url("/user/data"))
        .header("Authorization", format!("Bearer {}", token_a))
        .json(&json!({ "title": "Alice's Item", "content": "Private" }))
        .send()
        .await
        .unwrap();

    // Bob should see empty list
    let res = app
        .client
        .get(app.url("/user/data"))
        .header("Authorization", format!("Bearer {}", token_b))
        .send()
        .await
        .unwrap();

    let body: Value = res.json().await.unwrap();
    let data = body["data"].as_array().unwrap();
    assert_eq!(data.len(), 0);
}
