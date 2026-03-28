use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use serde::Serialize;
use utoipa::ToSchema;

use crate::state::AppState;

#[derive(Serialize, ToSchema)]
pub struct AuthConfigResponse {
    pub issuer_url: String,
    pub audience: String,
}

/// Returns the OIDC configuration for the current tenant.
///
/// Tenant resolution:
/// - Subdomain-based: `acme.example.com` → org slug `acme`
/// - Single-tenant mode: bare domain / localhost → bootstrap org
#[utoipa::path(
    get,
    path = "/api/v1/auth/config",
    responses(
        (status = 200, description = "Auth configuration", body = AuthConfigResponse),
        (status = 404, description = "Organization not found or no OIDC config"),
    ),
    tag = "auth"
)]
pub async fn auth_config(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    let host = headers
        .get("host")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("localhost");

    let org_slug = extract_subdomain(host);

    let org = if let Some(slug) = org_slug {
        // Subdomain mode: look up org by slug
        state
            .org_read_store
            .find_by_slug(slug)
            .await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to look up organization"))?
    } else {
        // Single-tenant mode: use bootstrap org
        if let Some(admin_org_id) = &state.admin_org_id {
            state
                .org_read_store
                .find_by_id(admin_org_id)
                .await
                .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to look up organization"))?
        } else {
            None
        }
    };

    let org = org.ok_or((StatusCode::NOT_FOUND, "Organization not found"))?;

    let configs = state
        .oidc_config_read_store
        .list_by_org(org.id(), 0, 1)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to look up OIDC config"))?;

    let config = configs
        .first()
        .ok_or((StatusCode::NOT_FOUND, "No OIDC config for this organization"))?;

    Ok(Json(AuthConfigResponse {
        issuer_url: config.issuer_url().to_string(),
        audience: config.audience().to_string(),
    }))
}

/// Extract subdomain from host header.
/// `acme.pigeon.dev` → Some("acme")
/// `pigeon.dev` → None
/// `localhost:3000` → None
fn extract_subdomain(host: &str) -> Option<&str> {
    // Strip port
    let host = host.split(':').next().unwrap_or(host);

    // localhost or IP — no subdomain
    if host == "localhost" || host.parse::<std::net::IpAddr>().is_ok() {
        return None;
    }

    let parts: Vec<&str> = host.split('.').collect();
    if parts.len() > 2 {
        Some(parts[0])
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_subdomain_from_three_part_host() {
        assert_eq!(extract_subdomain("acme.pigeon.dev"), Some("acme"));
    }

    #[test]
    fn no_subdomain_for_bare_domain() {
        assert_eq!(extract_subdomain("pigeon.dev"), None);
    }

    #[test]
    fn no_subdomain_for_localhost() {
        assert_eq!(extract_subdomain("localhost"), None);
        assert_eq!(extract_subdomain("localhost:3000"), None);
    }

    #[test]
    fn no_subdomain_for_ip() {
        assert_eq!(extract_subdomain("127.0.0.1"), None);
        assert_eq!(extract_subdomain("127.0.0.1:3000"), None);
    }

    #[test]
    fn extracts_subdomain_with_port() {
        assert_eq!(extract_subdomain("acme.pigeon.dev:3000"), Some("acme"));
    }
}
