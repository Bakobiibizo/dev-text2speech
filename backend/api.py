"""Text2Speech Backend API.

Exposes /synthesize endpoint for the Rust proxy.
Uses WhisperSpeech for TTS generation.
"""

import os
import tempfile
from pathlib import Path

import uvicorn
from fastapi import FastAPI, HTTPException
from fastapi.responses import Response
from pydantic import BaseModel, Field

app = FastAPI()

MODEL_REF = os.getenv("TTS_MODEL", "collabora/whisperspeech:s2a-q4-tiny-en+pl.model")
pipe = None


def get_pipeline():
    global pipe
    if pipe is None:
        # Import lazily: liveness and packaging checks must not download models.
        from whisperspeech.pipeline import Pipeline
        pipe = Pipeline(s2a_ref=MODEL_REF)
    return pipe


class SynthesizeRequest(BaseModel):
    text: str = Field(min_length=1, max_length=int(os.getenv("MAX_TEXT_CHARS", "5000")))
    voice: str | None = None


@app.get("/health")
def health():
    return {"status": "ok"}


@app.get("/ready")
def ready():
    if pipe is None:
        raise HTTPException(status_code=503, detail="model is not loaded")
    return {"status": "ready", "model": MODEL_REF}


@app.post("/warm")
def warm():
    """Pre-load the model."""
    get_pipeline()
    return {"status": "warmed"}


@app.post("/synthesize")
def synthesize(request: SynthesizeRequest) -> Response:
    """Generate speech from text."""
    pipeline = get_pipeline()
    
    with tempfile.NamedTemporaryFile(suffix=".wav", delete=False) as f:
        output_path = Path(f.name)
    
    try:
        pipeline.generate_to_file(str(output_path), request.text)
        audio_bytes = output_path.read_bytes()
        return Response(content=audio_bytes, media_type="audio/wav")
    finally:
        if output_path.exists():
            output_path.unlink()


if __name__ == "__main__":
    import torch
    if not torch.cuda.is_available():
        print("WARNING: No GPU detected, TTS will be slow")
    port = int(os.getenv("PORT", os.getenv("API_PORT", "7097")))
    uvicorn.run("api:app", host="0.0.0.0", port=port, reload=False)
