"""
Tests for the validate function and validation functionality.
"""

import pytest
from typing import Dict, Any

import specado
from specado import (
    PromptSpec, ProviderSpec, Message, ValidationResult,
    ValidationError
)


class TestValidateFunction:
    """Test cases for the validate function."""
    
    def test_validate_valid_prompt_spec(self, sample_prompt: PromptSpec):
        """Test validation of a valid PromptSpec."""
        result = specado.validate(sample_prompt, "prompt")
        
        assert isinstance(result, ValidationResult)
        assert result.is_valid is True
        assert len(result.errors) == 0
    
    def test_validate_valid_provider_spec(self, sample_provider: ProviderSpec):
        """Test validation of a valid ProviderSpec."""
        result = specado.validate(sample_provider, "provider")
        
        assert isinstance(result, ValidationResult)
        assert result.is_valid is True
        assert len(result.errors) == 0
    
    def test_validate_prompt_spec_dict(self, sample_prompt_dict: Dict[str, Any]):
        """Test validation of PromptSpec as dictionary."""
        result = specado.validate(sample_prompt_dict, "prompt")
        
        assert isinstance(result, ValidationResult)
        assert result.is_valid is True
        assert len(result.errors) == 0
    
    def test_validate_provider_spec_dict(self, sample_provider_dict: Dict[str, Any]):
        """Test validation of ProviderSpec as dictionary."""
        result = specado.validate(sample_provider_dict, "provider")
        
        assert isinstance(result, ValidationResult)
        assert result.is_valid is True
        assert len(result.errors) == 0
    
    def test_validate_invalid_schema_type(self, sample_prompt: PromptSpec):
        """Test validation with invalid schema type."""
        with pytest.raises(ValidationError):
            specado.validate(sample_prompt, "invalid_type")  # type: ignore
    
    def test_validate_wrong_spec_type(self, sample_prompt: PromptSpec):
        """Test validation with wrong spec type."""
        # Trying to validate a prompt as a provider should fail
        result = specado.validate(sample_prompt, "provider")
        
        assert isinstance(result, ValidationResult)
        assert result.is_valid is False
        assert len(result.errors) > 0


