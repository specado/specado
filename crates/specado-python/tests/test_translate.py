"""
Tests for the translate function and related functionality.
"""

import pytest
import json
from typing import Dict, Any

import specado
from specado import (
    PromptSpec, ProviderSpec, Message, TranslationResult,
    SpecadoError, TranslationError, ValidationError, ProviderError
)


class TestTranslateFunction:
    """Test cases for the translate function."""
    
    def test_translate_basic_prompt(self, sample_prompt: PromptSpec, sample_provider: ProviderSpec):
        """Test basic translation functionality."""
        result = specado.translate(
            prompt=sample_prompt,
            provider_spec=sample_provider,
            model_id="test-model",
            mode="standard"
        )
        
        assert isinstance(result, TranslationResult)
        assert isinstance(result.provider_request_json, dict)
        assert isinstance(result.has_lossiness, bool)
    
    def test_translate_with_strict_mode(self, sample_prompt: PromptSpec, sample_provider: ProviderSpec):
        """Test translation with strict mode."""
        result = specado.translate(
            prompt=sample_prompt,
            provider_spec=sample_provider,
            model_id="test-model",
            mode="strict"
        )
        
        assert isinstance(result, TranslationResult)
        assert result.provider_request_json is not None
    
    def test_translate_default_mode(self, sample_prompt: PromptSpec, sample_provider: ProviderSpec):
        """Test translation with default mode."""
        result = specado.translate(
            prompt=sample_prompt,
            provider_spec=sample_provider,
            model_id="test-model"
        )
        
        assert isinstance(result, TranslationResult)
    
    def test_translate_invalid_model_id(self, sample_prompt: PromptSpec, sample_provider: ProviderSpec):
        """Test translation with invalid model ID."""
        with pytest.raises(ProviderError):
            specado.translate(
                prompt=sample_prompt,
                provider_spec=sample_provider,
                model_id="nonexistent-model"
            )
    
    def test_translate_invalid_mode(self, sample_prompt: PromptSpec, sample_provider: ProviderSpec):
        """Test translation with invalid mode."""
        with pytest.raises(ValidationError):
            specado.translate(
                prompt=sample_prompt,
                provider_spec=sample_provider,
                model_id="test-model",
                mode="invalid-mode"
            )
    
    def test_translate_empty_messages(self, sample_provider: ProviderSpec):
        """Test translation with empty messages."""
        prompt = PromptSpec(
            model_class="Chat",
            messages=[],  # Empty messages should fail
            strict_mode="warn"
        )
        
        with pytest.raises((ValidationError, TranslationError)):
            specado.translate(
                prompt=prompt,
                provider_spec=sample_provider,
                model_id="test-model"
            )
    
    def test_translate_complex_prompt(self, complex_prompt: PromptSpec, sample_provider: ProviderSpec):
        """Test translation with complex prompt containing tools and sampling params."""
        result = specado.translate(
            prompt=complex_prompt,
            provider_spec=sample_provider,
            model_id="test-model"
        )
        
        assert isinstance(result, TranslationResult)
        assert result.provider_request_json is not None
        
        # Check if the provider request contains expected fields
        request = result.provider_request_json
        assert "model" in request or "messages" in request  # At least one should be present
    
    def test_translate_result_serialization(self, sample_prompt: PromptSpec, sample_provider: ProviderSpec):
        """Test that translation result can be serialized to dict."""
        result = specado.translate(
            prompt=sample_prompt,
            provider_spec=sample_provider,
            model_id="test-model"
        )
        
        result_dict = result.to_dict()
        assert isinstance(result_dict, dict)
        assert "provider_request_json" in result_dict
        assert "lossiness" in result_dict
        
        # Ensure it's JSON serializable
        json_str = json.dumps(result_dict)
        assert len(json_str) > 0
    
    def test_translate_with_system_message(self, sample_provider: ProviderSpec):
        """Test translation with system message."""
        prompt = PromptSpec(
            model_class="Chat",
            messages=[
                Message("system", "You are a helpful assistant."),
                Message("user", "Hello, world!")
            ],
            strict_mode="warn"
        )
        
        result = specado.translate(
            prompt=prompt,
            provider_spec=sample_provider,
            model_id="test-model"
        )
        
        assert isinstance(result, TranslationResult)
    
    def test_translate_lossiness_detection(self, sample_provider: ProviderSpec):
        """Test that lossiness is properly detected and reported."""
        # Create a prompt that might cause lossiness (unsupported features)
        prompt = PromptSpec(
            model_class="Chat",
            messages=[Message("user", "Test message")],
            sampling=specado.SamplingParams(
                temperature=0.7,
                top_p=0.9,
                frequency_penalty=0.5  # This might not be supported by all providers
            ),
            strict_mode="warn"
        )
        
        result = specado.translate(
            prompt=prompt,
            provider_spec=sample_provider,
            model_id="test-model"
        )
        
        # Result should exist regardless of lossiness
        assert isinstance(result, TranslationResult)
        assert isinstance(result.has_lossiness, bool)


