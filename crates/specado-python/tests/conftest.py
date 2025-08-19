"""
Pytest configuration and fixtures for Specado Python binding tests.
"""

import pytest
from typing import Dict, Any, List
import json

import specado
from specado import (
    PromptSpec, ProviderSpec, Message, Tool, SamplingParams, Limits,
    ProviderInfo, ModelSpec, TranslationResult
)


@pytest.fixture
def sample_message() -> Message:
    """Create a sample message for testing."""
    return Message("user", "Hello, world!")


@pytest.fixture  
def sample_system_message() -> Message:
    """Create a sample system message for testing."""
    return Message("system", "You are a helpful assistant.")


@pytest.fixture
def sample_prompt(sample_message: Message) -> PromptSpec:
    """Create a sample PromptSpec for testing."""
    return PromptSpec(
        model_class="Chat",
        messages=[sample_message],
        strict_mode="warn"
    )


@pytest.fixture
def complex_prompt() -> PromptSpec:
    """Create a complex PromptSpec with all optional fields for testing."""
    messages = [
        Message("system", "You are a helpful assistant."),
        Message("user", "What's the weather like?"),
        Message("assistant", "I'd need your location to check the weather."),
        Message("user", "I'm in San Francisco.")
    ]
    
    tool = Tool(
        name="get_weather",
        description="Get current weather for a location",
        json_schema={
            "type": "object",
            "properties": {
                "location": {
                    "type": "string",
                    "description": "City name"
                }
            },
            "required": ["location"]
        }
    )
    
    sampling = SamplingParams(
        temperature=0.7,
        top_p=0.9,
        frequency_penalty=0.1,
        presence_penalty=0.1
    )
    
    limits = Limits(
        max_output_tokens=1000,
        reasoning_tokens=500
    )
    
    return PromptSpec(
        model_class="Chat",
        messages=messages,
        tools=[tool],
        tool_choice="auto",
        sampling=sampling,
        limits=limits,
        strict_mode="warn"
    )


@pytest.fixture
def sample_provider_info() -> ProviderInfo:
    """Create a sample ProviderInfo for testing."""
    return ProviderInfo(
        name="test-provider",
        base_url="https://api.test-provider.com",
        headers={"Authorization": "Bearer test-token"}
    )


@pytest.fixture
def sample_model_spec() -> ModelSpec:
    """Create a sample ModelSpec for testing."""
    from specado.types import (
        EndpointsDict, InputModesDict, ToolingConfigDict, 
        JsonOutputConfigDict, ConstraintsDict, MappingsDict, 
        ResponseNormalizationDict, ConstraintLimitsDict,
        SyncNormalizationDict, StreamNormalizationDict, EventSelectorDict
    )
    
    endpoints: EndpointsDict = {
        "chat_completion": {
            "method": "POST",
            "path": "/v1/chat/completions",
            "protocol": "https"
        },
        "streaming_chat_completion": {
            "method": "POST", 
            "path": "/v1/chat/completions",
            "protocol": "https"
        }
    }
    
    input_modes: InputModesDict = {
        "messages": True,
        "single_text": False,
        "images": False
    }
    
    tooling: ToolingConfigDict = {
        "tools_supported": True,
        "parallel_tool_calls_default": True,
        "can_disable_parallel_tool_calls": True
    }
    
    json_output: JsonOutputConfigDict = {
        "native_param": True,
        "strategy": "response_format"
    }
    
    constraint_limits: ConstraintLimitsDict = {
        "max_tool_schema_bytes": 10000,
        "max_system_prompt_bytes": 5000
    }
    
    constraints: ConstraintsDict = {
        "system_prompt_location": "first",
        "forbid_unknown_top_level_fields": False,
        "mutually_exclusive": [],
        "resolution_preferences": ["temperature", "top_p"],
        "limits": constraint_limits
    }
    
    mappings: MappingsDict = {
        "paths": {
            "model": "$.model",
            "messages": "$.messages", 
            "temperature": "$.temperature"
        },
        "flags": {}
    }
    
    sync_norm: SyncNormalizationDict = {
        "content_path": "$.choices[0].message.content",
        "finish_reason_path": "$.choices[0].finish_reason",
        "finish_reason_map": {
            "stop": "stop",
            "length": "length",
            "tool_calls": "tool_call"
        }
    }
    
    event_selector: EventSelectorDict = {
        "type_path": "$.type",
        "routes": []
    }
    
    stream_norm: StreamNormalizationDict = {
        "protocol": "sse",
        "event_selector": event_selector
    }
    
    response_norm: ResponseNormalizationDict = {
        "sync": sync_norm,
        "stream": stream_norm
    }
    
    return ModelSpec.from_dict({
        "id": "test-model",
        "aliases": ["test-model-alias"],
        "family": "test",
        "endpoints": endpoints,
        "input_modes": input_modes,
        "tooling": tooling,
        "json_output": json_output,
        "parameters": {},
        "constraints": constraints,
        "mappings": mappings,
        "response_normalization": response_norm
    })


