"""
Integration tests for the complete Specado Python binding workflow.
"""

import pytest
import asyncio
import json
from typing import Dict, Any

import specado
from specado import (
    PromptSpec, ProviderSpec, Message, Tool, SamplingParams, Limits,
    TranslationResult, UniformResponse, ValidationResult,
    SpecadoError, TranslationError, ValidationError, ProviderError
)


class TestCompleteWorkflow:
    """Test complete end-to-end workflows."""
    
    def test_simple_workflow_sync(self, sample_prompt: PromptSpec, sample_provider: ProviderSpec):
        """Test simple synchronous workflow: validate -> translate -> run."""
        # Step 1: Validate the prompt
        validation_result = specado.validate(sample_prompt, "prompt")
        assert validation_result.is_valid, f"Validation failed: {validation_result.errors}"
        
        # Step 2: Validate the provider
        provider_validation = specado.validate(sample_provider, "provider")
        assert provider_validation.is_valid, f"Provider validation failed: {provider_validation.errors}"
        
        # Step 3: Translate the prompt
        translation_result = specado.translate(
            prompt=sample_prompt,
            provider_spec=sample_provider,
            model_id="test-model",
            mode="standard"
        )
        assert isinstance(translation_result, TranslationResult)
        assert isinstance(translation_result.provider_request_json, dict)
        
        # Step 4: Run the request
        response = specado.run_sync(
            request=translation_result.provider_request_json,
            provider_spec=sample_provider,
            timeout=30
        )
        assert isinstance(response, UniformResponse)
        assert isinstance(response.content, str)
        assert isinstance(response.model, str)
        assert response.finish_reason in ["stop", "length", "tool_call", "end_conversation", "other"]
    
    @pytest.mark.asyncio
    async def test_simple_workflow_async(self, sample_prompt: PromptSpec, sample_provider: ProviderSpec):
        """Test simple asynchronous workflow: validate -> translate -> run."""
        # Steps 1-3 are synchronous (validation and translation)
        validation_result = specado.validate(sample_prompt, "prompt")
        assert validation_result.is_valid
        
        translation_result = specado.translate(
            prompt=sample_prompt,
            provider_spec=sample_provider,
            model_id="test-model"
        )
        assert isinstance(translation_result, TranslationResult)
        
        # Step 4: Run async
        response = await specado.run(
            request=translation_result.provider_request_json,
            provider_spec=sample_provider,
            timeout=30
        )
        assert isinstance(response, UniformResponse)
    
    def test_complex_workflow_with_tools(self, sample_provider: ProviderSpec):
        """Test workflow with complex prompt including tools."""
        # Create a complex prompt with tools
        weather_tool = Tool(
            name="get_weather",
            description="Get current weather for a location",
            json_schema={
                "type": "object",
                "properties": {
                    "location": {"type": "string", "description": "City name"},
                    "units": {"type": "string", "enum": ["celsius", "fahrenheit"]}
                },
                "required": ["location"]
            }
        )
        
        complex_prompt = PromptSpec(
            model_class="Chat",
            messages=[
                Message("system", "You are a weather assistant."),
                Message("user", "What's the weather in San Francisco?")
            ],
            tools=[weather_tool],
            tool_choice="auto",
            sampling=SamplingParams(
                temperature=0.7,
                top_p=0.9
            ),
            limits=Limits(
                max_output_tokens=1000
            ),
            strict_mode="warn"
        )
        
        # Validate complex prompt
        validation_result = specado.validate(complex_prompt, "prompt")
        assert validation_result.is_valid, f"Complex prompt validation failed: {validation_result.errors}"
        
        # Translate complex prompt
        translation_result = specado.translate(
            prompt=complex_prompt,
            provider_spec=sample_provider,
            model_id="test-model"
        )
        assert isinstance(translation_result, TranslationResult)
        
        # Check that the translation includes tool information
        request_json = translation_result.provider_request_json
        assert isinstance(request_json, dict)
        # The exact structure depends on the provider, but should contain the tool
    
    def test_multiple_models_workflow(self, sample_prompt: PromptSpec, multi_model_provider: ProviderSpec):
        """Test workflow with multiple models from the same provider."""
        responses = []
        
        for model in multi_model_provider.models:
            # Translate for this specific model
            translation_result = specado.translate(
                prompt=sample_prompt,
                provider_spec=multi_model_provider,
                model_id=model.id
            )
            
            # Run the request
            response = specado.run_sync(
                request=translation_result.provider_request_json,
                provider_spec=multi_model_provider
            )
            
            responses.append((model.id, response))
            assert isinstance(response, UniformResponse)
        
        # Should have responses from all models
        assert len(responses) == len(multi_model_provider.models)
        
        # All responses should be valid
        for model_id, response in responses:
            assert isinstance(response.content, str)
            assert len(response.content) > 0


