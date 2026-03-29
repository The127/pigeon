use std::time::Duration;

use hmac::{Hmac, Mac};
use mockito::Server;
use pigeon_application::ports::delivery::{WebhookHttpClient, WebhookResult};
use pigeon_infrastructure::http::ReqwestWebhookClient;
use serde_json::json;
use sha2::Sha256;

#[tokio::test]
async fn delivers_payload_with_correct_signature() {
    let mut server = Server::new_async().await;
    let secret = "whsec_test_secret";
    let payload = json!({"event": "user.created", "data": {"id": 1}});

    let mock = server
        .mock("POST", "/webhook")
        .match_header("Content-Type", "application/json")
        .match_header("X-Pigeon-Signature", mockito::Matcher::Any)
        .with_status(200)
        .with_body("OK")
        .create_async()
        .await;

    let client = ReqwestWebhookClient::new(Duration::from_secs(5));
    let url = format!("{}/webhook", server.url());

    let result = client.deliver(&url, &payload, Some(secret)).await;

    mock.assert_async().await;

    match result {
        WebhookResult::Response {
            status_code,
            body,
            duration_ms,
        } => {
            assert_eq!(status_code, 200);
            assert_eq!(body, "OK");
            assert!(duration_ms >= 0);
        }
        WebhookResult::Error { message, .. } => {
            panic!("expected success, got error: {message}");
        }
    }
}

#[tokio::test]
async fn signature_is_valid_hmac_sha256() {
    let mut server = Server::new_async().await;
    let secret = "whsec_verify_me";
    let payload = json!({"test": true});
    let payload_bytes = serde_json::to_vec(&payload).unwrap();

    let expected_signature = {
        let mut mac =
            Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(&payload_bytes);
        format!("sha256={}", hex::encode(mac.finalize().into_bytes()))
    };

    let mock = server
        .mock("POST", "/hook")
        .match_header("X-Pigeon-Signature", expected_signature.as_str())
        .with_status(200)
        .with_body("verified")
        .create_async()
        .await;

    let client = ReqwestWebhookClient::new(Duration::from_secs(5));
    let url = format!("{}/hook", server.url());
    client.deliver(&url, &payload, Some(secret)).await;

    // If the signature doesn't match, mockito won't match the mock and the test fails
    mock.assert_async().await;
}

#[tokio::test]
async fn returns_error_response_codes() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("POST", "/failing")
        .with_status(500)
        .with_body("Internal Server Error")
        .create_async()
        .await;

    let client = ReqwestWebhookClient::new(Duration::from_secs(5));
    let url = format!("{}/failing", server.url());

    let result = client
        .deliver(&url, &json!({"x": 1}), Some("secret"))
        .await;

    mock.assert_async().await;

    match result {
        WebhookResult::Response {
            status_code, body, ..
        } => {
            assert_eq!(status_code, 500);
            assert_eq!(body, "Internal Server Error");
        }
        WebhookResult::Error { message, .. } => {
            panic!("expected response, got error: {message}");
        }
    }
}

#[tokio::test]
async fn returns_error_on_connection_refused() {
    let client = ReqwestWebhookClient::new(Duration::from_secs(1));

    let result = client
        .deliver(
            "http://127.0.0.1:1",
            &json!({"x": 1}),
            Some("secret"),
        )
        .await;

    match result {
        WebhookResult::Error { duration_ms, .. } => {
            assert!(duration_ms >= 0);
        }
        WebhookResult::Response { .. } => {
            panic!("expected error for connection refused");
        }
    }
}
