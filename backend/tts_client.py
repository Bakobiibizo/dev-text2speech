import requests
from pathlib import Path
from utilities.data_models import Text2SpeechRequest, TTSGenerationResponse


def generate_tts(text: str) -> TTSGenerationResponse:
    url = "http://localhost:7097/v1/api/tts"
    headers = {"Content-Type": "application/json"}
    data = Text2SpeechRequest(prompt=text).model_dump_json()
    response = requests.post(url, headers=headers, data=data)
    response.raise_for_status()
    return TTSGenerationResponse(**response.json())

def download_audio(url: str) -> bytes:
    r = requests.get(url)
    r.raise_for_status()
    return r.content

def save_to_file(filename: str, data: bytes):
    with open(filename, "wb") as f:
        f.write(data)
    
if __name__ == "__main__":
    resp = generate_tts("Hello World")
    audio_bytes = download_audio(resp.url) if resp.url else b""
    # Save using server-provided filename under local data/
    out_dir = Path("./data")
    out_dir.mkdir(exist_ok=True)
    out_path = out_dir / (Path(resp.audio_filename).name if resp.audio_filename else "output.wav")
    save_to_file(str(out_path), audio_bytes)
    print(f"Saved: {out_path}")
    