class TestValidatePromptSpec:
    """Test validation of PromptSpec objects and dictionaries."""
    
    def test_validate_prompt_missing_model_class(self):
        """Test validation of prompt missing model_class."""
        invalid_prompt = {
            "messages": [{"role": "user", "content": "Hello"}],
            "strict_mode": "warn"
        }
        
        result = specado.validate(invalid_prompt, "prompt")
        
        assert result.is_valid is False
        assert any("model_class" in error for error in result.errors)
    
    def test_validate_prompt_missing_messages(self):
        """Test validation of prompt missing messages."""
        invalid_prompt = {
            "model_class": "Chat",
            "strict_mode": "warn"
        }
        
        result = specado.validate(invalid_prompt, "prompt")
        
        assert result.is_valid is False
        assert any("messages" in error for error in result.errors)
    
    def test_validate_prompt_empty_messages(self):
        """Test validation of prompt with empty messages array."""
        invalid_prompt = {
            "model_class": "Chat",
            "messages": [],
            "strict_mode": "warn"
        }
        
        result = specado.validate(invalid_prompt, "prompt")
        
        assert result.is_valid is False
        assert any("empty" in error.lower() for error in result.errors)
    
    def test_validate_prompt_invalid_message_role(self):
        """Test validation of prompt with invalid message role."""
        invalid_prompt = {
            "model_class": "Chat",
            "messages": [
                {"role": "invalid_role", "content": "Hello"}
            ],
            "strict_mode": "warn"
        }
        
        result = specado.validate(invalid_prompt, "prompt")
        
        assert result.is_valid is False
        assert any("role" in error for error in result.errors)
    
    def test_validate_prompt_missing_message_content(self):
        """Test validation of prompt with message missing content."""
        invalid_prompt = {
            "model_class": "Chat",
            "messages": [
                {"role": "user"}  # Missing content
            ],
            "strict_mode": "warn"
        }
        
        result = specado.validate(invalid_prompt, "prompt")
        
        assert result.is_valid is False
        assert any("content" in error for error in result.errors)
    
    def test_validate_prompt_empty_message_content(self):
        """Test validation of prompt with empty message content."""
        invalid_prompt = {
            "model_class": "Chat",
            "messages": [
                {"role": "user", "content": ""}  # Empty content
            ],
            "strict_mode": "warn"
        }
        
        result = specado.validate(invalid_prompt, "prompt")
        
        assert result.is_valid is False
        assert any("content" in error and "empty" in error.lower() for error in result.errors)
    
    def test_validate_prompt_invalid_strict_mode(self):
        """Test validation of prompt with invalid strict_mode."""
        invalid_prompt = {
            "model_class": "Chat",
            "messages": [{"role": "user", "content": "Hello"}],
            "strict_mode": "invalid_mode"
        }
        
        result = specado.validate(invalid_prompt, "prompt")
        
        assert result.is_valid is False
        assert any("strict_mode" in error for error in result.errors)
    
    def test_validate_prompt_invalid_sampling_params(self):
        """Test validation of prompt with invalid sampling parameters."""
        invalid_prompt = {
            "model_class": "Chat",
            "messages": [{"role": "user", "content": "Hello"}],
            "strict_mode": "warn",
            "sampling": {
                "temperature": 3.0,  # Out of range (should be 0-2)
                "top_p": 1.5  # Out of range (should be 0-1)
            }
        }
        
        result = specado.validate(invalid_prompt, "prompt")
        
        assert result.is_valid is False
        assert any("temperature" in error for error in result.errors)
        assert any("top_p" in error for error in result.errors)
    
    def test_validate_prompt_invalid_limits(self):
        """Test validation of prompt with invalid limits."""
        invalid_prompt = {
            "model_class": "Chat",
            "messages": [{"role": "user", "content": "Hello"}],
            "strict_mode": "warn",
            "limits": {
                "max_output_tokens": 0  # Should be > 0
            }
        }
        
        result = specado.validate(invalid_prompt, "prompt")
        
        assert result.is_valid is False
        assert any("max_output_tokens" in error for error in result.errors)
    
    def test_validate_prompt_invalid_tools(self):
        """Test validation of prompt with invalid tools."""
        invalid_prompt = {
            "model_class": "Chat",
            "messages": [{"role": "user", "content": "Hello"}],
            "strict_mode": "warn",
            "tools": [
                {
                    # Missing name and json_schema
                    "description": "A tool"
                }
            ]
        }
        
        result = specado.validate(invalid_prompt, "prompt")
        
        assert result.is_valid is False
        assert any("name" in error for error in result.errors)
        assert any("json_schema" in error for error in result.errors)


class TestValidateProviderSpec:
    """Test validation of ProviderSpec objects and dictionaries."""
    
    def test_validate_provider_missing_spec_version(self):
        """Test validation of provider missing spec_version."""
        invalid_provider = {
            "provider": {
                "name": "test",
                "base_url": "https://api.test.com",
                "headers": {}
            },
            "models": [{"id": "test-model", "family": "test"}]
        }
        
        result = specado.validate(invalid_provider, "provider")
        
        assert result.is_valid is False
        assert any("spec_version" in error for error in result.errors)
    
    def test_validate_provider_missing_provider_info(self):
        """Test validation of provider missing provider info."""
        invalid_provider = {
            "spec_version": "1.0.0",
            "models": [{"id": "test-model", "family": "test"}]
        }
        
        result = specado.validate(invalid_provider, "provider")
        
        assert result.is_valid is False
        assert any("provider" in error for error in result.errors)
    
    def test_validate_provider_missing_models(self):
        """Test validation of provider missing models."""
        invalid_provider = {
            "spec_version": "1.0.0",
            "provider": {
                "name": "test",
                "base_url": "https://api.test.com",
                "headers": {}
            }
        }
        
        result = specado.validate(invalid_provider, "provider")
        
        assert result.is_valid is False
        assert any("models" in error for error in result.errors)
    
    def test_validate_provider_empty_models(self):
        """Test validation of provider with empty models array."""
        invalid_provider = {
            "spec_version": "1.0.0",
            "provider": {
                "name": "test",
                "base_url": "https://api.test.com",
                "headers": {}
            },
            "models": []
        }
        
        result = specado.validate(invalid_provider, "provider")
        
        assert result.is_valid is False
        assert any("empty" in error.lower() for error in result.errors)
    
    def test_validate_provider_invalid_provider_info(self):
        """Test validation of provider with invalid provider info."""
        invalid_provider = {
            "spec_version": "1.0.0",
            "provider": {
                "name": "",  # Empty name
                "base_url": "",  # Empty URL
                "headers": {}
            },
            "models": [{"id": "test-model", "family": "test"}]
        }
        
        result = specado.validate(invalid_provider, "provider")
        
        assert result.is_valid is False
        assert any("name" in error and "empty" in error.lower() for error in result.errors)
        assert any("base_url" in error and "empty" in error.lower() for error in result.errors)
    
    def test_validate_provider_invalid_model_spec(self):
        """Test validation of provider with invalid model specification."""
        invalid_provider = {
            "spec_version": "1.0.0",
            "provider": {
                "name": "test",
                "base_url": "https://api.test.com",
                "headers": {}
            },
            "models": [
                {
                    # Missing required fields
                    "family": "test"
                }
            ]
        }
        
        result = specado.validate(invalid_provider, "provider")
        
        assert result.is_valid is False
        assert any("id" in error for error in result.errors)


