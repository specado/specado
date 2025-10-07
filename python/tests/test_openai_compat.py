import json
import os
import socket
import threading
from pathlib import Path
from tempfile import TemporaryDirectory
from http.server import BaseHTTPRequestHandler, HTTPServer

import pytest

from specado import Client, Message, PromptSpec
from specado.compat import OpenAI


class _MockHandler(BaseHTTPRequestHandler):
    def do_POST(self):
        length = int(self.headers.get("content-length", 0))
        _ = self.rfile.read(length)
        body = {
            "data": {
                "content": "hello from python",
                "finish_reason": "stop",
            }
        }
        payload = json.dumps(body).encode("utf-8")
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(payload)))
        self.end_headers()
        self.wfile.write(payload)

    def log_message(self, *args, **kwargs):  # pragma: no cover - silence server logs
        return


def _free_port() -> int:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.bind(("", 0))
        return s.getsockname()[1]


@pytest.fixture(name="mock_server")
def _mock_server():
    port = _free_port()
    server = HTTPServer(("127.0.0.1", port), _MockHandler)

    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    yield f"http://127.0.0.1:{port}/chat"
    server.shutdown()
    thread.join(timeout=1)


def _write_provider(temp_dir: Path, url: str, token_env: str) -> Path:
    spec = f"""
provider: test
models:
  - id: test-model
auth:
  type: bearer
  token_env: {token_env}
endpoints:
  chat:
    method: POST
    url: {url}
    headers:
      content-type: application/json
mappings:
  request:
    - from: $.messages
      to: $.body.messages
  response:
    - from: $.data.content
      to: content
    - from: $.data.finish_reason
      to: finish_reason
constraints:
  supports:
    json_mode: true
    tools: true
""".strip()
    provider_path = temp_dir / "provider.yaml"
    provider_path.write_text(spec, encoding="utf-8")
    return provider_path


@pytest.mark.parametrize("temperature", [None, 0.3])
def test_python_client_and_openai_wrapper(mock_server: str, temperature: float | None):
    token_env = "SPECADO_PY_TEST_TOKEN"
    os.environ[token_env] = "py-secret"
    try:
        with TemporaryDirectory() as tmp:
            temp_path = Path(tmp)
            provider_path = _write_provider(temp_path, mock_server, token_env)

            prompt = PromptSpec(messages=[Message(role="user", content="Hello!")])
            client = Client(str(provider_path))
            response = client.complete(prompt)
            assert response["content"] == "hello from python"

            openai = OpenAI(str(provider_path))
            messages = [
                {"role": "user", "content": "Hello!"},
            ]
            completion = openai.chat.completions.create(
                model="test-model",
                messages=messages,
                temperature=temperature,
            )
            assert completion.choices[0].message.content == "hello from python"
    finally:
        os.environ.pop(token_env, None)
