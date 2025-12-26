# Directory Structure

- 📁 **./**
  - 📄 **Cargo.toml**

    📄 *File Path*: `./Cargo.toml`
    *Size*: 545 bytes | *Modified*: 2025-12-22 13:52:34

    ```
    [package]
    name = "dev-text2speech"
    version = "0.1.0"
    edition = "2021"
    
    [dependencies]
    tokio = { version = "1.43", features = ["full"] }
    axum = "0.7"
    serde = { version = "1.0", features = ["derive"] }
    serde_json = "1.0"
    tracing = "0.1"
    tracing-subscriber = { version = "0.3", features = ["env-filter"] }
    dotenv = "0.15"
    reqwest = { version = "0.12", features = ["json"] }
    tower-http = { version = "0.5", features = ["cors"] }
    tower = { version = "0.4", features = ["util"] }
    thiserror = "1.0"
    
    [bin]
    name = "dev-text2speech"
    path = "src/main.rs"
    ```

  - 📁 **docker/**
    - 📄 **Dockerfile.core**

      📄 *File Path*: `./docker/Dockerfile.core`
      *Size*: 1269 bytes | *Modified*: 2025-12-22 13:11:35

      ```
      FROM nvcr.io/nvidia/pytorch:25.09-py3
      
      ARG UID=1000
      ARG GID=1000
      
      ENV DEBIAN_FRONTEND=noninteractive
      RUN apt-get update && apt-get install -y --no-install-recommends \
       git git-lfs curl ca-certificates \
       build-essential pkg-config \
       cmake ninja-build \
       python3.12-venv python3.12-dev \
       && rm -rf /var/lib/apt/lists/*
      
      RUN pip install --no-cache-dir uv
      
      RUN pip uninstall -y pynvml || true \
      && pip install --no-cache-dir nvidia-ml-py
      
      RUN pip install --no-cache-dir --no-deps --upgrade setuptools wheel \
      && pip install --no-cache-dir --no-deps --no-build-isolation "git+https://github.com/pytorch/audio@release/2.9" \
      && python -c "import torchvision; print(torchvision.__version__)" || pip install --no-cache-dir --no-deps --no-build-isolation "git+https://github.com/pytorch/vision@release/2.9" \
      && python -c "import torch; print(torch.__version__, torch.version.cuda)" \
      && python -c "import torchaudio; print(torchaudio.__version__)" \
      && python -c "import torchvision; print(torchvision.__version__)"
      
      RUN groupadd -g ${GID} dev || true \
      && useradd -m -u ${UID} -g ${GID} -s /bin/bash dev || true \
      && mkdir -p /home/dev/.cache/huggingface /home/dev/.cache/torch /home/dev/.cache/uv /models \
      && chown -R ${UID}:${GID} /home/dev /models
      
      WORKDIR /workspace
      ```

  - 📄 **docker-compose.yml**

    📄 *File Path*: `./docker-compose.yml`
    *Size*: 1268 bytes | *Modified*: 2025-12-22 13:52:21

    ```
    services:
      dev-text2speech:
        build:
          context: .
          dockerfile: docker/Dockerfile.core
          args:
            UID: ${UID:-1000}
            GID: ${GID:-1000}
        image: ${CORE_IMAGE:-devkit-core:local}
        working_dir: /workspace
        ipc: host
        ulimits:
          memlock: -1
          stack: 67108864
        stdin_open: true
        tty: true
        volumes:
          - models:/models
          - hf-cache:/home/dev/.cache/huggingface
          - torch-cache:/home/dev/.cache/torch
          - uv-cache:/home/dev/.cache/uv
          - .:/workspace:cached
        ports:
          - "${API_PORT:-7097}:${API_PORT:-7097}"
        environment:
          - NVIDIA_VISIBLE_DEVICES=all
          - NVIDIA_DRIVER_CAPABILITIES=compute,utility
          - HOME=/home/dev
          - HF_HOME=/home/dev/.cache/huggingface
          - TRANSFORMERS_CACHE=/home/dev/.cache/huggingface
          - TORCH_HOME=/home/dev/.cache/torch
          - API_HOST=0.0.0.0
          - API_PORT=${API_PORT:-7097}
          - BACKEND_URL=${BACKEND_URL:-http://localhost:7097}
          - PRELOAD=${PRELOAD:-true}
        deploy:
          resources:
            reservations:
              devices:
                - capabilities: [gpu]
        user: "${UID:-1000}:${GID:-1000}"
        command: ["bash", "-lc", "cargo run --bin dev-text2speech"]
    
    volumes:
      models:
      hf-cache:
      torch-cache:
      uv-cache:
    ```

  - 📄 **launch.sh**

    📄 *File Path*: `./launch.sh`
    *Size*: 203 bytes | *Modified*: 2025-12-22 13:53:02

    ```
    #!/usr/bin/env bash
    set -euo pipefail
    
    ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    
    echo "[launch] starting dev-text2speech container..."
    cd "$ROOT_DIR"
    docker compose up -d dev-text2speech
    ```

  - 📁 **scripts/**
    - 📄 **add-missing-dep.sh**

      📄 *File Path*: `./scripts/add-missing-dep.sh`
      *Size*: 391 bytes | *Modified*: 2025-12-22 19:51:00

      ```
      #!/usr/bin/env bash
      set -euo pipefail
      
      ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
      MISSING_REQ="$ROOT_DIR/scripts/missing-deps.txt"
      
      if [[ $# -lt 1 ]]; then
        echo "Usage: $0 <package> [<package> ...]" >&2
        exit 2
      fi
      
      for pkg in "$@"; do
        echo "$pkg" >> "$MISSING_REQ"
        echo "[missing-deps] recorded: $pkg"
      done
      
      echo "[missing-deps] current list:"
      sort -u "$MISSING_REQ"
      ```

    - 📄 **setup.sh**

      📄 *File Path*: `./scripts/setup.sh`
      *Size*: 1276 bytes | *Modified*: 2025-12-22 19:50:54

      ```
      #!/usr/bin/env bash
      set -euo pipefail
      
      ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
      
      NO_CACHE=0
      for arg in "$@"; do
        case "$arg" in
          --no-cache) NO_CACHE=1 ;;
          -h|--help)
            echo "Usage: $0 [--no-cache]" >&2
            exit 0
            ;;
          *)
            echo "[setup] error: unknown argument: $arg" >&2
            exit 2
            ;;
        esac
      done
      
      log() { echo "[setup] $*"; }
      die() { echo "[setup] error: $*" >&2; exit 1; }
      
      command -v docker >/dev/null 2>&1 || die "docker is required"
      docker compose version >/dev/null 2>&1 || die "docker compose plugin is required (docker compose ...)"
      
      log "Building container (dev-text2speech)..."
      (
        cd "$ROOT_DIR"
        if [[ "$NO_CACHE" == "1" ]]; then
          docker compose build --no-cache dev-text2speech
        else
          docker compose build dev-text2speech
        fi
      )
      
      log "Starting container (dev-text2speech)..."
      (
        cd "$ROOT_DIR"
        docker compose up -d dev-text2speech
      )
      
      # Install remembered missing deps (persisted from add-missing-dep.sh)
      MISSING_REQ="$ROOT_DIR/scripts/missing-deps.txt"
      if [[ -f "$MISSING_REQ" ]]; then
        log "Installing remembered missing deps from scripts/missing-deps.txt..."
        (
          cd "$ROOT_DIR"
          docker compose run --rm dev-text2speech bash -lc "pip install -r /workspace/scripts/missing-deps.txt"
        )
      fi
      ```

  - 📁 **src/**
    - 📄 **config.rs**

      📄 *File Path*: `./src/config.rs`
      *Size*: 787 bytes | *Modified*: 2025-12-22 13:52:40

      ```
      use std::env;
      
      pub struct Config {
          pub api_host: String,
          pub api_port: u16,
          pub backend_url: String,
          pub preload: bool,
      }
      
      impl Config {
          pub fn load() -> Self {
              let api_host = env::var("API_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
              let api_port = env::var("API_PORT")
                  .ok()
                  .and_then(|v| v.parse().ok())
                  .unwrap_or(7097);
              let backend_url = env::var("BACKEND_URL").unwrap_or_else(|_| "http://localhost:7097".to_string());
              let preload = env::var("PRELOAD")
                  .ok()
                  .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                  .unwrap_or(true);
              Self {
                  api_host,
                  api_port,
                  backend_url,
                  preload,
              }
          }
      }
      ```

    - 📄 **main.rs**

      📄 *File Path*: `./src/main.rs`
      *Size*: 3847 bytes | *Modified*: 2025-12-22 13:52:49

      ```
      use std::net::SocketAddr;
      use std::sync::Arc;
      
      use axum::{
          extract::State,
          http::StatusCode,
          response::IntoResponse,
          routing::{get, post},
          Json, Router,
      };
      use reqwest::Client;
      use serde::{Deserialize, Serialize};
      use tokio::sync::RwLock;
      use tower_http::cors::CorsLayer;
      use tracing::{error, info};
      
      mod config;
      
      #[derive(Clone)]
      struct AppState {
          config: config::Config,
          client: Client,
          ready: Arc<RwLock<bool>>,
      }
      
      #[derive(Deserialize)]
      struct TtsRequest {
          text: String,
          #[serde(default)]
          voice: Option<String>,
      }
      
      #[derive(Serialize)]
      struct ReadyStatus {
          ready: bool,
      }
      
      async fn health() -> &'static str {
          "ok"
      }
      
      async fn ready(State(state): State<Arc<AppState>>) -> impl IntoResponse {
          let ready = *state.ready.read().await;
          (StatusCode::OK, Json(ReadyStatus { ready }))
      }
      
      async fn warm(State(state): State<Arc<AppState>>) -> impl IntoResponse {
          match call_backend_health(&state).await {
              Ok(_) => {
                  *state.ready.write().await = true;
                  (StatusCode::OK, "warmed")
              }
              Err(err) => {
                  error!("warm failed: {}", err);
                  (StatusCode::BAD_GATEWAY, "warm failed")
              }
          }
      }
      
      async fn synthesize(
          State(state): State<Arc<AppState>>,
          Json(req): Json<TtsRequest>,
      ) -> impl IntoResponse {
          let url = format!("{}/synthesize", state.config.backend_url);
          let payload = serde_json::json!({
              "text": req.text,
              "voice": req.voice,
          });
          match state.client.post(&url).json(&payload).send().await {
              Ok(resp) => {
                  let status = resp.status();
                  match resp.bytes().await {
                      Ok(bytes) => (status, bytes),
                      Err(err) => {
                          error!("backend read error: {}", err);
                          (StatusCode::BAD_GATEWAY, "backend read error".into())
                      }
                  }
              }
              Err(err) => {
                  error!("backend error: {}", err);
                  (StatusCode::BAD_GATEWAY, "backend error".into())
              }
          }
      }
      
      #[tokio::main]
      async fn main() {
          tracing_subscriber::fmt::init();
          dotenv::dotenv().ok();
      
          let cfg = config::Config::load();
          let state = Arc::new(AppState {
              config: cfg,
              client: Client::new(),
              ready: Arc::new(RwLock::new(false)),
          });
      
          // Warm backend on startup
          let preload_state = state.clone();
          tokio::spawn(async move {
              if preload_state.config.preload {
                  if call_backend_health(&preload_state).await.is_ok() {
                      *preload_state.ready.write().await = true;
                      info!("backend warmed");
                  }
              } else {
                  *preload_state.ready.write().await = true;
              }
          });
      
          let app = Router::new()
              .route("/health", get(health))
              .route("/ready", get(ready))
              .route("/warm", post(warm))
              .route("/synthesize", post(synthesize))
              .layer(CorsLayer::permissive())
              .with_state(state.clone());
      
          let addr: SocketAddr = format!("{}:{}", state.config.api_host, state.config.api_port)
              .parse()
              .unwrap_or_else(|e| {
                  error!("Invalid bind address: {}", e);
                  SocketAddr::from(([0, 0, 0, 0], 7097))
              });
          info!("listening on {}, proxying to {}", addr, state.config.backend_url);
          axum::Server::bind(&addr)
              .serve(app.into_make_service())
              .await
              .unwrap();
      }
      
      async fn call_backend_health(state: &AppState) -> Result<(), reqwest::Error> {
          let url = format!("{}/health", state.config.backend_url);
          let resp = state.client.get(url).send().await?;
          if resp.status().is_success() {
              Ok(())
          } else {
              Err(reqwest::Error::new(
                  reqwest::StatusCode::BAD_GATEWAY,
                  "backend unhealthy",
              ))
          }
      }
      ```