class TestTranslateEdgeCases:
    """Test edge cases and error conditions for translate function."""
    
    def test_translate_very_long_message(self, sample_provider: ProviderSpec):
        """Test translation with very long message content."""
        long_content = "A" * 10000  # 10k character message
        
        prompt = PromptSpec(
            model_class="Chat",
            messages=[Message("user", long_content)],
            strict_mode="warn"
        )
        
        result = specado.translate(
            prompt=prompt,
            provider_spec=sample_provider,
            model_id="test-model"
        )
        
        assert isinstance(result, TranslationResult)
    
    def test_translate_unicode_content(self, sample_provider: ProviderSpec):
        """Test translation with Unicode content."""
        unicode_content = "Hello 世界 🌍 🚀 مرحبا"
        
        prompt = PromptSpec(
            model_class="Chat",
            messages=[Message("user", unicode_content)],
            strict_mode="warn"
        )
        
        result = specado.translate(
            prompt=prompt,
            provider_spec=sample_provider,
            model_id="test-model"
        )
        
        assert isinstance(result, TranslationResult)
        # Ensure Unicode is preserved in the provider request
        request_json = json.dumps(result.provider_request_json)
        assert unicode_content in request_json or "世界" in request_json
    
    def test_translate_empty_model_id(self, sample_prompt: PromptSpec, sample_provider: ProviderSpec):
        """Test translation with empty model ID."""
        with pytest.raises((ValidationError, ProviderError)):
            specado.translate(
                prompt=sample_prompt,
                provider_spec=sample_provider,
                model_id=""
            )
    
    def test_translate_none_inputs(self, sample_provider: ProviderSpec):
        """Test translation with None inputs."""
        with pytest.raises((TypeError, ValidationError)):
            specado.translate(
                prompt=None,  # type: ignore
                provider_spec=sample_provider,
                model_id="test-model"
            )
        
        with pytest.raises((TypeError, ValidationError)):
            specado.translate(
                prompt=PromptSpec(
                    model_class="Chat",
                    messages=[Message("user", "Hello")],
                    strict_mode="warn"
                ),
                provider_spec=None,  # type: ignore
                model_id="test-model"
            )
    
    def test_translate_special_characters_in_model_id(self, sample_prompt: PromptSpec, sample_provider: ProviderSpec):
        """Test translation with special characters in model ID."""
        special_model_ids = [
            "model-with-dashes",
            "model_with_underscores", 
            "model.with.dots",
            "model:with:colons"
        ]
        
        for model_id in special_model_ids:
            try:
                result = specado.translate(
                    prompt=sample_prompt,
                    provider_spec=sample_provider,
                    model_id=model_id
                )
                # If it succeeds, it should return a valid result
                assert isinstance(result, TranslationResult)
            except ProviderError:
                # It's acceptable for some model IDs to not be found
                pass


class TestTranslateIntegration:
    """Integration tests for translate with various prompt and provider combinations."""
    
    def test_translate_chain_to_run(self, sample_prompt: PromptSpec, sample_provider: ProviderSpec):
        """Test that translate result can be chained to run function."""
        translation_result = specado.translate(
            prompt=sample_prompt,
            provider_spec=sample_provider,
            model_id="test-model"
        )
        
        # Create a provider request from the translation
        provider_request = specado.create_provider_request(
            translation_result, sample_provider
        )
        
        assert isinstance(provider_request, dict)
        assert len(provider_request) > 0
    
    def test_translate_multiple_models(self, sample_prompt: PromptSpec, multi_model_provider: ProviderSpec):
        """Test translation with provider that has multiple models."""
        for model in multi_model_provider.models:
            result = specado.translate(
                prompt=sample_prompt,
                provider_spec=multi_model_provider,
                model_id=model.id
            )
            assert isinstance(result, TranslationResult)
    
    def test_translate_all_message_roles(self, sample_provider: ProviderSpec):
        """Test translation with all supported message roles."""
        prompt = PromptSpec(
            model_class="Chat",
            messages=[
                Message("system", "You are a helpful assistant."),
                Message("user", "Hello!"),
                Message("assistant", "Hi there! How can I help you?"),
                Message("user", "What's the weather like?")
            ],
            strict_mode="warn"
        )
        
        result = specado.translate(
            prompt=prompt,
            provider_spec=sample_provider,
            model_id="test-model"
        )
        
        assert isinstance(result, TranslationResult)


class TestTranslatePerformance:
    """Performance tests for translate function."""
    
    def test_translate_performance_baseline(self, sample_prompt: PromptSpec, sample_provider: ProviderSpec):
        """Test baseline performance for translation."""
        import time
        
        start_time = time.time()
        
        for _ in range(10):
            result = specado.translate(
                prompt=sample_prompt,
                provider_spec=sample_provider,
                model_id="test-model"
            )
            assert isinstance(result, TranslationResult)
        
        end_time = time.time()
        avg_time = (end_time - start_time) / 10
        
        # Translation should be reasonably fast (< 100ms per operation)
        assert avg_time < 0.1, f"Translation too slow: {avg_time:.3f}s per operation"
    
    @pytest.mark.benchmark
    def test_translate_benchmark(self, benchmark, sample_prompt: PromptSpec, sample_provider: ProviderSpec):
        """Benchmark the translate function."""
        result = benchmark(
            specado.translate,
            sample_prompt,
            sample_provider,
            "test-model"
        )
        assert isinstance(result, TranslationResult)