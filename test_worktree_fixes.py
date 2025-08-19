#!/usr/bin/env python3
"""Test script to verify FFI fixes in worktrees."""

import os
import sys
import json
import ctypes
from ctypes import c_char_p, c_int, POINTER
import subprocess

def test_nodejs_worktree():
    """Test Node.js worktree FFI returns correct TranslationResult."""
    print("\n🔍 Testing Node.js worktree FFI translate...")
    
    # Build the Node.js FFI
    build_cmd = ["cargo", "build", "--release", "-p", "specado-ffi"]
    subprocess.run(build_cmd, cwd="/Users/jfeinblum/code/specado/worktrees/nodejs-binding", check=True)
    
    # Load the library
    lib_path = "/Users/jfeinblum/code/specado/worktrees/nodejs-binding/target/release/libspecado_ffi.dylib"
    if not os.path.exists(lib_path):
        lib_path = lib_path.replace(".dylib", ".so")
    
    lib = ctypes.cdll.LoadLibrary(lib_path)
    
    # Setup function signatures
    lib.specado_translate.argtypes = [c_char_p, c_char_p, c_char_p, c_char_p, POINTER(c_char_p)]
    lib.specado_translate.restype = c_int
    lib.specado_string_free.argtypes = [c_char_p]
    
    # Test inputs
    prompt_json = json.dumps({
        "prompt": {
            "messages": [{"role": "user", "content": "Hello"}],
            "temperature": 0.7,
            "max_tokens": 100
        }
    })
    
    # Read provider spec from golden corpus
    with open("/Users/jfeinblum/code/specado/golden-corpus/providers/openai/openai-provider.json") as f:
        provider_json = f.read()
    
    # Call translate
    result_ptr = c_char_p()
    ret = lib.specado_translate(
        prompt_json.encode(),
        provider_json.encode(),
        b"gpt-5",
        b"standard",
        ctypes.byref(result_ptr)
    )
    
    if ret != 0:
        print("❌ Node.js FFI Translation failed")
        return False
    
    # Parse result
    result_str = result_ptr.value.decode()
    result = json.loads(result_str)
    
    # Free the string
    lib.specado_string_free(result_ptr)
    
    # Check structure
    print(f"📊 Node.js result keys: {list(result.keys())}")
    
    if "provider_request_json" in result and "lossiness" in result:
        print("✅ Node.js FFI returns correct TranslationResult")
        if "items" in result["lossiness"] and "max_severity" in result["lossiness"]:
            print("  ✅ Has complete lossiness structure")
        return True
    elif "success" in result:
        print("❌ Node.js FFI still returns simplified format")
        return False
    else:
        print(f"❌ Unknown format: {list(result.keys())}")
        return False

def test_python_worktree():
    """Test Python worktree FFI returns correct TranslationResult."""
    print("\n🔍 Testing Python worktree FFI translate...")
    
    # Build the Python FFI
    build_cmd = ["cargo", "build", "--release", "-p", "specado-ffi"]
    subprocess.run(build_cmd, cwd="/Users/jfeinblum/code/specado/worktrees/python-binding", check=True)
    
    # Load the library
    lib_path = "/Users/jfeinblum/code/specado/worktrees/python-binding/target/release/libspecado_ffi.dylib"
    if not os.path.exists(lib_path):
        lib_path = lib_path.replace(".dylib", ".so")
    
    lib = ctypes.cdll.LoadLibrary(lib_path)
    
    # Setup function signatures
    lib.specado_translate.argtypes = [c_char_p, c_char_p, c_char_p, c_char_p, POINTER(c_char_p)]
    lib.specado_translate.restype = c_int
    lib.specado_string_free.argtypes = [c_char_p]
    
    # Test inputs
    prompt_json = json.dumps({
        "prompt": {
            "messages": [{"role": "user", "content": "Hello"}],
            "temperature": 0.7,
            "max_tokens": 100
        }
    })
    
    # Read provider spec from golden corpus
    with open("/Users/jfeinblum/code/specado/golden-corpus/providers/openai/openai-provider.json") as f:
        provider_json = f.read()
    
    # Call translate
    result_ptr = c_char_p()
    ret = lib.specado_translate(
        prompt_json.encode(),
        provider_json.encode(),
        b"gpt-5",
        b"standard",
        ctypes.byref(result_ptr)
    )
    
    if ret != 0:
        print("❌ Python FFI Translation failed")
        return False
    
    # Parse result
    result_str = result_ptr.value.decode()
    result = json.loads(result_str)
    
    # Free the string
    lib.specado_string_free(result_ptr)
    
    # Check structure
    print(f"📊 Python result keys: {list(result.keys())}")
    
    if "provider_request_json" in result and "lossiness" in result:
        print("✅ Python FFI returns correct TranslationResult")
        if "items" in result["lossiness"] and "max_severity" in result["lossiness"]:
            print("  ✅ Has complete lossiness structure")
        return True
    elif "success" in result:
        print("❌ Python FFI still returns simplified format")
        return False
    else:
        print(f"❌ Unknown format: {list(result.keys())}")
        return False

if __name__ == "__main__":
    print("=" * 60)
    print("Testing Worktree FFI Fixes")
    print("=" * 60)
    
    nodejs_ok = test_nodejs_worktree()
    python_ok = test_python_worktree()
    
    print("\n" + "=" * 60)
    print("Summary:")
    print(f"  Node.js worktree: {'✅ PASS' if nodejs_ok else '❌ FAIL'}")
    print(f"  Python worktree:  {'✅ PASS' if python_ok else '❌ FAIL'}")
    print("=" * 60)
    
    if not (nodejs_ok and python_ok):
        sys.exit(1)