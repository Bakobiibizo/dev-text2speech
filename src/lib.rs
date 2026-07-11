pub mod backend;
pub mod config;

use axum::{
    body::Body,
    extract::State,
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Semaphore;
use tower_http::{limit::RequestBodyLimitLayer, trace::TraceLayer};

#[derive(Clone)]
pub struct AppState {
    config: config::Config,
    client: reqwest::Client,
    permits: Arc<Semaphore>,
}

#[derive(Deserialize, Serialize)]
pub struct TtsRequest {
    pub text: String,
    #[serde(default)]
    pub voice: Option<String>,
}

#[derive(Serialize)]
struct Status {
    status: &'static str,
    backend: bool,
}

pub fn app(config: config::Config) -> Router {
    let client = reqwest::Client::builder()
        .timeout(config.request_timeout)
        .build()
        .expect("client");
    let permits = Arc::new(Semaphore::new(config.max_concurrent));
    Router::new()
        .route(
            "/health",
            get(|| async {
                Json(Status {
                    status: "ok",
                    backend: false,
                })
            }),
        )
        .route("/ready", get(ready))
        .route("/v1/audio/speech", post(synthesize))
        .route("/synthesize", post(synthesize))
        .layer(RequestBodyLimitLayer::new(64 * 1024))
        .layer(TraceLayer::new_for_http())
        .with_state(Arc::new(AppState {
            config,
            client,
            permits,
        }))
}

async fn ready(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let ok = state
        .client
        .get(format!("{}/ready", state.config.backend_url))
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false);
    let status = if ok {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    (
        status,
        Json(Status {
            status: if ok { "ready" } else { "unavailable" },
            backend: ok,
        }),
    )
}

async fn synthesize(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<TtsRequest>,
) -> Response {
    if let Some(expected) = &state.config.api_key {
        let supplied = headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "));
        if supplied != Some(expected.as_str()) {
            return error(StatusCode::UNAUTHORIZED, "unauthorized");
        }
    }
    let text = req.text.trim();
    if text.is_empty() || text.chars().count() > state.config.max_text_chars {
        return error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "text must be non-empty and within MAX_TEXT_CHARS",
        );
    }
    let _permit = match state.permits.try_acquire() {
        Ok(v) => v,
        Err(_) => {
            return error(
                StatusCode::TOO_MANY_REQUESTS,
                "synthesis capacity exhausted",
            )
        }
    };
    let upstream = match state
        .client
        .post(format!("{}/synthesize", state.config.backend_url))
        .json(&req)
        .send()
        .await
    {
        Ok(v) => v,
        Err(_) => return error(StatusCode::BAD_GATEWAY, "inference backend unavailable"),
    };
    let status = upstream.status();
    let content_type = upstream.headers().get(header::CONTENT_TYPE).cloned();
    let bytes = match upstream.bytes().await {
        Ok(v) => v,
        Err(_) => return error(StatusCode::BAD_GATEWAY, "invalid backend response"),
    };
    let mut response = Response::builder().status(status);
    if let Some(value) = content_type {
        response = response.header(header::CONTENT_TYPE, value);
    }
    response.body(Body::from(bytes)).expect("valid response")
}

fn error(status: StatusCode, message: &'static str) -> Response {
    (status, Json(serde_json::json!({"error": message}))).into_response()
}
