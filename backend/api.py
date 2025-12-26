"""Text2Speech Backend API.

Exposes /synthesize endpoint for the Rust proxy.
Uses WhisperSpeech for TTS generation.
"""

import base64
import os
import tempfile
from pathlib import Path

import torch
import uvicorn
from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel
from whisperspeech.pipeline import Pipeline

app = FastAPI()

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Load model on startup
MODEL_REF = os.getenv("TTS_MODEL", "collabora/whisperspeech:s2a-q4-tiny-en+pl.model")
pipe = None


def get_pipeline():
    global pipe
    if pipe is None:
        pipe = Pipeline(s2a_ref=MODEL_REF)
    return pipe


class SynthesizeRequest(BaseModel):
    text: str


class SynthesizeResponse(BaseModel):
    audio: str  # base64 encoded WAV


@app.get("/health")
def health():
    return {"status": "ok"}


@app.post("/warm")
def warm():
    """Pre-load the model."""
    get_pipeline()
    return {"status": "warmed"}


@app.post("/synthesize")
def synthesize(request: SynthesizeRequest) -> SynthesizeResponse:
    """Generate speech from text."""
    pipeline = get_pipeline()
    
    with tempfile.NamedTemporaryFile(suffix=".wav", delete=False) as f:
        output_path = Path(f.name)
    
    try:
        pipeline.generate_to_file(str(output_path), request.text)
        audio_bytes = output_path.read_bytes()
        audio_b64 = base64.b64encode(audio_bytes).decode("utf-8")
        return SynthesizeResponse(audio=audio_b64)
    finally:
        if output_path.exists():
            output_path.unlink()


if __name__ == "__main__":
    if not torch.cuda.is_available():
        print("WARNING: No GPU detected, TTS will be slow")
    port = int(os.getenv("PORT", "7101"))
    uvicorn.run("api:app", host="0.0.0.0", port=port, reload=False)
