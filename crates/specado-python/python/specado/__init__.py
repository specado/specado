"""
Specado Python Bindings

Universal LLM Specification Language for translating prompts across providers.

This package provides Python bindings for the Specado library, enabling:
- Translation of prompts to provider-specific formats
- Validation of prompt and provider specifications
- Execution of provider requests with normalized responses
- Support for both synchronous and asynchronous operations

Basic usage:
    >>> import specado
    >>> 
    >>> # Create a prompt
    >>> prompt = specado.PromptSpec(
    ...     model_class="Chat",
    ...     messages=[
    ...         specado.Message("user", "Hello, world!")
    ...     ]
    ... )
    >>> 
    >>> # Load a provider specification
    >>> provider_spec = specado.ProviderSpec.from_dict(provider_data)
    >>> 
    >>> # Translate the prompt
    >>> result = specado.translate(prompt, provider_spec, "gpt-4")
    >>> 
    >>> # Execute the request
    >>> response = await specado.run(result.provider_request_json, provider_spec)
    >>> print(response.content)
"""

from ._specado import (
    # Core functions
    translate,
    validate,
    run_async as run,
    run_sync,
    version,
    create_provider_request,
    
    # Type classes
    PromptSpec,
    ProviderSpec,
    Message,
    Tool,
    ToolChoice,
    ResponseFormat,
    SamplingParams,
    Limits,
    MediaConfig,
    ProviderInfo,
    ModelSpec,
    TranslationResult,
    UniformResponse,
    ValidationResult,
    
    # Exceptions
    SpecadoError,
    TranslationError,
    ValidationError,
    ProviderError,
    TimeoutError,
)

from .types import (
    # Type aliases and protocols
    PromptSpecDict,
    ProviderSpecDict,
    MessageDict,
    ToolDict,
    ProviderRequest,
    SchemaType,
)

__version__ = "0.1.0"
__author__ = "Specado Team"
__email__ = "contact@specado.com"
__license__ = "Apache-2.0"
__url__ = "https://www.specado.com"
__repository__ = "https://github.com/specado/specado"

__all__ = [
    # Core functions
    "translate",
    "validate", 
    "run",
    "run_sync",
    "version",
    "create_provider_request",
    
    # Type classes
    "PromptSpec",
    "ProviderSpec", 
    "Message",
    "Tool",
    "ToolChoice",
    "ResponseFormat",
    "SamplingParams",
    "Limits",
    "MediaConfig",
    "ProviderInfo",
    "ModelSpec",
    "TranslationResult",
    "UniformResponse",
    "ValidationResult",
    
    # Type aliases
    "PromptSpecDict",
    "ProviderSpecDict",
    "MessageDict", 
    "ToolDict",
    "ProviderRequest",
    "SchemaType",
    
    # Exceptions
    "SpecadoError",
    "TranslationError",
    "ValidationError", 
    "ProviderError",
    "TimeoutError",
    
    # Metadata
    "__version__",
    "__author__",
    "__email__",
    "__license__",
    "__url__",
    "__repository__",
]