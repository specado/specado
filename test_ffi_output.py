#!/usr/bin/env python3
"""
Test to verify what FFI actually returns
"""

import json
import ctypes
import os
from pathlib import Path

# Load the FFI library
lib_path = Path("/Users/jfeinblum/code/specado/target/debug/libspecado_ffi.dylib")
if not lib_path.exists():
    lib_path = Path("/Users/jfeinblum/code/specado/target/release/libspecado_ffi.dylib")
    
if not lib_path.exists():
    print(f"❌ FFI library not found. Please build it first: cargo build --release")
    exit(1)

lib = ctypes.CDLL(str(lib_path))

# Define the FFI function signature
lib.specado_translate.argtypes = [
    ctypes.c_char_p,  # prompt_json
    ctypes.c_char_p,  # provider_spec_json
    ctypes.c_char_p,  # model_id
    ctypes.c_char_p,  # mode
    ctypes.POINTER(ctypes.c_char_p)  # out_json
]
lib.specado_translate.restype = ctypes.c_int

lib.specado_string_free.argtypes = [ctypes.c_char_p]

# Create test data
prompt = {
    "prompt": {
        "messages": [{"role": "user", "content": "Hello"}],
        "temperature": 0.7,
        "max_tokens": 100
    }
}

# Load provider spec
provider_spec_path = Path("/Users/jfeinblum/code/specado/golden-corpus/providers/openai/openai-provider.json")
with open(provider_spec_path, 'r') as f:
    provider_spec = json.load(f)

# Call FFI translate
prompt_json = json.dumps(prompt).encode('utf-8')
provider_json = json.dumps(provider_spec).encode('utf-8')
model_id = b"gpt-5"
mode = b"standard"

out_ptr = ctypes.c_char_p()
result = lib.specado_translate(
    prompt_json,
    provider_json,
    model_id,
    mode,
    ctypes.byref(out_ptr)
)

if result == 0:  # Success
    # Get the result string
    result_str = out_ptr.value.decode('utf-8')
    result_json = json.loads(result_str)
    
    print("✅ FFI Translation succeeded!")
    print("\n📊 Result structure:")
    print(f"  Keys: {list(result_json.keys())}")
    
    # Check what fields are present
    if 'provider_request_json' in result_json:
        print("  ✅ Has provider_request_json")
    if 'lossiness' in result_json:
        print("  ✅ Has lossiness")
        if 'items' in result_json['lossiness']:
            print("    ✅ Lossiness has items")
        if 'max_severity' in result_json['lossiness']:
            print("    ✅ Lossiness has max_severity")
    if 'metadata' in result_json:
        print("  ✅ Has metadata")
    
    # Check if it looks like the old simplified format
    if 'success' in result_json or 'request' in result_json:
        print("  ❌ WARNING: Has old simplified format fields!")
    
    print("\n📝 Full result (first 500 chars):")
    print(json.dumps(result_json, indent=2)[:500])
    
    # Free the string
    lib.specado_string_free(out_ptr)
else:
    print(f"❌ FFI Translation failed with code: {result}")
    # Try to get error message
    lib.specado_get_last_error.restype = ctypes.c_char_p
    error = lib.specado_get_last_error()
    if error:
        print(f"  Error: {error.decode('utf-8')}")