"""
Type annotations and protocols for Specado Python bindings.

This module provides comprehensive type hints for all Specado types,
ensuring full mypy compatibility and excellent IDE support.
"""

from typing import Any, Dict, List, Literal, Optional, Protocol, Union, runtime_checkable
from typing_extensions import TypedDict, NotRequired

# Core type aliases
MessageRole = Literal["system", "user", "assistant"]
StrictMode = Literal["warn", "error"]
FinishReason = Literal["stop", "length", "tool_call", "end_conversation", "other"]
SchemaType = Literal["prompt", "provider"]
TranslationMode = Literal["standard", "strict"]

# Dictionary representations for JSON serialization
class MessageDict(TypedDict):
    """Dictionary representation of a Message."""
    role: MessageRole
    content: str
    name: NotRequired[Optional[str]]
    metadata: NotRequired[Optional[Dict[str, Any]]]

class ToolDict(TypedDict):
    """Dictionary representation of a Tool."""
    name: str
    description: NotRequired[Optional[str]]
    json_schema: Dict[str, Any]

class SamplingParamsDict(TypedDict, total=False):
    """Dictionary representation of SamplingParams."""
    temperature: Optional[float]
    top_p: Optional[float]
    top_k: Optional[int]
    frequency_penalty: Optional[float]
    presence_penalty: Optional[float]

class LimitsDict(TypedDict, total=False):
    """Dictionary representation of Limits."""
    max_output_tokens: Optional[int]
    reasoning_tokens: Optional[int]
    max_prompt_tokens: Optional[int]

class MediaConfigDict(TypedDict, total=False):
    """Dictionary representation of MediaConfig."""
    input_images: Optional[List[Dict[str, Any]]]
    input_audio: Optional[Dict[str, Any]]
    output_audio: Optional[Dict[str, Any]]

class PromptSpecDict(TypedDict):
    """Dictionary representation of a PromptSpec."""
    model_class: str
    messages: List[MessageDict]
    tools: NotRequired[Optional[List[ToolDict]]]
    tool_choice: NotRequired[Optional[Union[str, Dict[str, str]]]]
    response_format: NotRequired[Optional[Union[str, Dict[str, Any]]]]
    sampling: NotRequired[Optional[SamplingParamsDict]]
    limits: NotRequired[Optional[LimitsDict]]
    media: NotRequired[Optional[MediaConfigDict]]
    strict_mode: StrictMode

class ProviderInfoDict(TypedDict):
    """Dictionary representation of ProviderInfo."""
    name: str
    base_url: str
    headers: Dict[str, str]

class EndpointConfigDict(TypedDict):
    """Dictionary representation of EndpointConfig."""
    method: str
    path: str
    protocol: str
    query: NotRequired[Optional[Dict[str, str]]]
    headers: NotRequired[Optional[Dict[str, str]]]

class EndpointsDict(TypedDict):
    """Dictionary representation of Endpoints."""
    chat_completion: EndpointConfigDict
    streaming_chat_completion: EndpointConfigDict

class InputModesDict(TypedDict):
    """Dictionary representation of InputModes."""
    messages: bool
    single_text: bool
    images: bool

class ToolingConfigDict(TypedDict):
    """Dictionary representation of ToolingConfig."""
    tools_supported: bool
    parallel_tool_calls_default: bool
    can_disable_parallel_tool_calls: bool
    disable_switch: NotRequired[Optional[Dict[str, Any]]]

class JsonOutputConfigDict(TypedDict):
    """Dictionary representation of JsonOutputConfig."""
    native_param: bool
    strategy: str

class ConstraintLimitsDict(TypedDict):
    """Dictionary representation of ConstraintLimits."""
    max_tool_schema_bytes: int
    max_system_prompt_bytes: int

class ConstraintsDict(TypedDict):
    """Dictionary representation of Constraints."""
    system_prompt_location: str
    forbid_unknown_top_level_fields: bool
    mutually_exclusive: List[List[str]]
    resolution_preferences: List[str]
    limits: ConstraintLimitsDict

class MappingsDict(TypedDict):
    """Dictionary representation of Mappings."""
    paths: Dict[str, str]
    flags: Dict[str, Any]

class SyncNormalizationDict(TypedDict):
    """Dictionary representation of SyncNormalization."""
    content_path: str
    finish_reason_path: str
    finish_reason_map: Dict[str, str]

class EventRouteDict(TypedDict):
    """Dictionary representation of EventRoute."""
    when: str
    emit: str
    text_path: NotRequired[Optional[str]]
    name_path: NotRequired[Optional[str]]
    args_path: NotRequired[Optional[str]]

class EventSelectorDict(TypedDict):
    """Dictionary representation of EventSelector."""
    type_path: str
    routes: List[EventRouteDict]

class StreamNormalizationDict(TypedDict):
    """Dictionary representation of StreamNormalization."""
    protocol: str
    event_selector: EventSelectorDict

class ResponseNormalizationDict(TypedDict):
    """Dictionary representation of ResponseNormalization."""
    sync: SyncNormalizationDict
    stream: StreamNormalizationDict

class ModelSpecDict(TypedDict):
    """Dictionary representation of ModelSpec."""
    id: str
    aliases: NotRequired[Optional[List[str]]]
    family: str
    endpoints: EndpointsDict
    input_modes: InputModesDict
    tooling: ToolingConfigDict
    json_output: JsonOutputConfigDict
    parameters: Dict[str, Any]
    constraints: ConstraintsDict
    mappings: MappingsDict
    response_normalization: ResponseNormalizationDict

