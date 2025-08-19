# FFI Memory Management Guidelines

## Overview
This document provides comprehensive guidelines for memory management across the Specado FFI boundary. Following these patterns ensures memory safety and prevents leaks when integrating Specado with other languages.

## Core Principles

### 1. Ownership Rules
- **Caller owns input**: The calling code owns and manages memory for input parameters
- **Callee allocates output**: Specado allocates memory for return values
- **Explicit deallocation**: All allocated memory must be explicitly freed using provided functions

### 2. String Handling

#### Passing Strings TO Specado
```c
// Strings passed to Specado functions must be null-terminated UTF-8
const char* json_input = "{\"prompt\": \"Hello\"}";
specado_result_t result = specado_translate(
    json_input,     // Caller owns this memory
    provider_spec,  // Caller owns this memory
    model_id,       // Caller owns this memory
    "standard",     // Caller owns this memory
    &output         // Specado will allocate
);
```

#### Receiving Strings FROM Specado
```c
char* output = NULL;
specado_result_t result = specado_translate(..., &output);

if (result == SPECADO_SUCCESS && output != NULL) {
    // Use the output string
    printf("Result: %s\n", output);
    
    // IMPORTANT: Free the string when done
    specado_string_free(output);
}
```

### 3. Error Handling

#### Getting Error Messages
```c
// Get last error - do NOT free this string
const char* error_msg = specado_get_last_error();
if (error_msg != NULL) {
    fprintf(stderr, "Error: %s\n", error_msg);
}

// Clear error state when appropriate
specado_clear_error();
```

### 4. Context Management

```c
// Create a context
specado_context_t* ctx = specado_context_new();
if (ctx == NULL) {
    // Handle allocation failure
    const char* error = specado_get_last_error();
    fprintf(stderr, "Failed to create context: %s\n", error);
    return -1;
}

// Use the context...

// Free the context when done
specado_context_free(ctx);
```

## Language-Specific Patterns

### Python (ctypes)
```python
import ctypes
from ctypes import c_char_p, POINTER, c_int

# Load the library
lib = ctypes.CDLL("./libspecado_ffi.so")

# Define function signatures
lib.specado_translate.argtypes = [
    c_char_p,  # prompt_json
    c_char_p,  # provider_spec_json
    c_char_p,  # model_id
    c_char_p,  # mode
    POINTER(c_char_p)  # out_json
]
lib.specado_translate.restype = c_int

# String cleanup
lib.specado_string_free.argtypes = [c_char_p]
lib.specado_string_free.restype = None

# Use the function
def translate(prompt, provider_spec, model_id, mode="standard"):
    output = c_char_p()
    result = lib.specado_translate(
        prompt.encode('utf-8'),
        provider_spec.encode('utf-8'),
        model_id.encode('utf-8'),
        mode.encode('utf-8'),
        ctypes.byref(output)
    )
    
    if result == 0 and output.value:
        # Convert to Python string
        result_str = output.value.decode('utf-8')
        # Free the C string
        lib.specado_string_free(output)
        return result_str
    else:
        error = lib.specado_get_last_error()
        if error:
            raise Exception(error.decode('utf-8'))
        raise Exception("Unknown error")
```

### Node.js (ffi-napi)
```javascript
const ffi = require('ffi-napi');
const ref = require('ref-napi');

// Define types
const stringPtr = ref.refType('string');

// Load the library
const lib = ffi.Library('./libspecado_ffi', {
    'specado_translate': ['int', ['string', 'string', 'string', 'string', stringPtr]],
    'specado_string_free': ['void', ['pointer']],
    'specado_get_last_error': ['string', []],
    'specado_clear_error': ['void', []]
});

// Wrapper function
function translate(prompt, providerSpec, modelId, mode = 'standard') {
    const outputRef = ref.alloc(stringPtr);
    
    const result = lib.specado_translate(
        JSON.stringify(prompt),
        JSON.stringify(providerSpec),
        modelId,
        mode,
        outputRef
    );
    
    if (result === 0) {
        const output = outputRef.deref();
        const resultStr = output.toString();
        
        // Free the allocated string
        lib.specado_string_free(output);
        
        return JSON.parse(resultStr);
    } else {
        const error = lib.specado_get_last_error();
        throw new Error(error || 'Unknown error');
    }
}
```

