from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Dict, Iterable, List, Optional

from ._native import Client as _NativeClient

__all__ = ["Client", "PromptSpec", "Message", "__version__"]
__version__ = "0.1.0"


@dataclass
class Message:
    role: str
    content: str


@dataclass
class PromptSpec:
    messages: List[Message]
    sampling: Optional[Dict[str, float]] = None
    strict_mode: str = "Warn"

    def to_dict(self) -> Dict[str, Any]:
        return {
            "version": "1",
            "messages": [
                {"role": message.role, "content": message.content}
                for message in self.messages
            ],
            "sampling": self.sampling or {},
            "strict_mode": self.strict_mode,
        }


class Client:
    """High-level Python wrapper around the native Specado client."""

    def __init__(self, provider_path: str):
        self._client = _NativeClient(provider_path)

    def complete(self, prompt: PromptSpec | Dict[str, Any]) -> Dict[str, Any]:
        if isinstance(prompt, PromptSpec):
            payload = prompt.to_dict()
        else:
            payload = prompt
        return self._client.complete(payload)

    async def acomplete(self, prompt: PromptSpec | Dict[str, Any]) -> Dict[str, Any]:
        from asyncio import get_running_loop

        loop = get_running_loop()
        return await loop.run_in_executor(None, self.complete, prompt)


from . import compat  # noqa: E402

__all__.append("compat")
