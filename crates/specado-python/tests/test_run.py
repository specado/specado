"""
Tests for the run functions (both sync and async) and related functionality.
"""

import pytest
import asyncio
import time
from typing import Dict, Any

import specado
from specado import (
    PromptSpec, ProviderSpec, UniformResponse, TranslationResult,
    ProviderError, TimeoutError, SpecadoError
)


class TestRunSync:
    """Test cases for the synchronous run function."""
    
    def test_run_sync_basic(self, sample_provider_request: Dict[str, Any], sample_provider: ProviderSpec):
        """Test basic synchronous run functionality."""
        response = specado.run_sync(
            request=sample_provider_request,
            provider_spec=sample_provider,
            timeout=30
        )
        
        assert isinstance(response, UniformResponse)
        assert isinstance(response.model, str)
        assert isinstance(response.content, str)
        assert isinstance(response.finish_reason, str)
        assert response.finish_reason in ["stop", "length", "tool_call", "end_conversation", "other"]
    
    def test_run_sync_no_timeout(self, sample_provider_request: Dict[str, Any], sample_provider: ProviderSpec):
        """Test synchronous run without timeout."""
        response = specado.run_sync(
            request=sample_provider_request,
            provider_spec=sample_provider
        )
        
        assert isinstance(response, UniformResponse)
    
    def test_run_sync_with_timeout(self, sample_provider_request: Dict[str, Any], sample_provider: ProviderSpec):
        """Test synchronous run with timeout."""
        response = specado.run_sync(
            request=sample_provider_request,
            provider_spec=sample_provider,
            timeout=5
        )
        
        assert isinstance(response, UniformResponse)
    
    def test_run_sync_invalid_request(self, sample_provider: ProviderSpec):
        """Test synchronous run with invalid request."""
        invalid_request = {"invalid": "request"}
        
        with pytest.raises((ProviderError, SpecadoError)):
            specado.run_sync(
                request=invalid_request,
                provider_spec=sample_provider
            )
    
    def test_run_sync_empty_request(self, sample_provider: ProviderSpec):
        """Test synchronous run with empty request."""
        with pytest.raises((ProviderError, SpecadoError)):
            specado.run_sync(
                request={},
                provider_spec=sample_provider
            )
    
    def test_run_sync_response_serialization(self, sample_provider_request: Dict[str, Any], sample_provider: ProviderSpec):
        """Test that run response can be serialized."""
        response = specado.run_sync(
            request=sample_provider_request,
            provider_spec=sample_provider
        )
        
        response_dict = response.to_dict()
        assert isinstance(response_dict, dict)
        assert "model" in response_dict
        assert "content" in response_dict
        assert "finish_reason" in response_dict
        
        # Should be JSON serializable
        import json
        json_str = json.dumps(response_dict)
        assert len(json_str) > 0


class TestRunAsync:
    """Test cases for the asynchronous run function."""
    
    @pytest.mark.asyncio
    async def test_run_async_basic(self, sample_provider_request: Dict[str, Any], sample_provider: ProviderSpec):
        """Test basic asynchronous run functionality."""
        response = await specado.run(
            request=sample_provider_request,
            provider_spec=sample_provider,
            timeout=30
        )
        
        assert isinstance(response, UniformResponse)
        assert isinstance(response.model, str)
        assert isinstance(response.content, str)
        assert isinstance(response.finish_reason, str)
        assert response.finish_reason in ["stop", "length", "tool_call", "end_conversation", "other"]
    
    @pytest.mark.asyncio
    async def test_run_async_no_timeout(self, sample_provider_request: Dict[str, Any], sample_provider: ProviderSpec):
        """Test asynchronous run without timeout."""
        response = await specado.run(
            request=sample_provider_request,
            provider_spec=sample_provider
        )
        
        assert isinstance(response, UniformResponse)
    
    @pytest.mark.asyncio
    async def test_run_async_with_timeout(self, sample_provider_request: Dict[str, Any], sample_provider: ProviderSpec):
        """Test asynchronous run with timeout."""
        response = await specado.run(
            request=sample_provider_request,
            provider_spec=sample_provider,
            timeout=5
        )
        
        assert isinstance(response, UniformResponse)
    
    @pytest.mark.asyncio
    async def test_run_async_concurrent_requests(self, sample_provider_request: Dict[str, Any], sample_provider: ProviderSpec):
        """Test multiple concurrent async requests."""
        tasks = [
            specado.run(sample_provider_request, sample_provider, timeout=30)
            for _ in range(5)
        ]
        
        responses = await asyncio.gather(*tasks)
        
        assert len(responses) == 5
        for response in responses:
            assert isinstance(response, UniformResponse)
    
    @pytest.mark.asyncio
    async def test_run_async_invalid_request(self, sample_provider: ProviderSpec):
        """Test asynchronous run with invalid request."""
        invalid_request = {"invalid": "request"}
        
        with pytest.raises((ProviderError, SpecadoError)):
            await specado.run(
                request=invalid_request,
                provider_spec=sample_provider
            )
    
    @pytest.mark.asyncio
    async def test_run_async_timeout_behavior(self, sample_provider_request: Dict[str, Any], sample_provider: ProviderSpec):
        """Test async timeout behavior."""
        # Test with very short timeout (this might timeout depending on the provider)
        try:
            response = await specado.run(
                request=sample_provider_request,
                provider_spec=sample_provider,
                timeout=1  # 1 second timeout
            )
            # If it succeeds, that's fine
            assert isinstance(response, UniformResponse)
        except TimeoutError:
            # If it times out, that's also expected
            pass


