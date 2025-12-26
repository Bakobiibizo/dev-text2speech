# Text2Speech Proxy + Backend
# Bundles the Rust proxy with the WhisperSpeech Python backend

FROM rust:1.83-bookworm AS builder

WORKDIR /build
COPY Cargo.toml ./
COPY src ./src
RUN cargo build --release

# Runtime image with CUDA support
FROM nvidia/cuda:12.1.0-runtime-ubuntu22.04

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    python3 \
    python3-pip \
    ffmpeg \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy Rust binary
COPY --from=builder /build/target/release/dev-text2speech /app/proxy

# Copy backend code
COPY backend/ /app/backend/

# Install Python dependencies
RUN python3 -m pip install --no-cache-dir \
    fastapi uvicorn pydantic torch torchaudio whisperspeech webdataset

# Environment defaults
# Proxy listens on 7101, backend on internal port 8101
ENV API_HOST=0.0.0.0
ENV API_PORT=7101
ENV BACKEND_URL=http://localhost:8101
ENV BACKEND_CMD=python3
ENV BACKEND_ARGS="-m uvicorn api:app --host 0.0.0.0 --port 8101"
ENV BACKEND_WORKDIR=/app/backend
ENV BACKEND_PORT=8101
ENV BACKEND_HEALTH_PATH=/health
ENV PRELOAD=true

EXPOSE 7101

CMD ["/app/proxy"]
