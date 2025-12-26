# dev-text2speech

Development workspace for the text-to-speech proxy/backend.

## Requirements
- Docker + GPU recommended (CPU works but slower)
- Built/tested on **aarch64**. For x86_64, pick the matching base image tag and rebuild locally.

## Build
```bash
docker build -t inference/text2speech:local .
```

## Run (standalone)
```bash
docker run --gpus all -d -p 7101:7101 inference/text2speech:local
```

## Run with docker-compose (root of repo)
```bash
docker compose up text2speech
```

## Notes
- Exposes API_PORT default 7101 (see docker-compose)
