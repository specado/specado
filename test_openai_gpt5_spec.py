#!/usr/bin/env python3
"""Test the OpenAI GPT-5 provider specification with the translation engine."""

import json
import yaml
from pathlib import Path

def test_gpt5_spec_structure():
    """Test that the GPT-5 spec has all required fields and correct structure."""
    
    spec_file = Path("providers/openai/gpt-5.yaml")
    
    with open(spec_file, 'r') as f:
        spec = yaml.safe_load(f)
    
    # Test provider info
    assert spec['spec_version'] == "1.1.0"
    assert spec['provider']['name'] == "openai"
    assert spec['provider']['base_url'] == "https://api.openai.com"
    assert spec['provider']['auth']['type'] == "bearer"
    assert "${ENV:OPENAI_API_KEY}" in spec['provider']['auth']['value_template']
    
    # Test models
    assert len(spec['models']) == 2
    gpt5 = spec['models'][0]
    gpt5_preview = spec['models'][1]
    
    # Test GPT-5 model configuration
    assert gpt5['id'] == "gpt-5"
    assert "gpt-5-turbo" in gpt5['aliases']
    assert gpt5['family'] == "gpt-5"
    
    # Test endpoints
    assert gpt5['endpoints']['chat_completion']['method'] == "POST"
    assert gpt5['endpoints']['chat_completion']['path'] == "/v1/responses"
    assert gpt5['endpoints']['streaming_chat_completion']['headers']['Accept'] == "text/event-stream"
    
    # Test input modes
    assert gpt5['input_modes']['messages'] == True
    assert gpt5['input_modes']['images'] == True
    assert gpt5['input_modes']['single_text'] == False
    
    # Test tooling
    assert gpt5['tooling']['tools_supported'] == True
    assert gpt5['tooling']['parallel_tool_calls_default'] == True
    assert gpt5['tooling']['strict_tools_support'] == True
    assert "auto" in gpt5['tooling']['tool_choice_modes']
    assert "specific" in gpt5['tooling']['tool_choice_modes']
    
    # Test JSON output
    assert gpt5['json_output']['native_param'] == True
    assert gpt5['json_output']['strategy'] == "json_schema"
    
    # Test parameters
    params = gpt5['parameters']
    assert params['temperature']['minimum'] == 0
    assert params['temperature']['maximum'] == 2
    assert params['max_tokens']['maximum'] == 128000
    assert 'response_format' in params
    assert 'tools' in params
    assert 'tool_choice' in params
    
    # Test constraints
    constraints = gpt5['constraints']
    assert constraints['system_prompt_location'] == "message_role"
    assert constraints['forbid_unknown_top_level_fields'] == True
    assert ["n", "stream"] in constraints['mutually_exclusive']
    assert constraints['limits']['max_tool_schema_bytes'] == 100000
    
    # Test mappings
    mappings = gpt5['mappings']
    assert "$.messages" in mappings['paths']
    assert mappings['paths']["$.messages"] == "$.messages"
    assert mappings['paths']["$.limits.max_output_tokens"] == "$.max_tokens"
    assert mappings['paths']["$.sampling.temperature"] == "$.temperature"
    
    # Test response normalization
    normalization = gpt5['response_normalization']
    assert normalization['sync']['content_path'] == "choices[0].message.content"
    assert normalization['sync']['finish_reason_path'] == "choices[0].finish_reason"
    assert normalization['stream']['protocol'] == "sse"
    assert normalization['stream']['event_selector']['type_path'] == "object"
    
    # Test GPT-5 preview model
    assert gpt5_preview['id'] == "gpt-5-preview"
    assert "gpt-5-turbo-preview" in gpt5_preview['aliases']
    assert gpt5_preview['family'] == "gpt-5"
    
    print("✅ All tests passed for OpenAI GPT-5 specification")
    return True

def test_translation_compatibility():
    """Test that the spec would work with the translation engine."""
    
    spec_file = Path("providers/openai/gpt-5.yaml")
    
    with open(spec_file, 'r') as f:
        spec = yaml.safe_load(f)
    
    gpt5 = spec['models'][0]
    
    # Simulate a simple translation scenario
    uniform_prompt = {
        "messages": [
            {"role": "system", "content": "You are a helpful assistant"},
            {"role": "user", "content": "Hello!"}
        ],
        "sampling": {
            "temperature": 0.7,
            "top_p": 0.9
        },
        "limits": {
            "max_output_tokens": 1000
        }
    }
    
    # Apply mappings (simplified)
    paths = gpt5['mappings']['paths']
    
    # Check that critical mappings exist
    assert "$.messages" in paths
    assert "$.sampling.temperature" in paths
    assert "$.sampling.top_p" in paths
    assert "$.limits.max_output_tokens" in paths
    
    # Check mapped values
    assert paths["$.sampling.temperature"] == "$.temperature"
    assert paths["$.sampling.top_p"] == "$.top_p"
    assert paths["$.limits.max_output_tokens"] == "$.max_tokens"
    
    print("✅ Translation compatibility tests passed")
    return True

if __name__ == "__main__":
    test_gpt5_spec_structure()
    test_translation_compatibility()
    print("\n🎉 All tests passed for OpenAI GPT-5 provider specification!")