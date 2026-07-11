import importlib.util
from pathlib import Path
from fastapi.testclient import TestClient

spec = importlib.util.spec_from_file_location("tts_api", Path(__file__).parents[1] / "api.py")
api = importlib.util.module_from_spec(spec)
spec.loader.exec_module(api)
client = TestClient(api.app)

def test_health_does_not_load_model():
    assert client.get("/health").status_code == 200
    assert api.pipe is None
    assert client.get("/ready").status_code == 503

def test_synthesis_returns_wav(monkeypatch, tmp_path):
    class FakePipeline:
        def generate_to_file(self, path, text):
            Path(path).write_bytes(b"RIFF-test")
    monkeypatch.setattr(api, "get_pipeline", lambda: FakePipeline())
    response = client.post("/synthesize", json={"text": "hello"})
    assert response.status_code == 200
    assert response.headers["content-type"] == "audio/wav"
    assert response.content == b"RIFF-test"