class ProviderSpecDict(TypedDict):
    """Dictionary representation of a ProviderSpec."""
    spec_version: str
    provider: ProviderInfoDict
    models: List[ModelSpecDict]

class LossinessItemDict(TypedDict):
    """Dictionary representation of a LossinessItem."""
    code: str
    path: str
    message: str
    severity: str
    before: NotRequired[Optional[Any]]
    after: NotRequired[Optional[Any]]

class LossinessSummaryDict(TypedDict):
    """Dictionary representation of LossinessSummary."""
    total_items: int
    by_severity: Dict[str, int]
    by_code: Dict[str, int]

class LossinessReportDict(TypedDict):
    """Dictionary representation of LossinessReport."""
    items: List[LossinessItemDict]
    max_severity: str
    summary: LossinessSummaryDict

class TranslationMetadataDict(TypedDict):
    """Dictionary representation of TranslationMetadata."""
    provider: str
    model: str
    timestamp: str
    duration_ms: NotRequired[Optional[int]]
    strict_mode: StrictMode

class TranslationResultDict(TypedDict):
    """Dictionary representation of TranslationResult."""
    provider_request_json: Dict[str, Any]
    lossiness: LossinessReportDict
    metadata: NotRequired[Optional[TranslationMetadataDict]]

class ToolCallDict(TypedDict):
    """Dictionary representation of ToolCall."""
    name: str
    arguments: Dict[str, Any]
    id: NotRequired[Optional[str]]

class UniformResponseDict(TypedDict):
    """Dictionary representation of UniformResponse."""
    model: str
    content: str
    finish_reason: FinishReason
    tool_calls: NotRequired[Optional[List[ToolCallDict]]]
    raw_metadata: Dict[str, Any]

class ValidationResultDict(TypedDict):
    """Dictionary representation of ValidationResult."""
    is_valid: bool
    errors: List[str]

# Provider request type - flexible dict for provider-specific requests
ProviderRequest = Dict[str, Any]

# Protocol definitions for structural typing
@runtime_checkable
class PromptSpecProtocol(Protocol):
    """Protocol for PromptSpec-like objects."""
    
    @property
    def model_class(self) -> str:
        """Model class identifier."""
        ...
    
    @property
    def messages(self) -> List[Any]:
        """List of messages."""
        ...
    
    @property
    def strict_mode(self) -> str:
        """Strict mode setting."""
        ...
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary representation."""
        ...

@runtime_checkable
class ProviderSpecProtocol(Protocol):
    """Protocol for ProviderSpec-like objects."""
    
    @property
    def spec_version(self) -> str:
        """Specification version."""
        ...
    
    @property
    def provider(self) -> Any:
        """Provider information."""
        ...
    
    @property
    def models(self) -> List[Any]:
        """List of model specifications."""
        ...
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary representation."""
        ...

@runtime_checkable
class TranslationResultProtocol(Protocol):
    """Protocol for TranslationResult-like objects."""
    
    @property
    def provider_request_json(self) -> Dict[str, Any]:
        """Provider-specific request JSON."""
        ...
    
    @property
    def has_lossiness(self) -> bool:
        """Whether the translation has any lossiness."""
        ...
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary representation."""
        ...

@runtime_checkable
class UniformResponseProtocol(Protocol):
    """Protocol for UniformResponse-like objects."""
    
    @property
    def model(self) -> str:
        """Model identifier."""
        ...
    
    @property
    def content(self) -> str:
        """Response content."""
        ...
    
    @property
    def finish_reason(self) -> str:
        """Reason for finishing."""
        ...
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary representation."""
        ...

@runtime_checkable
class ValidationResultProtocol(Protocol):
    """Protocol for ValidationResult-like objects."""
    
    @property
    def is_valid(self) -> bool:
        """Whether the validation passed."""
        ...
    
    @property
    def errors(self) -> List[str]:
        """List of validation errors."""
        ...

# Union types for flexible input handling
PromptSpecInput = Union[PromptSpecProtocol, PromptSpecDict]
ProviderSpecInput = Union[ProviderSpecProtocol, ProviderSpecDict]
MessageInput = Union[MessageDict, Any]  # Any for Message objects
ToolInput = Union[ToolDict, Any]  # Any for Tool objects

# Type guards for runtime type checking
def is_prompt_spec_dict(obj: Any) -> bool:
    """Check if object is a valid PromptSpecDict."""
    if not isinstance(obj, dict):
        return False
    
    required_keys = {"model_class", "messages", "strict_mode"}
    return all(key in obj for key in required_keys)

def is_provider_spec_dict(obj: Any) -> bool:
    """Check if object is a valid ProviderSpecDict."""
    if not isinstance(obj, dict):
        return False
    
    required_keys = {"spec_version", "provider", "models"}
    return all(key in obj for key in required_keys)

def is_message_dict(obj: Any) -> bool:
    """Check if object is a valid MessageDict."""
    if not isinstance(obj, dict):
        return False
    
    required_keys = {"role", "content"}
    return all(key in obj for key in required_keys)

# Helper type aliases for common combinations
TranslationInput = Union[PromptSpecInput, ProviderSpecInput, str]
ValidationInput = Union[Dict[str, Any], Any]  # Any for typed objects