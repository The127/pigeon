use std::time::Duration;

use async_trait::async_trait;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use tokio::time::Instant;

use pigeon_application::ports::delivery::{WebhookHttpClient, WebhookResult};

pub struct ReqwestWebhookClient {
    client: reqwest::Client,
}

impl ReqwestWebhookClient {
    pub fn new(timeout: Duration) -> Self {
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .expect("failed to build reqwest client");
        Self { client }
    }

    fn sign(payload: &[u8], secret: &str) -> String {
        let mut mac =
            Hmac::<Sha256>::new_from_slice(secret.as_bytes()).expect("HMAC accepts any key size");
        mac.update(payload);
        let result = mac.finalize();
        format!("sha256={}", hex::encode(result.into_bytes()))
    }
}

#[async_trait]
impl WebhookHttpClient for ReqwestWebhookClient {
    async fn deliver(
        &self,
        url: &str,
        payload: &serde_json::Value,
        signing_secret: &str,
    ) -> WebhookResult {
        let body = serde_json::to_vec(payload).expect("payload is valid JSON");
        let signature = Self::sign(&body, signing_secret);

        let start = Instant::now();

        let result = self
            .client
            .post(url)
            .header("Content-Type", "application/json")
            .header("X-Pigeon-Signature", signature)
            .body(body)
            .send()
            .await;

        let duration_ms = start.elapsed().as_millis() as i64;

        match result {
            Ok(response) => {
                let status_code = response.status().as_u16();
                let body = response
                    .text()
                    .await
                    .unwrap_or_else(|e| format!("failed to read body: {e}"));
                WebhookResult::Response {
                    status_code,
                    body,
                    duration_ms,
                }
            }
            Err(e) => WebhookResult::Error {
                message: e.to_string(),
                duration_ms,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_produces_deterministic_hmac() {
        let sig = ReqwestWebhookClient::sign(b"hello", "secret");
        // Known HMAC-SHA256("hello", "secret")
        assert!(sig.starts_with("sha256="));
        assert_eq!(sig.len(), 7 + 64); // "sha256=" + 64 hex chars

        let sig2 = ReqwestWebhookClient::sign(b"hello", "secret");
        assert_eq!(sig, sig2);
    }

    #[test]
    fn sign_differs_for_different_payloads() {
        let sig1 = ReqwestWebhookClient::sign(b"hello", "secret");
        let sig2 = ReqwestWebhookClient::sign(b"world", "secret");
        assert_ne!(sig1, sig2);
    }

    #[test]
    fn sign_differs_for_different_secrets() {
        let sig1 = ReqwestWebhookClient::sign(b"hello", "secret1");
        let sig2 = ReqwestWebhookClient::sign(b"hello", "secret2");
        assert_ne!(sig1, sig2);
    }
}
