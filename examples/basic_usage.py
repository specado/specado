#!/usr/bin/env python3
"""
Basic usage example for Specado Python bindings
"""

import json
import os
from pathlib import Path

# Import specado module
try:
    import specado
    from specado import PromptSpec, ProviderSpec, Message, Sampling, Limits
    from specado import validate, translate
except ImportError:
    print("❌ Please install specado first: maturin develop")
    exit(1)

def main():
    """Demonstrate basic usage of Specado Python bindings"""
    
    print("🚀 Specado Python Bindings Example")
    
    # Get version information
    print(f"📦 Version: {specado.__version__}")
    
    # Define a prompt with proper structure
    prompt = PromptSpec(
        model_class="Chat",
        messages=[
            Message(
                role="user",
                content="Hello, world! Can you help me understand how LLM APIs work?"
            )
        ],
        sampling=Sampling(
            temperature=0.7
        ),
        limits=Limits(
            max_output_tokens=1000
        ),
        strict_mode="standard"
    )
    
    print("\n📝 Prompt created")
    print(f"  Messages: {len(prompt.messages)}")
    print(f"  Temperature: {prompt.sampling.temperature}")
    print(f"  Max tokens: {prompt.limits.max_output_tokens}")
    
    # Validate the prompt
    print("\n🔍 Validating prompt...")
    validation_result = validate(prompt, "prompt")
    
    if validation_result.valid:
        print("✅ Prompt is valid!")
    else:
        print(f"❌ Validation errors: {validation_result.errors}")
        return
    
    # Load a real provider specification from golden corpus
    provider_spec_path = Path(__file__).parent.parent / "providers" / "openai" / "gpt-5.json"
    
    try:
        with open(provider_spec_path, 'r') as f:
            provider_spec_dict = json.load(f)
        print(f"\n📋 Loaded provider spec: {provider_spec_dict['provider']['name']}")
        
        # Create ProviderSpec from dict
        provider_spec = ProviderSpec.from_dict(provider_spec_dict)
        
    except (FileNotFoundError, json.JSONDecodeError, AttributeError) as e:
        print(f"❌ Failed to load provider spec: {e}")
        print("Using valid provider spec structure from golden corpus...")
        
        # Fallback to a valid provider spec structure
        provider_spec_dict = {
            "spec_version": "1.0.0",
            "provider": {
                "name": "openai",
                "base_url": "https://api.openai.com/v1",
                "headers": {
                    "Authorization": "Bearer ${OPENAI_API_KEY}"
                }
            },
            "models": [
                {
                    "id": "gpt-4",
                    "aliases": ["gpt-4-turbo"],
                    "family": "gpt",
                    "endpoints": {
                        "chat_completion": {
                            "method": "POST",
                            "path": "/chat/completions",
                            "protocol": "http"
                        },
                        "streaming_chat_completion": {
                            "method": "POST",
                            "path": "/chat/completions",
                            "protocol": "sse"
                        }
                    },
                    "input_modes": {
                        "messages": True,
                        "single_text": False,
                        "images": False
                    },
                    "tooling": {
                        "tools_supported": True,
                        "parallel_tool_calls_default": True,
                        "can_disable_parallel_tool_calls": True,
                        "disable_switch": {
                            "parallel_tool_calls": False
                        }
                    },
                    "json_output": {
                        "native_param": True,
                        "strategy": "response_format"
                    },
                    "parameters": {
                        "temperature": {
                            "type": "number",
                            "minimum": 0.0,
                            "maximum": 2.0,
                            "default": 1.0
                        },
                        "max_tokens": {
                            "type": "integer",
                            "minimum": 1,
                            "maximum": 128000
                        }
                    },
                    "constraints": {
                        "system_prompt_location": "first_message",
                        "forbid_unknown_top_level_fields": True,
                        "mutually_exclusive": [["temperature", "top_p"]],
                        "resolution_preferences": ["temperature"],
                        "limits": {
                            "max_tool_schema_bytes": 16384,
                            "max_system_prompt_bytes": 32768
                        }
                    },
                    "mappings": {
                        "paths": {
                            "$.limits.max_output_tokens": "$.max_tokens",
                            "$.sampling.temperature": "$.temperature"
                        },
                        "flags": {}
                    },
                    "response_normalization": {
                        "sync": {
                            "content_path": "$.choices[0].message.content",
                            "finish_reason_path": "$.choices[0].finish_reason",
                            "finish_reason_map": {
                                "stop": "stop",
                                "length": "length"
                            }
                        },
                        "stream": {
                            "protocol": "sse",
                            "event_selector": {
                                "type_path": "$.choices[0].delta",
                                "routes": []
                            }
                        }
                    }
                }
            ]
        }
        
        # Create ProviderSpec from dict
        provider_spec = ProviderSpec.from_dict(provider_spec_dict)
    
    # Validate the provider spec
    print("\n🔍 Validating provider spec...")
    provider_validation = validate(provider_spec, "provider")
    
    if provider_validation.valid:
        print("✅ Provider spec is valid!")
    else:
        print(f"❌ Provider validation errors: {provider_validation.errors}")
        return
    
    # Translate the prompt
    print("\n🔄 Translating prompt to provider format...")
    model_id = provider_spec.models[0].id if hasattr(provider_spec, 'models') else "gpt-4"
    
    try:
        translation_result = translate(
            prompt=prompt,
            provider_spec=provider_spec,
            model_id=model_id,
            mode="standard"
        )
        
        print("✅ Translation successful!")
        
        # Display the full TranslationResult including lossiness
        print("\n📊 Translation Result:")
        
        # Parse the provider request
        if hasattr(translation_result, 'provider_request_json'):
            request = json.loads(translation_result.provider_request_json) if isinstance(translation_result.provider_request_json, str) else translation_result.provider_request_json
            print(f"  Request: {json.dumps(request, indent=2)}")
        
        # Display lossiness report
        if hasattr(translation_result, 'lossiness'):
            lossiness = translation_result.lossiness
            print("\n  Lossiness Report:")
            print(f"    Max Severity: {lossiness.max_severity}")
            
            if hasattr(lossiness, 'items') and lossiness.items:
                print("    Items:")
                for item in lossiness.items:
                    print(f"      - {item.severity}: {item.message} ({item.path})")
            else:
                print("    ✅ No lossiness detected")
        
        # Display metadata if present
        if hasattr(translation_result, 'metadata') and translation_result.metadata:
            print(f"\n  Metadata: {translation_result.metadata}")
        
    except Exception as e:
        print(f"❌ Translation failed: {e}")
        return
    
    # Note about execution
    print("\n🚀 To execute the request, you would call:")
    print("""
# Using the translated request with an HTTP client
import httpx

async def execute_request(request, api_key):
    async with httpx.AsyncClient() as client:
        response = await client.post(
            "https://api.openai.com/v1/chat/completions",
            json=request,
            headers={"Authorization": f"Bearer {api_key}"}
        )
        return response.json()
    """)

if __name__ == "__main__":
    main()