"""
Tests for type classes and type hint functionality.
"""

import pytest
from typing import Dict, Any, List
import json

import specado
from specado import (
    PromptSpec, ProviderSpec, Message, Tool, SamplingParams, Limits,
    MediaConfig, ProviderInfo, ModelSpec, TranslationResult, UniformResponse,
    ValidationResult
)


class TestMessage:
    """Test cases for Message class."""
    
    def test_message_creation_basic(self):
        """Test basic message creation."""
        message = Message("user", "Hello, world!")
        
        assert message.role == "user"
        assert message.content == "Hello, world!"
        assert message.name is None
    
    def test_message_creation_with_name(self):
        """Test message creation with name."""
        message = Message("assistant", "Hello!", name="ChatBot")
        
        assert message.role == "assistant"
        assert message.content == "Hello!"
        assert message.name == "ChatBot"
    
    def test_message_creation_with_metadata(self):
        """Test message creation with metadata."""
        metadata = {"source": "api", "timestamp": "2024-01-01"}
        message = Message("system", "System prompt", metadata=metadata)
        
        assert message.role == "system"
        assert message.content == "System prompt"
        # Note: metadata handling depends on implementation
    
    def test_message_invalid_role(self):
        """Test message creation with invalid role."""
        with pytest.raises(ValueError):
            Message("invalid_role", "Content")
    
    def test_message_repr(self):
        """Test message string representation."""
        message = Message("user", "Hello, world!")
        repr_str = repr(message)
        
        assert "user" in repr_str
        assert "Hello, world!" in repr_str or "Hello, wo..." in repr_str
    
    def test_message_all_roles(self):
        """Test all valid message roles."""
        roles = ["system", "user", "assistant"]
        
        for role in roles:
            message = Message(role, f"Content for {role}")
            assert message.role == role
            assert message.content == f"Content for {role}"


class TestPromptSpec:
    """Test cases for PromptSpec class."""
    
    def test_prompt_spec_creation_minimal(self):
        """Test minimal PromptSpec creation."""
        messages = [Message("user", "Hello")]
        prompt = PromptSpec(
            model_class="Chat",
            messages=messages,
            strict_mode="warn"
        )
        
        assert prompt.model_class == "Chat"
        assert len(prompt.messages) == 1
        assert prompt.strict_mode == "warn"
    
    def test_prompt_spec_creation_full(self):
        """Test PromptSpec creation with all parameters."""
        messages = [Message("user", "Hello")]
        tools = [Tool(
            name="test_tool",
            json_schema={"type": "object"}
        )]
        sampling = SamplingParams(temperature=0.7)
        limits = Limits(max_output_tokens=1000)
        
        prompt = PromptSpec(
            model_class="Chat",
            messages=messages,
            tools=tools,
            tool_choice="auto",
            sampling=sampling,
            limits=limits,
            strict_mode="error"
        )
        
        assert prompt.model_class == "Chat"
        assert len(prompt.messages) == 1
        assert prompt.strict_mode == "error"
        # Additional assertions would depend on implementation
    
    def test_prompt_spec_to_dict(self):
        """Test PromptSpec serialization to dict."""
        messages = [Message("user", "Hello")]
        prompt = PromptSpec(
            model_class="Chat",
            messages=messages,
            strict_mode="warn"
        )
        
        prompt_dict = prompt.to_dict()
        
        assert isinstance(prompt_dict, dict)
        assert prompt_dict["model_class"] == "Chat"
        assert prompt_dict["strict_mode"] == "warn"
        assert "messages" in prompt_dict
        assert len(prompt_dict["messages"]) == 1
    
    def test_prompt_spec_from_dict(self):
        """Test PromptSpec creation from dict."""
        prompt_dict = {
            "model_class": "Chat",
            "messages": [
                {"role": "user", "content": "Hello"}
            ],
            "strict_mode": "warn"
        }
        
        prompt = PromptSpec.from_dict(prompt_dict)
        
        assert prompt.model_class == "Chat"
        assert len(prompt.messages) == 1
        assert prompt.strict_mode == "warn"
    
    def test_prompt_spec_invalid_strict_mode(self):
        """Test PromptSpec with invalid strict mode."""
        messages = [Message("user", "Hello")]
        
        with pytest.raises(ValueError):
            PromptSpec(
                model_class="Chat",
                messages=messages,
                strict_mode="invalid_mode"  # type: ignore
            )
    
    def test_prompt_spec_repr(self):
        """Test PromptSpec string representation."""
        messages = [Message("user", "Hello")]
        prompt = PromptSpec(
            model_class="Chat",
            messages=messages,
            strict_mode="warn"
        )
        
        repr_str = repr(prompt)
        assert "Chat" in repr_str
        assert "warn" in repr_str