@pytest.fixture
def sample_provider(sample_provider_info: ProviderInfo, sample_model_spec: ModelSpec) -> ProviderSpec:
    """Create a sample ProviderSpec for testing."""
    return ProviderSpec(
        spec_version="1.0.0",
        provider=sample_provider_info,
        models=[sample_model_spec]
    )


@pytest.fixture
def multi_model_provider(sample_provider_info: ProviderInfo, sample_model_spec: ModelSpec) -> ProviderSpec:
    """Create a ProviderSpec with multiple models for testing."""
    # Create additional model specs
    model_specs = [sample_model_spec]
    
    for i in range(2, 5):  # Add models 2, 3, 4
        model_dict = sample_model_spec.to_dict()
        model_dict["id"] = f"test-model-{i}"
        model_dict["aliases"] = [f"test-model-{i}-alias"]
        model_specs.append(ModelSpec.from_dict(model_dict))
    
    return ProviderSpec(
        spec_version="1.0.0",
        provider=sample_provider_info,
        models=model_specs
    )


@pytest.fixture
def sample_prompt_dict(sample_prompt: PromptSpec) -> Dict[str, Any]:
    """Create a sample prompt as dictionary for testing."""
    return sample_prompt.to_dict()


@pytest.fixture
def sample_provider_dict(sample_provider: ProviderSpec) -> Dict[str, Any]:
    """Create a sample provider as dictionary for testing."""
    return sample_provider.to_dict()


@pytest.fixture
def sample_translation_result(sample_prompt: PromptSpec, sample_provider: ProviderSpec) -> TranslationResult:
    """Create a sample TranslationResult for testing."""
    return specado.translate(
        prompt=sample_prompt,
        provider_spec=sample_provider,
        model_id="test-model"
    )


@pytest.fixture
def sample_provider_request(sample_translation_result: TranslationResult) -> Dict[str, Any]:
    """Create a sample provider request for testing."""
    return sample_translation_result.provider_request_json


# Mock fixtures for testing without real providers
@pytest.fixture
def mock_provider() -> ProviderSpec:
    """Create a mock provider that doesn't make real API calls."""
    from specado.types import ProviderSpecDict
    
    mock_spec: ProviderSpecDict = {
        "spec_version": "1.0.0",
        "provider": {
            "name": "mock-provider",
            "base_url": "https://mock.api.com",
            "headers": {}
        },
        "models": [
            {
                "id": "mock-model",
                "family": "mock",
                "endpoints": {
                    "chat_completion": {
                        "method": "POST",
                        "path": "/mock/chat",
                        "protocol": "https"
                    },
                    "streaming_chat_completion": {
                        "method": "POST",
                        "path": "/mock/chat/stream", 
                        "protocol": "https"
                    }
                },
                "input_modes": {
                    "messages": True,
                    "single_text": False,
                    "images": False
                },
                "tooling": {
                    "tools_supported": False,
                    "parallel_tool_calls_default": False,
                    "can_disable_parallel_tool_calls": False
                },
                "json_output": {
                    "native_param": False,
                    "strategy": "none"
                },
                "parameters": {},
                "constraints": {
                    "system_prompt_location": "first",
                    "forbid_unknown_top_level_fields": False,
                    "mutually_exclusive": [],
                    "resolution_preferences": [],
                    "limits": {
                        "max_tool_schema_bytes": 1000,
                        "max_system_prompt_bytes": 1000
                    }
                },
                "mappings": {
                    "paths": {},
                    "flags": {}
                },
                "response_normalization": {
                    "sync": {
                        "content_path": "$.content",
                        "finish_reason_path": "$.finish_reason",
                        "finish_reason_map": {}
                    },
                    "stream": {
                        "protocol": "sse",
                        "event_selector": {
                            "type_path": "$.type",
                            "routes": []
                        }
                    }
                }
            }
        ]
    }
    
    return ProviderSpec.from_dict(mock_spec)