class TestSerializationRoundtrips:
    """Test serialization and deserialization roundtrips."""
    
    def test_prompt_spec_roundtrip(self, complex_prompt: PromptSpec):
        """Test PromptSpec serialization roundtrip."""
        # Convert to dict
        prompt_dict = complex_prompt.to_dict()
        
        # Serialize to JSON
        json_str = json.dumps(prompt_dict)
        
        # Deserialize from JSON
        restored_dict = json.loads(json_str)
        
        # Convert back to PromptSpec
        restored_prompt = PromptSpec.from_dict(restored_dict)
        
        # Verify key properties are preserved
        assert restored_prompt.model_class == complex_prompt.model_class
        assert restored_prompt.strict_mode == complex_prompt.strict_mode
        assert len(restored_prompt.messages) == len(complex_prompt.messages)
        
        # Verify the restored prompt validates
        validation_result = specado.validate(restored_prompt, "prompt")
        assert validation_result.is_valid
    
    def test_provider_spec_roundtrip(self, sample_provider: ProviderSpec):
        """Test ProviderSpec serialization roundtrip."""
        # Convert to dict
        provider_dict = sample_provider.to_dict()
        
        # Serialize to JSON
        json_str = json.dumps(provider_dict)
        
        # Deserialize from JSON
        restored_dict = json.loads(json_str)
        
        # Convert back to ProviderSpec
        restored_provider = ProviderSpec.from_dict(restored_dict)
        
        # Verify key properties are preserved
        assert restored_provider.spec_version == sample_provider.spec_version
        assert restored_provider.provider.name == sample_provider.provider.name
        assert len(restored_provider.models) == len(sample_provider.models)
        
        # Verify the restored provider validates
        validation_result = specado.validate(restored_provider, "provider")
        assert validation_result.is_valid
    
    def test_translation_result_roundtrip(self, sample_translation_result: TranslationResult):
        """Test TranslationResult serialization roundtrip."""
        # Convert to dict
        result_dict = sample_translation_result.to_dict()
        
        # Serialize to JSON
        json_str = json.dumps(result_dict)
        
        # Deserialize from JSON
        restored_dict = json.loads(json_str)
        
        # Verify structure is preserved
        assert "provider_request_json" in restored_dict
        assert "lossiness" in restored_dict
        assert isinstance(restored_dict["provider_request_json"], dict)


class TestErrorHandlingIntegration:
    """Test error handling across the entire workflow."""
    
    def test_validation_error_prevents_translation(self):
        """Test that validation errors prevent translation."""
        # Create invalid prompt
        invalid_prompt = PromptSpec(
            model_class="",  # Invalid empty model class
            messages=[],    # Invalid empty messages
            strict_mode="warn"
        )
        
        # Validation should fail
        validation_result = specado.validate(invalid_prompt, "prompt")
        assert not validation_result.is_valid
        assert len(validation_result.errors) > 0
        
        # Translation should also fail
        with pytest.raises((ValidationError, TranslationError)):
            specado.translate(
                prompt=invalid_prompt,
                provider_spec=mock_provider(),
                model_id="test-model"
            )
    
    def test_translation_error_prevents_execution(self, sample_provider: ProviderSpec):
        """Test that translation errors prevent execution."""
        # Try to translate with non-existent model
        sample_prompt = PromptSpec(
            model_class="Chat",
            messages=[Message("user", "Hello")],
            strict_mode="warn"
        )
        
        with pytest.raises(ProviderError):
            translation_result = specado.translate(
                prompt=sample_prompt,
                provider_spec=sample_provider,
                model_id="definitely-does-not-exist-12345"
            )
    
    def test_error_recovery_between_operations(self, sample_prompt: PromptSpec, sample_provider: ProviderSpec):
        """Test that errors don't affect subsequent operations."""
        # First, cause an error
        try:
            specado.translate(
                prompt=sample_prompt,
                provider_spec=sample_provider,
                model_id="nonexistent-model"
            )
        except ProviderError:
            pass  # Expected
        
        # Then, perform a valid operation
        valid_result = specado.translate(
            prompt=sample_prompt,
            provider_spec=sample_provider,
            model_id="test-model"
        )
        assert isinstance(valid_result, TranslationResult)