class TestCreateProviderRequest:
    """Test cases for the create_provider_request helper function."""
    
    def test_create_provider_request_basic(self, sample_translation_result: TranslationResult, sample_provider: ProviderSpec):
        """Test basic provider request creation."""
        request = specado.create_provider_request(
            translation_result=sample_translation_result,
            provider_spec=sample_provider
        )
        
        assert isinstance(request, dict)
        assert len(request) > 0
    
    def test_create_provider_request_chain(self, sample_prompt: PromptSpec, sample_provider: ProviderSpec):
        """Test chaining translate -> create_provider_request -> run."""
        # First translate
        translation_result = specado.translate(
            prompt=sample_prompt,
            provider_spec=sample_provider,
            model_id="test-model"
        )
        
        # Create provider request
        provider_request = specado.create_provider_request(
            translation_result=translation_result,
            provider_spec=sample_provider
        )
        
        # Run the request
        response = specado.run_sync(
            request=provider_request,
            provider_spec=sample_provider
        )
        
        assert isinstance(response, UniformResponse)


class TestRunIntegration:
    """Integration tests for run functions with real workflows."""
    
    def test_complete_workflow_sync(self, sample_prompt: PromptSpec, sample_provider: ProviderSpec):
        """Test complete workflow: prompt -> translate -> run (sync)."""
        # Step 1: Translate
        translation_result = specado.translate(
            prompt=sample_prompt,
            provider_spec=sample_provider,
            model_id="test-model"
        )
        
        # Step 2: Run
        response = specado.run_sync(
            request=translation_result.provider_request_json,
            provider_spec=sample_provider
        )
        
        assert isinstance(response, UniformResponse)
        assert len(response.content) > 0
    
    @pytest.mark.asyncio
    async def test_complete_workflow_async(self, sample_prompt: PromptSpec, sample_provider: ProviderSpec):
        """Test complete workflow: prompt -> translate -> run (async)."""
        # Step 1: Translate
        translation_result = specado.translate(
            prompt=sample_prompt,
            provider_spec=sample_provider,
            model_id="test-model"
        )
        
        # Step 2: Run async
        response = await specado.run(
            request=translation_result.provider_request_json,
            provider_spec=sample_provider
        )
        
        assert isinstance(response, UniformResponse)
        assert len(response.content) > 0
    
    def test_multiple_models_workflow(self, sample_prompt: PromptSpec, multi_model_provider: ProviderSpec):
        """Test workflow with multiple models."""
        responses = []
        
        for model in multi_model_provider.models:
            # Translate for each model
            translation_result = specado.translate(
                prompt=sample_prompt,
                provider_spec=multi_model_provider,
                model_id=model.id
            )
            
            # Run for each model
            response = specado.run_sync(
                request=translation_result.provider_request_json,
                provider_spec=multi_model_provider
            )
            
            responses.append(response)
            assert isinstance(response, UniformResponse)
        
        assert len(responses) == len(multi_model_provider.models)
    
    @pytest.mark.asyncio
    async def test_concurrent_different_models(self, sample_prompt: PromptSpec, multi_model_provider: ProviderSpec):
        """Test concurrent requests to different models."""
        async def run_model(model_id: str) -> UniformResponse:
            translation_result = specado.translate(
                prompt=sample_prompt,
                provider_spec=multi_model_provider,
                model_id=model_id
            )
            
            return await specado.run(
                request=translation_result.provider_request_json,
                provider_spec=multi_model_provider
            )
        
        tasks = [
            run_model(model.id)
            for model in multi_model_provider.models[:3]  # Limit to first 3 models
        ]
        
        responses = await asyncio.gather(*tasks)
        
        assert len(responses) == min(3, len(multi_model_provider.models))
        for response in responses:
            assert isinstance(response, UniformResponse)


