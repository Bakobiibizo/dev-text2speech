use axum::{
    body::Body,
    http::{Request, StatusCode},
    routing::{get, post},
    Router,
};
use dev_text2speech::{app, config::Config};
use http_body_util::BodyExt;
use std::time::Duration;
use tower::ServiceExt;

async fn config(backend_url: String, key: Option<&str>) -> Config {
    Config {
        api_host: "127.0.0.1".into(),
        api_port: 7101,
        backend_url,
        api_key: key.map(str::to_owned),
        max_text_chars: 10,
        max_concurrent: 1,
        request_timeout: Duration::from_secs(2),
        backend: dev_text2speech::backend::BackendConfig::from_env(8101),
    }
}

async fn backend() -> String {
    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/ready", get(|| async { "ready" }))
        .route(
            "/synthesize",
            post(|| async { ([("content-type", "audio/wav")], b"RIFFdemo") }),
        );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    format!("http://{addr}")
}

#[tokio::test]
async fn readiness_and_audio_passthrough() {
    let service = app(config(backend().await, None).await);
    let ready = service
        .clone()
        .oneshot(Request::get("/ready").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(ready.status(), StatusCode::OK);
    let response = service
        .oneshot(
            Request::post("/v1/audio/speech")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"text":"hello"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()["content-type"], "audio/wav");
    assert_eq!(
        response.into_body().collect().await.unwrap().to_bytes(),
        &b"RIFFdemo"[..]
    );
}

#[tokio::test]
async fn rejects_missing_auth_and_oversized_text() {
    let service = app(config(backend().await, Some("secret")).await);
    let request = |text: &str, auth: bool| {
        let mut r = Request::post("/synthesize").header("content-type", "application/json");
        if auth {
            r = r.header("authorization", "Bearer secret");
        }
        r.body(Body::from(format!(r#"{{"text":"{text}"}}"#)))
            .unwrap()
    };
    assert_eq!(
        service
            .clone()
            .oneshot(request("hello", false))
            .await
            .unwrap()
            .status(),
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        service
            .oneshot(request("elevenchars", true))
            .await
            .unwrap()
            .status(),
        StatusCode::UNPROCESSABLE_ENTITY
    );
}