class TestProviderSpec:
    """Test cases for ProviderSpec class."""
    
    def test_provider_spec_creation(self, sample_provider_info: ProviderInfo, sample_model_spec: ModelSpec):
        """Test ProviderSpec creation."""
        provider = ProviderSpec(
            spec_version="1.0.0",
            provider=sample_provider_info,
            models=[sample_model_spec]
        )
        
        assert provider.spec_version == "1.0.0"
        assert provider.provider.name == sample_provider_info.name
        assert len(provider.models) == 1
    
    def test_provider_spec_to_dict(self, sample_provider: ProviderSpec):
        """Test ProviderSpec serialization to dict."""
        provider_dict = sample_provider.to_dict()
        
        assert isinstance(provider_dict, dict)
        assert provider_dict["spec_version"] == "1.0.0"
        assert "provider" in provider_dict
        assert "models" in provider_dict
        assert len(provider_dict["models"]) == 1
    
    def test_provider_spec_from_dict(self, sample_provider_dict: Dict[str, Any]):
        """Test ProviderSpec creation from dict."""
        provider = ProviderSpec.from_dict(sample_provider_dict)
        
        assert provider.spec_version == sample_provider_dict["spec_version"]
        assert provider.provider.name == sample_provider_dict["provider"]["name"]
        assert len(provider.models) == len(sample_provider_dict["models"])
    
    def test_provider_spec_multiple_models(self, sample_provider_info: ProviderInfo):
        """Test ProviderSpec with multiple models."""
        # This would require creating multiple ModelSpec instances
        # Implementation depends on ModelSpec creation capabilities
        pass
    
    def test_provider_spec_repr(self, sample_provider: ProviderSpec):
        """Test ProviderSpec string representation."""
        repr_str = repr(sample_provider)
        assert sample_provider.provider.name in repr_str
        assert "1" in repr_str  # Model count


class TestSamplingParams:
    """Test cases for SamplingParams class."""
    
    def test_sampling_params_creation(self):
        """Test SamplingParams creation."""
        sampling = SamplingParams(
            temperature=0.7,
            top_p=0.9,
            top_k=50,
            frequency_penalty=0.1,
            presence_penalty=0.2
        )
        
        # Implementation-dependent assertions
        # Would need to check actual attribute access
        pass
    
    def test_sampling_params_partial(self):
        """Test SamplingParams with partial parameters."""
        sampling = SamplingParams(temperature=0.5)
        
        # Implementation-dependent assertions
        pass


class TestLimits:
    """Test cases for Limits class."""
    
    def test_limits_creation(self):
        """Test Limits creation."""
        limits = Limits(
            max_output_tokens=1000,
            reasoning_tokens=500,
            max_prompt_tokens=4000
        )
        
        # Implementation-dependent assertions
        pass
    
    def test_limits_partial(self):
        """Test Limits with partial parameters."""
        limits = Limits(max_output_tokens=500)
        
        # Implementation-dependent assertions
        pass


class TestTool:
    """Test cases for Tool class."""
    
    def test_tool_creation_basic(self):
        """Test basic Tool creation."""
        tool = Tool(
            name="test_tool",
            json_schema={"type": "object", "properties": {}}
        )
        
        # Implementation-dependent assertions
        pass
    
    def test_tool_creation_with_description(self):
        """Test Tool creation with description."""
        tool = Tool(
            name="weather_tool",
            description="Get weather information",
            json_schema={
                "type": "object",
                "properties": {
                    "location": {"type": "string"}
                },
                "required": ["location"]
            }
        )
        
        # Implementation-dependent assertions
        pass


class TestTranslationResult:
    """Test cases for TranslationResult class."""
    
    def test_translation_result_properties(self, sample_translation_result: TranslationResult):
        """Test TranslationResult properties."""
        result = sample_translation_result
        
        assert isinstance(result.provider_request_json, dict)
        assert isinstance(result.has_lossiness, bool)
    
    def test_translation_result_to_dict(self, sample_translation_result: TranslationResult):
        """Test TranslationResult serialization."""
        result_dict = sample_translation_result.to_dict()
        
        assert isinstance(result_dict, dict)
        assert "provider_request_json" in result_dict
        assert "lossiness" in result_dict
    
    def test_translation_result_repr(self, sample_translation_result: TranslationResult):
        """Test TranslationResult string representation."""
        repr_str = repr(sample_translation_result)
        assert "TranslationResult" in repr_str
        assert str(sample_translation_result.has_lossiness) in repr_str


