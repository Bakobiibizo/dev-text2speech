# dev-text2speech

A bounded HTTP service and command-line client for the WhisperSpeech text-to-speech pipeline. The Rust edge service handles authentication, limits, readiness, and WAV passthrough; the Python process owns model inference.

## Run

CPU images build on both ARM64 and x86_64:

```bash
API_KEY="$(openssl rand -hex 24)" docker compose up --build text2speech
```

For NVIDIA CUDA:

```bash
API_KEY="$(openssl rand -hex 24)" docker compose --profile nvidia up --build text2speech-nvidia
```

The initial synthesis downloads the configured model into the persistent `tts-cache` volume. To reuse an existing cache, set `MODEL_CACHE_PATH=/absolute/cache/path`. Set `TTS_MODEL` to select another compatible WhisperSpeech model.

```bash
curl --fail http://localhost:7101/health
curl --fail -H "Authorization: Bearer $API_KEY" \
  -H 'Content-Type: application/json' \
  --data '{"text":"Hello from WhisperSpeech"}' \
  http://localhost:7101/v1/audio/speech > speech.wav
```

`/health` reports process liveness. `/ready` returns 200 only when the backend is reachable. The backend's `/ready` additionally distinguishes a loaded model.

## CLI and local development

Run the backend and edge service separately:

```bash
cd backend
python -m uvicorn api:app --host 127.0.0.1 --port 8101
# another shell
BACKEND_URL=http://127.0.0.1:8101 cargo run -- serve
cargo run -- synthesize "A short test" --output speech.wav
```

Important configuration:

| Variable | Default | Purpose |
|---|---:|---|
| `API_HOST` | `127.0.0.1` | Safe bind address (container overrides to `0.0.0.0`) |
| `API_PORT` | `7101` | Public service port |
| `API_KEY` | unset | Optional bearer token; required when configured |
| `MAX_TEXT_CHARS` | `5000` | Unicode character limit |
| `MAX_CONCURRENT_SYNTHESIS` | `1` | GPU/CPU workload bound |
| `REQUEST_TIMEOUT_SECONDS` | `300` | Backend request timeout |
| `MANAGE_BACKEND` | `false` | Let the edge service launch the configured backend |

Do not expose the service publicly without `API_KEY` and a TLS-terminating reverse proxy. CORS is deliberately not enabled.

## Validation

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
python -m pytest backend/tests
docker compose config
```

The tests use a synthetic WAV-producing backend and never download model weights. Real synthesis is an explicit deployment smoke test because the model is large and accelerator-specific.

## Architecture notes

- CPU container: Debian/Python multi-architecture image for ARM64 and x86_64.
- NVIDIA container: CUDA 12.1 runtime selected by the Compose `nvidia` profile.
- Model downloads live under `/cache`; no weights are baked into images or source control.
- Legacy training notebooks remain as upstream research references, not as the supported runtime interface.

Licensed under the repository's MIT license. WhisperSpeech models and dependencies retain their respective licenses.