### Go (cgo)
```go
// #include "specado.h"
// #include <stdlib.h>
import "C"
import (
    "encoding/json"
    "fmt"
    "unsafe"
)

func Translate(prompt, providerSpec interface{}, modelID, mode string) (map[string]interface{}, error) {
    // Marshal inputs to JSON
    promptJSON, _ := json.Marshal(prompt)
    specJSON, _ := json.Marshal(providerSpec)
    
    // Convert to C strings
    cPrompt := C.CString(string(promptJSON))
    cSpec := C.CString(string(specJSON))
    cModel := C.CString(modelID)
    cMode := C.CString(mode)
    
    defer C.free(unsafe.Pointer(cPrompt))
    defer C.free(unsafe.Pointer(cSpec))
    defer C.free(unsafe.Pointer(cModel))
    defer C.free(unsafe.Pointer(cMode))
    
    // Prepare output
    var output *C.char
    
    // Call the function
    result := C.specado_translate(cPrompt, cSpec, cModel, cMode, &output)
    
    if result == 0 && output != nil {
        // Convert to Go string
        goStr := C.GoString(output)
        
        // Free the allocated string
        C.specado_string_free(output)
        
        // Parse result
        var resultMap map[string]interface{}
        json.Unmarshal([]byte(goStr), &resultMap)
        return resultMap, nil
    }
    
    // Get error message
    errMsg := C.specado_get_last_error()
    if errMsg != nil {
        return nil, fmt.Errorf("%s", C.GoString(errMsg))
    }
    return nil, fmt.Errorf("unknown error")
}
```

## Memory Leak Prevention

### Common Pitfalls

1. **Forgetting to free output strings**
   ```c
   // WRONG - Memory leak!
   char* output;
   specado_translate(..., &output);
   // Missing: specado_string_free(output);
   ```

2. **Double-freeing**
   ```c
   // WRONG - Double free!
   specado_string_free(output);
   specado_string_free(output); // Error!
   ```

3. **Freeing error strings**
   ```c
   // WRONG - Don't free error strings!
   const char* error = specado_get_last_error();
   free(error); // Never do this!
   ```

### Testing for Memory Leaks

#### Using Valgrind (Linux/macOS)
```bash
valgrind --leak-check=full --show-leak-kinds=all ./your_program
```

#### Using AddressSanitizer
```bash
# Compile with AddressSanitizer
gcc -fsanitize=address -g your_program.c -lspecado_ffi -o your_program

# Run the program
./your_program
```

#### Using Memory Profilers
- **Windows**: Application Verifier, Visual Studio Diagnostics
- **macOS**: Instruments (Leaks tool)
- **Cross-platform**: Dr. Memory

## Buffer Management

### Fixed-Size Buffers
When you know the maximum size:
```c
char buffer[8192];
size_t buffer_size = sizeof(buffer);
specado_result_t result = specado_translate_into_buffer(
    input, 
    buffer, 
    &buffer_size
);
```

### Dynamic Buffers
For unknown sizes:
```c
// First call to get size
size_t required_size = 0;
specado_translate_size(input, &required_size);

// Allocate buffer
char* buffer = malloc(required_size);

// Second call to fill buffer
specado_translate_into_buffer(input, buffer, &required_size);

// Use buffer...

// Clean up
free(buffer);
```

## Thread Safety

### Thread-Local Error State
Error messages are stored in thread-local storage:
```c
// Thread 1
specado_translate(...); // Sets error for thread 1
const char* error1 = specado_get_last_error(); // Gets thread 1's error

// Thread 2 (concurrent)
specado_translate(...); // Sets error for thread 2
const char* error2 = specado_get_last_error(); // Gets thread 2's error
```