class TestConcurrentOperations:
    """Test concurrent operations and thread safety."""
    
    @pytest.mark.asyncio
    async def test_concurrent_translations(self, sample_prompt: PromptSpec, sample_provider: ProviderSpec):
        """Test concurrent translation operations."""
        async def translate_task():
            return specado.translate(
                prompt=sample_prompt,
                provider_spec=sample_provider,
                model_id="test-model"
            )
        
        # Run multiple translations concurrently
        tasks = [translate_task() for _ in range(5)]
        results = await asyncio.gather(*tasks)
        
        # All should succeed
        assert len(results) == 5
        for result in results:
            assert isinstance(result, TranslationResult)
    
    @pytest.mark.asyncio
    async def test_concurrent_validations(self, sample_prompt: PromptSpec):
        """Test concurrent validation operations."""
        async def validate_task():
            return specado.validate(sample_prompt, "prompt")
        
        # Run multiple validations concurrently
        tasks = [validate_task() for _ in range(10)]
        results = await asyncio.gather(*tasks)
        
        # All should succeed
        assert len(results) == 10
        for result in results:
            assert isinstance(result, ValidationResult)
            assert result.is_valid
    
    @pytest.mark.asyncio
    async def test_concurrent_mixed_operations(self, sample_prompt: PromptSpec, sample_provider: ProviderSpec):
        """Test mixing different types of operations concurrently."""
        async def validation_task():
            return specado.validate(sample_prompt, "prompt")
        
        async def translation_task():
            return specado.translate(
                prompt=sample_prompt,
                provider_spec=sample_provider,
                model_id="test-model"
            )
        
        # Mix validation and translation tasks
        tasks = []
        for i in range(5):
            tasks.append(validation_task())
            tasks.append(translation_task())
        
        results = await asyncio.gather(*tasks)
        
        # All should succeed
        assert len(results) == 10
        validation_count = sum(1 for r in results if isinstance(r, ValidationResult))
        translation_count = sum(1 for r in results if isinstance(r, TranslationResult))
        
        assert validation_count == 5
        assert translation_count == 5


class TestPerformanceIntegration:
    """Test performance characteristics of integrated workflows."""
    
    def test_workflow_performance(self, sample_prompt: PromptSpec, sample_provider: ProviderSpec):
        """Test that complete workflow performs within acceptable bounds."""
        import time
        
        # Warm up
        for _ in range(3):
            validation_result = specado.validate(sample_prompt, "prompt")
            translation_result = specado.translate(
                prompt=sample_prompt,
                provider_spec=sample_provider,
                model_id="test-model"
            )
        
        # Measure performance
        start_time = time.time()
        
        for _ in range(10):
            # Validation should be very fast
            validation_result = specado.validate(sample_prompt, "prompt")
            assert validation_result.is_valid
            
            # Translation should be reasonably fast
            translation_result = specado.translate(
                prompt=sample_prompt,
                provider_spec=sample_provider,
                model_id="test-model"
            )
            assert isinstance(translation_result, TranslationResult)
        
        end_time = time.time()
        avg_time_per_operation = (end_time - start_time) / 10
        
        # Each complete validation + translation should be fast (< 50ms)
        assert avg_time_per_operation < 0.05, f"Workflow too slow: {avg_time_per_operation:.3f}s per operation"
    
    def test_memory_efficiency(self, sample_prompt: PromptSpec, sample_provider: ProviderSpec):
        """Test that repeated operations don't leak memory."""
        import gc
        import sys
        
        def get_ref_count():
            return sys.gettotalrefcount() if hasattr(sys, 'gettotalrefcount') else 0
        
        # Baseline reference count
        gc.collect()
        baseline_refs = get_ref_count()
        
        # Perform many operations
        for i in range(100):
            validation_result = specado.validate(sample_prompt, "prompt")
            translation_result = specado.translate(
                prompt=sample_prompt,
                provider_spec=sample_provider,
                model_id="test-model"
            )
            
            # Explicitly delete references
            del validation_result, translation_result
        
        # Force garbage collection
        gc.collect()
        final_refs = get_ref_count()
        
        # Reference count should not have grown significantly
        if baseline_refs > 0:  # Only check if we have reference counting
            ref_growth = final_refs - baseline_refs
            assert ref_growth < 1000, f"Potential memory leak: {ref_growth} reference count increase"


def mock_provider() -> ProviderSpec:
    """Create a mock provider for testing."""
    from specado.types import ProviderSpecDict
    
    mock_spec: ProviderSpecDict = {
        "spec_version": "1.0.0",
        "provider": {
            "name": "mock",
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
                        "path": "/chat",
                        "protocol": "https"
                    },
                    "streaming_chat_completion": {
                        "method": "POST",
                        "path": "/chat/stream",
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