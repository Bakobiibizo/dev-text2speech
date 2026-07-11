FROM rust:1.83-bookworm AS builder
WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --locked --release

# Multi-architecture CPU image. Use docker/Dockerfile.cuda for NVIDIA hosts.
FROM python:3.11-slim-bookworm
RUN apt-get update && apt-get install -y --no-install-recommends ffmpeg libsndfile1 && rm -rf /var/lib/apt/lists/*
RUN useradd --create-home --uid 10001 app
WORKDIR /app
COPY --from=builder /build/target/release/dev-text2speech /usr/local/bin/dev-text2speech
COPY backend/requirements.txt /app/backend/requirements.txt
RUN pip install --no-cache-dir -r /app/backend/requirements.txt
COPY backend /app/backend
RUN mkdir -p /models /cache && chown -R app:app /app /models /cache
USER app
ENV API_HOST=0.0.0.0 API_PORT=7101 BACKEND_PORT=8101 BACKEND_URL=http://127.0.0.1:8101 \
    BACKEND_CMD=python3 BACKEND_ARGS="-m uvicorn api:app --host 127.0.0.1 --port 8101" \
    BACKEND_WORKDIR=/app/backend MANAGE_BACKEND=true HF_HOME=/cache/huggingface TORCH_HOME=/cache/torch
VOLUME ["/cache", "/models"]
EXPOSE 7101
HEALTHCHECK --interval=30s --timeout=3s --start-period=10s CMD python3 -c "import urllib.request; urllib.request.urlopen('http://127.0.0.1:7101/health')"
CMD ["dev-text2speech", "serve"]
