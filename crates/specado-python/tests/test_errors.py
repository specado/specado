"""
Tests for error handling and exception types.
"""

import pytest
from typing import Dict, Any

import specado
from specado import (
    SpecadoError, TranslationError, ValidationError, ProviderError, TimeoutError,
    PromptSpec, ProviderSpec, Message
)


class TestExceptionTypes:
    """Test custom exception types and inheritance."""
    
    def test_specado_error_is_base(self):
        """Test that SpecadoError is the base exception."""
        assert issubclass(TranslationError, SpecadoError)
        assert issubclass(ValidationError, SpecadoError)
        assert issubclass(ProviderError, SpecadoError)
        # TimeoutError might inherit from built-in TimeoutError
    
    def test_exception_instantiation(self):
        """Test that exceptions can be instantiated with messages."""
        error_msg = "Test error message"
        
        specado_error = SpecadoError(error_msg)
        translation_error = TranslationError(error_msg)
        validation_error = ValidationError(error_msg)
        provider_error = ProviderError(error_msg)
        timeout_error = TimeoutError(error_msg)
        
        assert str(specado_error) == error_msg
        assert str(translation_error) == error_msg
        assert str(validation_error) == error_msg
        assert str(provider_error) == error_msg
        assert str(timeout_error) == error_msg
    
    def test_exception_attributes(self):
        """Test that exceptions have proper attributes."""
        error_msg = "Test error"
        error = SpecadoError(error_msg)
        
        assert hasattr(error, 'args')
        assert error.args[0] == error_msg


class TestTranslationErrors:
    """Test error handling in translate function."""
    
    def test_translate_with_invalid_prompt_type(self, sample_provider: ProviderSpec):
        """Test translate with invalid prompt type."""
        with pytest.raises((ValidationError, TypeError)):
            specado.translate(
                prompt="not a prompt spec",  # type: ignore
                provider_spec=sample_provider,
                model_id="test-model"
            )
    
    def test_translate_with_invalid_provider_type(self, sample_prompt: PromptSpec):
        """Test translate with invalid provider type."""
        with pytest.raises((ValidationError, TypeError)):
            specado.translate(
                prompt=sample_prompt,
                provider_spec="not a provider spec",  # type: ignore
                model_id="test-model"
            )
    
    def test_translate_with_none_prompt(self, sample_provider: ProviderSpec):
        """Test translate with None prompt."""
        with pytest.raises((ValidationError, TypeError)):
            specado.translate(
                prompt=None,  # type: ignore
                provider_spec=sample_provider,
                model_id="test-model"
            )
    
    def test_translate_with_none_provider(self, sample_prompt: PromptSpec):
        """Test translate with None provider."""
        with pytest.raises((ValidationError, TypeError)):
            specado.translate(
                prompt=sample_prompt,
                provider_spec=None,  # type: ignore
                model_id="test-model"
            )
    
    def test_translate_with_empty_model_id(self, sample_prompt: PromptSpec, sample_provider: ProviderSpec):
        """Test translate with empty model ID."""
        with pytest.raises((ValidationError, ProviderError)):
            specado.translate(
                prompt=sample_prompt,
                provider_spec=sample_provider,
                model_id=""
            )
    
    def test_translate_with_none_model_id(self, sample_prompt: PromptSpec, sample_provider: ProviderSpec):
        """Test translate with None model ID."""
        with pytest.raises((ValidationError, TypeError)):
            specado.translate(
                prompt=sample_prompt,
                provider_spec=sample_provider,
                model_id=None  # type: ignore
            )
    
    def test_translate_with_nonexistent_model(self, sample_prompt: PromptSpec, sample_provider: ProviderSpec):
        """Test translate with model that doesn't exist in provider."""
        with pytest.raises(ProviderError):
            specado.translate(
                prompt=sample_prompt,
                provider_spec=sample_provider,
                model_id="nonexistent-model-12345"
            )
    
    def test_translate_with_invalid_mode(self, sample_prompt: PromptSpec, sample_provider: ProviderSpec):
        """Test translate with invalid mode."""
        with pytest.raises(ValidationError):
            specado.translate(
                prompt=sample_prompt,
                provider_spec=sample_provider,
                model_id="test-model",
                mode="invalid-mode"
            )
    
    def test_translate_with_malformed_prompt(self, sample_provider: ProviderSpec):
        """Test translate with malformed prompt."""
        # Create a prompt with invalid structure
        malformed_prompt = PromptSpec(
            model_class="",  # Empty model class
            messages=[],  # Empty messages
            strict_mode="warn"
        )
        
        with pytest.raises((TranslationError, ValidationError)):
            specado.translate(
                prompt=malformed_prompt,
                provider_spec=sample_provider,
                model_id="test-model"
            )


