from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Any, Dict, List, Optional, Union

from ._native import (
    Client as _NativeClient,
    create_prompt as _native_create_prompt,
    load_prompt as _native_load_prompt,
    simple_prompt as _native_simple_prompt,
)

__all__ = [
    "Client",
    "PromptSpec",
    "Message",
    "__version__",
    "load_prompt",
    "build_prompt",
    "create_prompt",
]

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

_catalog_candidates = [
    Path(__file__).resolve().parent / "providers",
    Path(__file__).resolve().parents[2] / "crates" / "specado-providers" / "providers",
]

_BUNDLED_PROVIDERS_DIR: Optional[Path] = None
for _candidate in _catalog_candidates:
    if _candidate.exists():
        _BUNDLED_PROVIDERS_DIR = _candidate
        break

del _metadata
del _detect_version


def load_prompt(path: Union[str, Path]) -> Dict[str, Any]:
    """Load a prompt specification from JSON or YAML.

    Args:
        path: Path to a `.json`, `.yaml`, or `.yml` prompt file.

    Returns:
        A dictionary ready to pass to ``Client.complete``.

    Raises:
        RuntimeError: If the file cannot be read or parsed.
    """

    resolved = Path(path).expanduser().resolve()
    if not resolved.exists():
        raise RuntimeError(f"Prompt file not found: {resolved}")

    return _native_load_prompt(str(resolved))


def create_prompt(options: Dict[str, Any]) -> Dict[str, Any]:
    """Construct a prompt specification from a builder dictionary."""

    return _native_create_prompt(dict(options))


def build_prompt(
    user_message: str,
    *,
    system_message: Optional[str] = None,
    temperature: Optional[float] = None,
    sampling: Optional[Dict[str, Any]] = None,
    strict_mode: str = "Warn",
) -> Dict[str, Any]:
    """Create a minimal prompt spec from high-level arguments."""

    options: Dict[str, Any] = {"message": user_message, "strict_mode": strict_mode}
    if system_message:
        options["system"] = system_message
    if sampling:
        options["sampling"] = dict(sampling)
    if temperature is not None:
        options["temperature"] = temperature

    return _native_simple_prompt(options)


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
        provider: str,
        *,
        model: Optional[str] = None,
        providers_dir: Optional[Union[str, Path]] = None,
        watch: Optional[bool] = None,
        audit_config: Optional[Dict[str, Any]] = None,
    ) -> None:
        """Create a Specado client.

        Args:
            provider: Provider identifier (friendly name or spec path).
            model: Optional model identifier to disambiguate multi-model providers.
            providers_dir: Override for the provider catalog directory.
            watch: Enable experimental hot-reload plumbing (no watcher started yet).
            audit_config: Optional audit logging configuration forwarded to the core layer.
        """

        if _BUNDLED_PROVIDERS_DIR and _BUNDLED_PROVIDERS_DIR.exists():
            bundled_dir = _BUNDLED_PROVIDERS_DIR
        else:
            bundled_dir = None
        providers_dir_arg: Optional[str]
        if providers_dir is None:
            providers_dir_arg = str(bundled_dir) if bundled_dir else None
        else:
            providers_dir_arg = str(Path(providers_dir))

        self._client = _NativeClient(
            provider,
            model=model,
            providers_dir=providers_dir_arg,
            watch=watch,
            audit_config=audit_config,
        )
        self._providers_dir = providers_dir_arg

    def complete(self, prompt: PromptSpec | Dict[str, Any]) -> Dict[str, Any]:
        if isinstance(prompt, PromptSpec):
            payload = prompt.to_dict()
        else:
            payload = prompt
        return self._client.complete(payload)

    def complete_file(self, path: Union[str, Path]) -> Dict[str, Any]:
        """Load a prompt from disk and execute it."""

        resolved = Path(path).expanduser().resolve()
        return self._client.complete_file(str(resolved))

    def complete_text(
        self,
        user_message: str,
        *,
        system_message: Optional[str] = None,
        temperature: Optional[float] = None,
        sampling: Optional[Dict[str, Any]] = None,
        strict_mode: str = "Warn",
    ) -> Dict[str, Any]:
        """Execute a simple text prompt without constructing a full spec."""

        options: Dict[str, Any] = {"strict_mode": strict_mode}
        if system_message is not None:
            options["system"] = system_message
        if sampling is not None:
            options["sampling"] = dict(sampling)
        if temperature is not None:
            options["temperature"] = temperature

        return self._client.complete_text(user_message, options=options)

    async def acomplete(self, prompt: PromptSpec | Dict[str, Any]) -> Dict[str, Any]:
        from asyncio import get_running_loop

        loop = get_running_loop()
        return await loop.run_in_executor(None, self.complete, prompt)


from . import compat  # noqa: E402

__all__.append("compat")