class TestRunErrorHandling:
    """Test error handling in run functions."""
    
    def test_run_sync_network_error(self, sample_provider: ProviderSpec):
        """Test handling of network errors in sync run."""
        # Create a request that might cause network issues
        network_breaking_request = {
            "model": "test-model",
            "messages": [{"role": "user", "content": "Hello"}],
            "invalid_endpoint": True
        }
        
        with pytest.raises((ProviderError, SpecadoError)):
            specado.run_sync(
                request=network_breaking_request,
                provider_spec=sample_provider,
                timeout=5
            )
    
    @pytest.mark.asyncio
    async def test_run_async_network_error(self, sample_provider: ProviderSpec):
        """Test handling of network errors in async run."""
        network_breaking_request = {
            "model": "test-model", 
            "messages": [{"role": "user", "content": "Hello"}],
            "invalid_endpoint": True
        }
        
        with pytest.raises((ProviderError, SpecadoError)):
            await specado.run(
                request=network_breaking_request,
                provider_spec=sample_provider,
                timeout=5
            )
    
    def test_run_sync_malformed_request(self, sample_provider: ProviderSpec):
        """Test handling of malformed requests."""
        malformed_requests = [
            None,
            "not a dict",
            [],
            {"completely": {"wrong": {"structure": True}}},
        ]
        
        for request in malformed_requests:
            with pytest.raises((ProviderError, SpecadoError, TypeError)):
                specado.run_sync(
                    request=request,  # type: ignore
                    provider_spec=sample_provider
                )


class TestRunPerformance:
    """Performance tests for run functions."""
    
    def test_run_sync_performance_baseline(self, sample_provider_request: Dict[str, Any], sample_provider: ProviderSpec):
        """Test baseline performance for sync run."""
        start_time = time.time()
        
        # Run multiple requests
        for _ in range(5):
            response = specado.run_sync(
                request=sample_provider_request,
                provider_spec=sample_provider,
                timeout=30
            )
            assert isinstance(response, UniformResponse)
        
        end_time = time.time()
        avg_time = (end_time - start_time) / 5
        
        # Each request should complete in reasonable time (< 5s)
        assert avg_time < 5.0, f"Run too slow: {avg_time:.3f}s per operation"
    
    @pytest.mark.asyncio
    async def test_run_async_performance_baseline(self, sample_provider_request: Dict[str, Any], sample_provider: ProviderSpec):
        """Test baseline performance for async run."""
        start_time = time.time()
        
        # Run multiple concurrent requests
        tasks = [
            specado.run(sample_provider_request, sample_provider, timeout=30)
            for _ in range(5)
        ]
        
        responses = await asyncio.gather(*tasks)
        
        end_time = time.time()
        total_time = end_time - start_time
        
        assert len(responses) == 5
        for response in responses:
            assert isinstance(response, UniformResponse)
        
        # Concurrent requests should be faster than sequential
        assert total_time < 10.0, f"Concurrent run too slow: {total_time:.3f}s total"
    
    @pytest.mark.benchmark
    def test_run_sync_benchmark(self, benchmark, sample_provider_request: Dict[str, Any], sample_provider: ProviderSpec):
        """Benchmark the sync run function."""
        result = benchmark(
            specado.run_sync,
            sample_provider_request,
            sample_provider,
            30
        )
        assert isinstance(result, UniformResponse)