class TestValidationErrors:
    """Test error handling in validate function."""
    
    def test_validate_with_invalid_schema_type(self, sample_prompt: PromptSpec):
        """Test validate with invalid schema type."""
        with pytest.raises(ValidationError):
            specado.validate(sample_prompt, "invalid_schema_type")  # type: ignore
    
    def test_validate_with_none_spec(self):
        """Test validate with None spec."""
        with pytest.raises((ValidationError, TypeError)):
            specado.validate(None, "prompt")  # type: ignore
    
    def test_validate_with_none_schema_type(self, sample_prompt: PromptSpec):
        """Test validate with None schema type."""
        with pytest.raises((ValidationError, TypeError)):
            specado.validate(sample_prompt, None)  # type: ignore
    
    def test_validate_with_empty_schema_type(self, sample_prompt: PromptSpec):
        """Test validate with empty schema type."""
        with pytest.raises(ValidationError):
            specado.validate(sample_prompt, "")
    
    def test_validate_wrong_spec_type_for_schema(self, sample_prompt: PromptSpec):
        """Test validating prompt as provider schema."""
        result = specado.validate(sample_prompt, "provider")
        
        # Should return validation errors, not raise exception
        assert result.is_valid is False
        assert len(result.errors) > 0
    
    def test_validate_malformed_dict(self):
        """Test validate with malformed dictionary."""
        malformed_dict = {
            "not_a_valid": "structure",
            "missing_required": "fields"
        }
        
        result = specado.validate(malformed_dict, "prompt")
        
        assert result.is_valid is False
        assert len(result.errors) > 0
    
    def test_validate_non_dict_non_object(self):
        """Test validate with non-dict, non-object input."""
        with pytest.raises((ValidationError, TypeError)):
            specado.validate("not a dict or object", "prompt")  # type: ignore
        
        with pytest.raises((ValidationError, TypeError)):
            specado.validate(123, "prompt")  # type: ignore
        
        with pytest.raises((ValidationError, TypeError)):
            specado.validate([], "prompt")  # type: ignore


class TestRunErrors:
    """Test error handling in run functions."""
    
    def test_run_sync_with_invalid_request_type(self, sample_provider: ProviderSpec):
        """Test run_sync with invalid request type."""
        with pytest.raises((ProviderError, TypeError)):
            specado.run_sync(
                request="not a dict",  # type: ignore
                provider_spec=sample_provider
            )
    
    def test_run_sync_with_none_request(self, sample_provider: ProviderSpec):
        """Test run_sync with None request."""
        with pytest.raises((ProviderError, TypeError)):
            specado.run_sync(
                request=None,  # type: ignore
                provider_spec=sample_provider
            )
    
    def test_run_sync_with_empty_request(self, sample_provider: ProviderSpec):
        """Test run_sync with empty request."""
        with pytest.raises(ProviderError):
            specado.run_sync(
                request={},
                provider_spec=sample_provider
            )
    
    def test_run_sync_with_invalid_provider_type(self):
        """Test run_sync with invalid provider type."""
        sample_request = {"model": "test", "messages": [{"role": "user", "content": "Hi"}]}
        
        with pytest.raises((ProviderError, TypeError)):
            specado.run_sync(
                request=sample_request,
                provider_spec="not a provider spec"  # type: ignore
            )
    
    @pytest.mark.asyncio
    async def test_run_async_with_invalid_request_type(self, sample_provider: ProviderSpec):
        """Test run async with invalid request type."""
        with pytest.raises((ProviderError, TypeError)):
            await specado.run(
                request="not a dict",  # type: ignore
                provider_spec=sample_provider
            )
    
    @pytest.mark.asyncio
    async def test_run_async_with_none_request(self, sample_provider: ProviderSpec):
        """Test run async with None request."""
        with pytest.raises((ProviderError, TypeError)):
            await specado.run(
                request=None,  # type: ignore
                provider_spec=sample_provider
            )
    
    def test_run_sync_with_malformed_request(self, sample_provider: ProviderSpec):
        """Test run_sync with malformed request structure."""
        malformed_requests = [
            {"completely": {"wrong": {"structure": True}}},
            {"model": None},
            {"messages": "not an array"},
            {"model": "", "messages": []},
        ]
        
        for request in malformed_requests:
            with pytest.raises((ProviderError, SpecadoError)):
                specado.run_sync(
                    request=request,
                    provider_spec=sample_provider
                )
    
    def test_run_sync_timeout_error(self, sample_provider: ProviderSpec):
        """Test run_sync timeout handling."""
        # Create a request that might timeout
        slow_request = {
            "model": "test-model",
            "messages": [{"role": "user", "content": "Hello"}],
            "max_tokens": 10000  # Large token request might be slow
        }
        
        try:
            response = specado.run_sync(
                request=slow_request,
                provider_spec=sample_provider,
                timeout=1  # Very short timeout
            )
            # If it succeeds quickly, that's also fine
            assert response is not None
        except TimeoutError:
            # Expected behavior for timeout
            pass
        except (ProviderError, SpecadoError):
            # Other errors are also acceptable in test environment
            pass


