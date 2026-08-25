use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
};
use tower::ServiceExt;

use super::*;

fn deserialize_api_env(
    additional_values: impl IntoIterator<Item = (&'static str, &'static str)>,
) -> Env {
    let mut values = vec![
        ("SUPABASE_URL", "http://127.0.0.1:54321"),
        ("SUPABASE_ANON_KEY", "anon-key"),
        ("SUPABASE_SERVICE_ROLE_KEY", "service-role-key"),
        ("OPENROUTER_API_KEY", "openrouter-key"),
        ("API_BASE_URL", "http://127.0.0.1:3001"),
        ("RESEND_API_KEY", "resend-key"),
        ("RESEND_FROM_EMAIL", "test@example.com"),
    ];
    values.extend(additional_values);
    envy::from_iter(
        values
            .into_iter()
            .map(|(key, value)| (key.to_string(), value.to_string())),
    )
    .unwrap()
}

fn api_env(with_hosted_integrations: bool) -> &'static crate::env::RuntimeConfig {
    let integration_values = with_hosted_integrations.then_some([
        ("NANGO_API_KEY", "nango-key"),
        ("NANGO_WEBHOOK_SIGNING_KEY", "nango-signing-key"),
        ("STRIPE_SECRET_KEY", "sk_test_hosted"),
        ("STRIPE_MONTHLY_PRICE_ID", "price_monthly"),
        ("STRIPE_YEARLY_PRICE_ID", "price_yearly"),
        ("LOOPS_KEY", "loops-key"),
        ("PYANNOTE_API_KEY", "pyannote-key"),
        ("EXA_API_KEY", "exa-key"),
        ("JINA_API_KEY", "jina-key"),
    ]);
    let env = deserialize_api_env(integration_values.into_iter().flatten());
    Box::leak(Box::new(
        crate::env::RuntimeConfig::resolve(env)
            .unwrap_or_else(|error| panic!("environment should validate: {error}")),
    ))
}

async fn request_status(app: &Router, method: Method, path: &str) -> StatusCode {
    app.clone()
        .oneshot(
            Request::builder()
                .method(method)
                .uri(path)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap()
        .status()
}

#[tokio::test]
async fn boots_without_private_sync_routes() {
    let app = app_with_env(api_env(false)).await;
    assert_eq!(
        request_status(&app, Method::GET, "/health").await,
        StatusCode::OK
    );
    for path in [
        "/sync/token",
        "/sync/replica/credentials",
        "/sync/e2ee/identity",
    ] {
        assert_eq!(
            request_status(&app, Method::POST, path).await,
            StatusCode::NOT_FOUND
        );
    }
}

#[tokio::test]
async fn hosted_integrations_remain_optional() {
    let app = app_with_env(api_env(true)).await;
    assert_ne!(
        request_status(&app, Method::POST, "/nango/webhook").await,
        StatusCode::NOT_FOUND
    );
}

#[test]
fn core_supabase_configuration_remains_required() {
    for missing in [
        "SUPABASE_URL",
        "SUPABASE_ANON_KEY",
        "SUPABASE_SERVICE_ROLE_KEY",
    ] {
        let values = [
            ("SUPABASE_URL", "http://127.0.0.1:54321"),
            ("SUPABASE_ANON_KEY", "anon-key"),
            ("SUPABASE_SERVICE_ROLE_KEY", "service-role-key"),
        ]
        .into_iter()
        .filter(|(key, _)| *key != missing)
        .map(|(key, value)| (key.to_string(), value.to_string()));
        assert!(envy::from_iter::<_, Env>(values).is_err());
    }
}
