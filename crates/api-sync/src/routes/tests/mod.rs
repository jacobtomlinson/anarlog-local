use anlg_api_auth::{AuthContext, Claims};
use axum::{
    Extension,
    body::{Body, to_bytes},
    http::{HeaderValue, Request, StatusCode, header as http_header},
};
use serde_json::Value;
use tower::ServiceExt;
use wiremock::MockServer;

use super::*;
use crate::{SharedNotesConfig, SyncError};

fn test_router(server: &MockServer, api_key: &str, entitlements: &[&str]) -> Router {
    let state = AppState::new(
        SharedNotesConfig::new(server.uri(), "service-role-key")
            .unwrap()
            .with_resend_email(api_key, "sender@example.com")
            .unwrap(),
    );
    session_share_router(state.clone())
        .merge(web_edit_router(state))
        .layer(Extension(AuthContext {
            token: "supabase-token".to_string(),
            claims: Claims {
                sub: "user-123".to_string(),
                email: None,
                entitlements: entitlements
                    .iter()
                    .map(|entitlement| (*entitlement).to_string())
                    .collect(),
                subscription_status: None,
                trial_end: None,
                has_payment_method: None,
            },
        }))
}

async fn response_json(response: axum::response::Response) -> Value {
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

mod publication;
mod web_edits;