# Test data fixtures
@pytest.fixture
def test_messages() -> List[Dict[str, Any]]:
    """Provide various test messages."""
    return [
        {"role": "system", "content": "You are a helpful assistant."},
        {"role": "user", "content": "Hello!"},
        {"role": "assistant", "content": "Hi there! How can I help you?"},
        {"role": "user", "content": "What's 2+2?"},
        {"role": "assistant", "content": "2+2 equals 4."}
    ]


@pytest.fixture
def test_tools() -> List[Dict[str, Any]]:
    """Provide various test tools."""
    return [
        {
            "name": "get_weather",
            "description": "Get current weather",
            "json_schema": {
                "type": "object",
                "properties": {
                    "location": {"type": "string"}
                },
                "required": ["location"]
            }
        },
        {
            "name": "calculate",
            "description": "Perform calculations", 
            "json_schema": {
                "type": "object",
                "properties": {
                    "expression": {"type": "string"}
                },
                "required": ["expression"]
            }
        }
    ]


# Performance testing setup
@pytest.fixture(scope="session")
def benchmark_provider() -> ProviderSpec:
    """Create a provider optimized for benchmarking."""
    # Use the same mock provider but with session scope for performance
    from specado.types import ProviderSpecDict
    
    benchmark_spec: ProviderSpecDict = {
        "spec_version": "1.0.0",
        "provider": {
            "name": "benchmark-provider",
            "base_url": "https://benchmark.api.com",
            "headers": {}
        },
        "models": [
            {
                "id": "benchmark-model",
                "family": "benchmark",
                "endpoints": {
                    "chat_completion": {
                        "method": "POST",
                        "path": "/benchmark/chat",
                        "protocol": "https"
                    },
                    "streaming_chat_completion": {
                        "method": "POST",
                        "path": "/benchmark/chat/stream",
                        "protocol": "https"  
                    }
                },
                "input_modes": {
                    "messages": True,
                    "single_text": False,
                    "images": False
                },
                "tooling": {
                    "tools_supported": False,
                    "parallel_tool_calls_default": False,
                    "can_disable_parallel_tool_calls": False
                },
                "json_output": {
                    "native_param": False,
                    "strategy": "none"
                },
                "parameters": {},
                "constraints": {
                    "system_prompt_location": "first",
                    "forbid_unknown_top_level_fields": False,
                    "mutually_exclusive": [],
                    "resolution_preferences": [],
                    "limits": {
                        "max_tool_schema_bytes": 1000,
                        "max_system_prompt_bytes": 1000
                    }
                },
                "mappings": {
                    "paths": {},
                    "flags": {}
                },
                "response_normalization": {
                    "sync": {
                        "content_path": "$.content",
                        "finish_reason_path": "$.finish_reason", 
                        "finish_reason_map": {}
                    },
                    "stream": {
                        "protocol": "sse",
                        "event_selector": {
                            "type_path": "$.type",
                            "routes": []
                        }
                    }
                }
            }
        ]
    }
    
    return ProviderSpec.from_dict(benchmark_spec)


# Configuration for different test types
def pytest_configure(config):
    """Configure pytest with custom markers."""
    config.addinivalue_line(
        "markers", "benchmark: mark test as a benchmark test"
    )
    config.addinivalue_line(
        "markers", "integration: mark test as an integration test"
    )
    config.addinivalue_line(
        "markers", "slow: mark test as slow running"
    )


def pytest_collection_modifyitems(config, items):
    """Modify test collection to add markers."""
    for item in items:
        # Add benchmark marker to benchmark tests
        if "benchmark" in item.name or "performance" in item.name:
            item.add_marker(pytest.mark.benchmark)
        
        # Add integration marker to integration tests  
        if "integration" in item.name or "workflow" in item.name:
            item.add_marker(pytest.mark.integration)
        
        # Add slow marker to tests that might be slow
        if "concurrent" in item.name or "large" in item.name:
            item.add_marker(pytest.mark.slow)


# Utilities for test cleanup
@pytest.fixture(autouse=True)
def cleanup_after_test():
    """Cleanup fixture that runs after each test."""
    yield
    # Add any cleanup logic here if needed
    pass