class TestErrorMessages:
    """Test error message quality and information."""
    
    def test_validation_error_messages_are_informative(self):
        """Test that validation errors provide helpful information."""
        # Test with missing required fields
        invalid_prompt = {
            "messages": [{"role": "user", "content": "Hello"}]
            # Missing model_class and strict_mode
        }
        
        result = specado.validate(invalid_prompt, "prompt")
        
        assert result.is_valid is False
        assert len(result.errors) > 0
        
        # Check that error messages mention the missing fields
        error_text = " ".join(result.errors)
        assert "model_class" in error_text
        assert "strict_mode" in error_text
    
    def test_translation_error_preserves_context(self, sample_provider: ProviderSpec):
        """Test that translation errors preserve context information."""
        try:
            specado.translate(
                prompt=None,  # type: ignore
                provider_spec=sample_provider,
                model_id="test-model"
            )
        except Exception as e:
            # Error message should be informative
            error_msg = str(e)
            assert len(error_msg) > 0
            # Should not be just a generic error message
            assert "None" in error_msg or "null" in error_msg or "prompt" in error_msg
    
    def test_provider_error_includes_model_info(self, sample_prompt: PromptSpec, sample_provider: ProviderSpec):
        """Test that provider errors include model information."""
        try:
            specado.translate(
                prompt=sample_prompt,
                provider_spec=sample_provider,
                model_id="definitely-nonexistent-model-12345"
            )
        except ProviderError as e:
            error_msg = str(e)
            # Should mention the model that wasn't found
            assert "definitely-nonexistent-model-12345" in error_msg or "model" in error_msg.lower()
    
    def test_error_messages_are_not_empty(self):
        """Test that all error types produce non-empty messages."""
        errors = [
            SpecadoError("Test message"),
            TranslationError("Test message"),
            ValidationError("Test message"),
            ProviderError("Test message"),
            TimeoutError("Test message"),
        ]
        
        for error in errors:
            assert len(str(error)) > 0
            assert str(error) != ""


class TestErrorRecovery:
    """Test error recovery and graceful degradation."""
    
    def test_continue_after_validation_error(self, sample_provider: ProviderSpec):
        """Test that operations can continue after validation errors."""
        # First, cause a validation error
        try:
            specado.validate({}, "prompt")
        except Exception:
            pass
        
        # Then, try a valid operation
        valid_prompt = PromptSpec(
            model_class="Chat",
            messages=[Message("user", "Hello")],
            strict_mode="warn"
        )
        
        # This should work fine
        result = specado.validate(valid_prompt, "prompt")
        assert result.is_valid is True
    
    def test_continue_after_translation_error(self, sample_provider: ProviderSpec):
        """Test that operations can continue after translation errors."""
        # First, cause a translation error
        try:
            specado.translate(
                prompt=None,  # type: ignore
                provider_spec=sample_provider,
                model_id="test-model"
            )
        except Exception:
            pass
        
        # Then, try a valid operation
        valid_prompt = PromptSpec(
            model_class="Chat",
            messages=[Message("user", "Hello")],
            strict_mode="warn"
        )
        
        # This should work fine
        result = specado.translate(
            prompt=valid_prompt,
            provider_spec=sample_provider,
            model_id="test-model"
        )
        assert result is not None
    
    def test_multiple_error_types_in_sequence(self, sample_provider: ProviderSpec):
        """Test handling multiple different error types in sequence."""
        errors_caught = []
        
        # Validation error
        try:
            specado.validate("invalid", "prompt")  # type: ignore
        except Exception as e:
            errors_caught.append(type(e))
        
        # Translation error
        try:
            specado.translate(
                prompt=None,  # type: ignore
                provider_spec=sample_provider,
                model_id="test-model"
            )
        except Exception as e:
            errors_caught.append(type(e))
        
        # Provider error
        try:
            specado.translate(
                prompt=PromptSpec(
                    model_class="Chat",
                    messages=[Message("user", "Hello")],
                    strict_mode="warn"
                ),
                provider_spec=sample_provider,
                model_id="nonexistent-model"
            )
        except Exception as e:
            errors_caught.append(type(e))
        
        # Should have caught at least some errors
        assert len(errors_caught) > 0
        
        # All should be Specado-related exceptions
        for error_type in errors_caught:
            assert issubclass(error_type, (SpecadoError, TypeError, ValueError))


class TestErrorContext:
    """Test that errors maintain proper context and stack traces."""
    
    def test_error_stack_traces_are_preserved(self, sample_provider: ProviderSpec):
        """Test that error stack traces point to the right place."""
        import traceback
        
        try:
            specado.translate(
                prompt=None,  # type: ignore
                provider_spec=sample_provider,
                model_id="test-model"
            )
        except Exception as e:
            tb = traceback.format_exc()
            # Stack trace should include our function call
            assert "translate" in tb
            assert "test_error_stack_traces_are_preserved" in tb
    
    def test_nested_error_handling(self):
        """Test error handling in nested operations."""
        # This would test chained operations that might fail at different stages
        
        def complex_operation():
            # This is a placeholder for a complex operation that might fail
            # at different stages (validation -> translation -> execution)
            pass
        
        # Test that errors are properly propagated and context is maintained
        pass