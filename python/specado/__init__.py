from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Dict, Iterable, List, Optional

from ._native import Client as _NativeClient

__all__ = ["Client", "PromptSpec", "Message", "__version__"]

try:  # pragma: no cover - exercised during packaging verification
    from importlib import metadata as _metadata
except ImportError:  # Python < 3.8 fallback via importlib_metadata
    import importlib_metadata as _metadata  # type: ignore[assignment]

def _detect_version() -> str:
    try:
        return _metadata.version("specado")
    except _metadata.PackageNotFoundError:
        from pathlib import Path

        try:
            import tomllib  # Python >= 3.11
        except ModuleNotFoundError:  # pragma: no cover - fallback for older runtimes
            try:
                import tomli as tomllib  # type: ignore[no-redef]
            except ModuleNotFoundError:
                tomllib = None  # type: ignore[assignment]

        pyproject = Path(__file__).resolve().parents[2] / "pyproject.toml"
        if pyproject.exists() and tomllib is not None:
            with pyproject.open("rb") as fh:
                data = tomllib.load(fh)
            project = data.get("project") or {}
            version = project.get("version")
            if isinstance(version, str):
                return version
        return "0.0.0"

__version__ = _detect_version()

del _metadata
del _detect_version


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

    def __init__(
        self,
        provider_path: str,
        watch: Optional[bool] = None,
        audit_config: Optional[Dict[str, Any]] = None,
    ) -> None:
        """Create a Specado client.

        Args:
            provider_path: Path to the provider YAML or JSON file.
            watch: Enable experimental hot-reload plumbing (no watcher started yet).
            audit_config: Optional audit logging configuration forwarded to the core layer.
        """

        self._client = _NativeClient(
            provider_path,
            watch=watch,
            audit_config=audit_config,
        )

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