class TestValidateEdgeCases:
    """Test edge cases and error conditions for validate function."""
    
    def test_validate_non_dict_input(self):
        """Test validation with non-dictionary input."""
        with pytest.raises((ValidationError, TypeError)):
            specado.validate("not a dict", "prompt")  # type: ignore
    
    def test_validate_none_input(self):
        """Test validation with None input."""
        with pytest.raises((ValidationError, TypeError)):
            specado.validate(None, "prompt")  # type: ignore
    
    def test_validate_empty_dict(self):
        """Test validation with empty dictionary."""
        result = specado.validate({}, "prompt")
        
        assert result.is_valid is False
        assert len(result.errors) > 0
    
    def test_validate_nested_validation_errors(self):
        """Test that nested validation errors are properly reported."""
        complex_invalid_prompt = {
            "model_class": "",  # Invalid
            "messages": [
                {"role": "invalid", "content": ""},  # Multiple issues
                {"role": "user"}  # Missing content
            ],
            "strict_mode": "invalid",  # Invalid
            "sampling": {
                "temperature": -1,  # Invalid
                "top_p": 2  # Invalid
            },
            "tools": [
                {"description": "No name or schema"}  # Invalid
            ]
        }
        
        result = specado.validate(complex_invalid_prompt, "prompt")
        
        assert result.is_valid is False
        assert len(result.errors) >= 5  # Should have multiple errors
    
    def test_validate_very_large_spec(self):
        """Test validation with very large specification."""
        large_prompt = {
            "model_class": "Chat",
            "messages": [
                {"role": "user", "content": f"Message {i}"}
                for i in range(1000)  # 1000 messages
            ],
            "strict_mode": "warn"
        }
        
        result = specado.validate(large_prompt, "prompt")
        
        # Should handle large specs gracefully
        assert isinstance(result, ValidationResult)
    
    def test_validate_unicode_content(self):
        """Test validation with Unicode content."""
        unicode_prompt = {
            "model_class": "Chat",
            "messages": [
                {"role": "user", "content": "Hello 世界 🌍 🚀 مرحبا"}
            ],
            "strict_mode": "warn"
        }
        
        result = specado.validate(unicode_prompt, "prompt")
        
        assert result.is_valid is True


class TestValidatePerformance:
    """Performance tests for validate function."""
    
    def test_validate_performance_baseline(self, sample_prompt: PromptSpec):
        """Test baseline performance for validation."""
        import time
        
        start_time = time.time()
        
        for _ in range(100):
            result = specado.validate(sample_prompt, "prompt")
            assert result.is_valid is True
        
        end_time = time.time()
        avg_time = (end_time - start_time) / 100
        
        # Validation should be very fast (< 10ms per operation)
        assert avg_time < 0.01, f"Validation too slow: {avg_time:.3f}s per operation"
    
    @pytest.mark.benchmark
    def test_validate_benchmark(self, benchmark, sample_prompt: PromptSpec):
        """Benchmark the validate function."""
        result = benchmark(specado.validate, sample_prompt, "prompt")
        assert result.is_valid is True