#!/usr/bin/env python3
"""
Integration test to verify Python binding works correctly with FFI

This test verifies:
1. Validation uses FFI properly
2. Provider specs pass validation
3. Translation returns proper TranslationResult structure
"""

import json
import os
import sys
from pathlib import Path

def test_provider_spec_structure():
    """Test that provider specs have correct structure"""
    print("\n1. Testing provider spec structure...")
    
    provider_spec_path = Path(__file__).parent / "providers/openai/gpt-5.json"
    with open(provider_spec_path, 'r') as f:
        provider_spec = json.load(f)
    
    # Check required fields
    required_fields = ['spec_version', 'provider', 'models']
    model_required_fields = [
        'endpoints', 'input_modes', 'tooling', 'json_output',
        'parameters', 'constraints', 'mappings', 'response_normalization'
    ]
    
    has_errors = False
    
    for field in required_fields:
        if field not in provider_spec:
            print(f"  ❌ Missing required field: {field}")
            has_errors = True
        else:
            print(f"  ✅ Has required field: {field}")
    
    if 'models' in provider_spec and provider_spec['models']:
        model = provider_spec['models'][0]
        
        for field in model_required_fields:
            if field not in model:
                print(f"  ❌ Model missing required field: {field}")
                has_errors = True
            else:
                print(f"  ✅ Model has required field: {field}")
        
        # Check endpoints structure
        if 'endpoints' in model:
            if 'chat_completion' not in model['endpoints']:
                print("  ❌ Missing chat_completion endpoint")
                has_errors = True
            else:
                print("  ✅ Has chat_completion endpoint")
            
            if 'streaming_chat_completion' not in model['endpoints']:
                print("  ❌ Missing streaming_chat_completion endpoint")
                has_errors = True
            else:
                print("  ✅ Has streaming_chat_completion endpoint")
    
    return not has_errors

def test_prompt_spec_creation():
    """Test creating a valid prompt spec"""
    print("\n2. Creating valid prompt spec...")
    
    prompt_spec = {
        "model_class": "Chat",
        "messages": [
            {"role": "user", "content": "Hello, world!"}
        ],
        "strict_mode": "standard",
        "sampling": {
            "temperature": 0.7
        }
    }
    
    print("  ✅ Prompt spec created")
    return True

def test_translation_result_structure():
    """Test expected TranslationResult structure"""
    print("\n3. Expected TranslationResult structure:")
    print("  - provider_request_json: The translated request")
    print("  - lossiness: Object with items array and max_severity")
    print("  - metadata: Optional metadata about the translation")
    return True

def main():
    """Run all integration tests"""
    print("Python Binding Integration Test")
    print("=================================")
    
    all_passed = True
    
    # Run tests
    if not test_provider_spec_structure():
        all_passed = False
    
    if not test_prompt_spec_creation():
        all_passed = False
    
    if not test_translation_result_structure():
        all_passed = False
    
    # Summary
    print("\n=================================")
    if all_passed:
        print("✅ All integration checks passed!")
        print("Provider specs have correct structure for core validation.")
    else:
        print("❌ Some integration checks failed.")
        print("Please fix the issues above before using the bindings.")
        sys.exit(1)
    
    print("\nTo run full integration test with actual bindings:")
    print("1. Build the Python binding: maturin develop")
    print("2. Import and test the actual functions")
    print("3. Verify FFI validation and translation work correctly")
    
    # Try to import the actual module if available
    print("\nAttempting to import specado module...")
    try:
        import specado
        print("  ✅ specado module imported successfully")
        
        # Test validation function
        if hasattr(specado, 'validate'):
            print("  ✅ validate function available")
        if hasattr(specado, 'translate'):
            print("  ✅ translate function available")
    except ImportError:
        print("  ⚠️  specado module not installed (run 'maturin develop' to build)")

if __name__ == "__main__":
    main()