### Concurrent Operations
Multiple threads can safely call Specado functions:
```c
#pragma omp parallel for
for (int i = 0; i < num_requests; i++) {
    char* output = NULL;
    specado_result_t result = specado_translate(
        requests[i], 
        spec, 
        model, 
        mode, 
        &output
    );
    
    if (result == SPECADO_SUCCESS) {
        process_result(output);
        specado_string_free(output);
    }
}
```

## Best Practices

1. **Always check return codes**
   ```c
   if (result != SPECADO_SUCCESS) {
       handle_error();
   }
   ```

2. **Use RAII in C++**
   ```cpp
   class SpecadoString {
       char* str_;
   public:
       SpecadoString(char* s) : str_(s) {}
       ~SpecadoString() { 
           if (str_) specado_string_free(str_); 
       }
       operator const char*() const { return str_; }
   };
   ```

3. **Implement cleanup handlers**
   ```c
   void cleanup(char** strings, size_t count) {
       for (size_t i = 0; i < count; i++) {
           if (strings[i]) {
               specado_string_free(strings[i]);
               strings[i] = NULL;
           }
       }
   }
   ```

4. **Use reference counting for shared data**
   ```c
   typedef struct {
       char* data;
       atomic_int ref_count;
   } SharedString;
   ```

## Debugging Memory Issues

### Debug Builds
Enable debug assertions:
```bash
cargo build --features debug-memory
```

### Memory Tracking
Use the built-in memory tracking (when enabled):
```c
// Get current allocation count
size_t allocations = specado_debug_allocation_count();

// Perform operations...

// Check for leaks
size_t final_count = specado_debug_allocation_count();
assert(allocations == final_count);
```

### Logging
Enable verbose logging:
```c
setenv("SPECADO_FFI_LOG", "debug", 1);
```

## Platform-Specific Considerations

### Windows
- Use `LoadLibrary` / `FreeLibrary` for dynamic loading
- Ensure calling convention matches (`__cdecl` by default)
- Handle wide strings if needed (convert to UTF-8)

### macOS
- Universal binary support for Intel and Apple Silicon
- Use `@rpath` for library loading
- Handle Objective-C interop if needed

### Linux
- Set `LD_LIBRARY_PATH` or use rpath
- Consider `dlopen` with `RTLD_LOCAL` flag
- Handle different libc implementations (glibc vs musl)

## Performance Considerations

### Minimizing Allocations
```c
// Reuse buffers when possible
static thread_local char buffer[65536];

// Use stack allocation for small strings
char small_buffer[256];
```

### Batch Operations
```c
// Process multiple items in one call
specado_translate_batch(items, count, outputs);
```

### Zero-Copy Options
When performance is critical:
```c
// Get a view into internal data (no allocation)
const char* view = specado_get_string_view(...);
// Use the view (valid until next call)
// No need to free
```

## Error Recovery

### Graceful Degradation
```c
char* output = NULL;
specado_result_t result = specado_translate(..., &output);

if (result != SPECADO_SUCCESS) {
    // Try fallback
    result = specado_translate_simple(..., &output);
}

if (result != SPECADO_SUCCESS) {
    // Use default response
    output = strdup(DEFAULT_RESPONSE);
}
```

### Resource Cleanup on Error
```c
char* temp1 = NULL;
char* temp2 = NULL;

if ((result = operation1(&temp1)) != SPECADO_SUCCESS) {
    goto cleanup;
}

if ((result = operation2(&temp2)) != SPECADO_SUCCESS) {
    goto cleanup;
}

// Success path
process(temp1, temp2);

cleanup:
    if (temp1) specado_string_free(temp1);
    if (temp2) specado_string_free(temp2);
    return result;
```

## Summary

Following these memory management guidelines ensures:
- No memory leaks
- No segmentation faults
- Predictable performance
- Cross-platform compatibility
- Easy debugging and maintenance

Always remember:
1. Check return codes
2. Free allocated memory
3. Don't free error strings
4. Use appropriate tools for testing
5. Follow platform-specific best practices