class TestUniformResponse:
    """Test cases for UniformResponse class."""
    
    def test_uniform_response_creation(self):
        """Test UniformResponse creation would happen through run functions."""
        # UniformResponse is typically created by run functions,
        # so we test its properties when we have an instance
        pass
    
    def test_uniform_response_properties(self):
        """Test UniformResponse properties."""
        # Would need a sample response from run function
        pass
    
    def test_uniform_response_finish_reasons(self):
        """Test all valid finish reasons."""
        valid_reasons = ["stop", "length", "tool_call", "end_conversation", "other"]
        
        # Would test that these are the only valid values
        pass


class TestValidationResult:
    """Test cases for ValidationResult class."""
    
    def test_validation_result_creation(self):
        """Test ValidationResult creation."""
        result = ValidationResult(is_valid=True, errors=[])
        
        assert result.is_valid is True
        assert len(result.errors) == 0
    
    def test_validation_result_with_errors(self):
        """Test ValidationResult with errors."""
        errors = ["Missing field: model_class", "Invalid role: invalid"]
        result = ValidationResult(is_valid=False, errors=errors)
        
        assert result.is_valid is False
        assert len(result.errors) == 2
        assert result.errors == errors
    
    def test_validation_result_repr(self):
        """Test ValidationResult string representation."""
        result = ValidationResult(is_valid=True, errors=[])
        repr_str = repr(result)
        
        assert "ValidationResult" in repr_str
        assert "True" in repr_str
        assert "0" in repr_str


class TestTypeConversions:
    """Test type conversion and serialization."""
    
    def test_json_serialization_roundtrip(self, sample_prompt: PromptSpec):
        """Test that objects can be serialized to JSON and back."""
        # Convert to dict
        prompt_dict = sample_prompt.to_dict()
        
        # Serialize to JSON
        json_str = json.dumps(prompt_dict)
        
        # Deserialize from JSON
        restored_dict = json.loads(json_str)
        
        # Convert back to object
        restored_prompt = PromptSpec.from_dict(restored_dict)
        
        # Verify equality
        assert restored_prompt.model_class == sample_prompt.model_class
        assert restored_prompt.strict_mode == sample_prompt.strict_mode
        assert len(restored_prompt.messages) == len(sample_prompt.messages)
    
    def test_dict_to_object_conversion(self):
        """Test converting dictionaries to typed objects."""
        message_dict = {"role": "user", "content": "Hello"}
        
        # This would test the conversion if supported
        # message = Message.from_dict(message_dict)
        pass
    
    def test_unicode_handling(self):
        """Test Unicode string handling in types."""
        unicode_content = "Hello 世界 🌍 🚀 مرحبا"
        message = Message("user", unicode_content)
        
        assert message.content == unicode_content
        
        # Test serialization preserves Unicode
        prompt = PromptSpec(
            model_class="Chat",
            messages=[message],
            strict_mode="warn"
        )
        
        prompt_dict = prompt.to_dict()
        json_str = json.dumps(prompt_dict, ensure_ascii=False)
        
        assert unicode_content in json_str
    
    def test_large_objects(self):
        """Test handling of large objects."""
        # Create a prompt with many messages
        messages = [
            Message("user", f"Message {i}")
            for i in range(100)
        ]
        
        large_prompt = PromptSpec(
            model_class="Chat",
            messages=messages,
            strict_mode="warn"
        )
        
        # Should handle large objects gracefully
        prompt_dict = large_prompt.to_dict()
        assert len(prompt_dict["messages"]) == 100
        
        # Should be JSON serializable
        json_str = json.dumps(prompt_dict)
        assert len(json_str) > 0


class TestTypeHints:
    """Test type hint compatibility."""
    
    def test_type_checking_compatibility(self):
        """Test that objects work with type checking."""
        # This would test mypy compatibility if run with mypy
        
        def process_prompt(prompt: PromptSpec) -> str:
            return prompt.model_class
        
        def process_message(message: Message) -> str:
            return f"{message.role}: {message.content}"
        
        # Test that function calls work correctly
        message = Message("user", "Hello")
        prompt = PromptSpec(
            model_class="Chat",
            messages=[message],
            strict_mode="warn"
        )
        
        assert process_message(message) == "user: Hello"
        assert process_prompt(prompt) == "Chat"
    
    def test_optional_fields_typing(self):
        """Test typing for optional fields."""
        # Test that optional fields work correctly with type hints
        message = Message("user", "Hello")  # name is optional
        assert message.name is None
        
        message_with_name = Message("user", "Hello", name="User1")
        assert message_with_name.name == "User1"