#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>

/// Result codes for FFI operations
enum class SpecadoResult {
  /// Operation completed successfully
  Success = 0,
  /// Invalid input parameters
  InvalidInput = -1,
  /// JSON parsing error
  JsonError = -2,
  /// Provider not found
  ProviderNotFound = -3,
  /// Model not found
  ModelNotFound = -4,
  /// Network error during operation
  NetworkError = -5,
  /// Authentication failure
  AuthenticationError = -6,
  /// Rate limit exceeded
  RateLimitError = -7,
  /// Timeout occurred
  TimeoutError = -8,
  /// Internal error
  InternalError = -9,
  /// Memory allocation failure
  MemoryError = -10,
  /// Invalid UTF-8 string
  Utf8Error = -11,
  /// Null pointer provided
  NullPointer = -12,
  /// Operation cancelled
  Cancelled = -13,
  /// Not implemented
  NotImplemented = -14,
  /// Unknown error
  Unknown = -99,
};

/// Opaque handle for a Specado context
struct SpecadoContext {
  uint8_t _private[0];
};

/// FFI-safe byte buffer
struct SpecadoBuffer {
  /// Pointer to byte data
  const uint8_t *data;
  /// Length of the buffer
  uintptr_t len;
  /// Whether this buffer owns its data
  bool owned;
};

extern "C" {

/// Initialize a new Specado context
///
/// # Safety
/// The returned context must be freed with `specado_context_free`
SpecadoContext *specado_context_new();

/// Free a Specado context
///
/// # Safety
/// The context pointer must have been created by `specado_context_new`
void specado_context_free(SpecadoContext *context);

/// Translate a prompt to a provider-specific request
///
/// # Parameters
/// - `prompt_json`: JSON string containing the prompt and configuration
/// - `provider_spec_json`: JSON string containing the provider specification
/// - `model_id`: The model identifier to use
/// - `mode`: Translation mode ("standard" or "strict")
/// - `out_json`: Output parameter for the resulting JSON string
///
/// # Returns
/// A SpecadoResult indicating success or failure
///
/// # Safety
/// - All string pointers must be valid null-terminated C strings
/// - The output string must be freed with `specado_string_free`
SpecadoResult specado_translate(const char *prompt_json,
                                const char *provider_spec_json,
                                const char *model_id,
                                const char *mode,
                                char **out_json);

/// Run a translated request against a provider
///
/// # Parameters
/// - `provider_request_json`: JSON string containing the provider request
/// - `timeout_seconds`: Timeout in seconds (0 for default)
/// - `out_response_json`: Output parameter for the response JSON
///
/// # Returns
/// A SpecadoResult indicating success or failure
///
/// # Safety
/// - All string pointers must be valid null-terminated C strings
/// - The output string must be freed with `specado_string_free`
SpecadoResult specado_run(const char *provider_request_json,
                          int timeout_seconds,
                          char **out_response_json);

/// Validate a specification against its schema
///
/// # Parameters
/// - `spec_json`: JSON string containing the specification to validate
/// - `spec_type`: Type of specification ("prompt_spec" or "provider_spec")
/// - `mode`: Validation mode ("basic", "partial", or "strict")
/// - `out_result_json`: Output parameter for the validation result JSON
///
/// # Returns
/// A SpecadoResult indicating success or failure
///
/// # Safety
/// - All string pointers must be valid null-terminated C strings
/// - The output string must be freed with `specado_string_free`
SpecadoResult specado_validate(const char *spec_json,
                               const char *spec_type,
                               const char *mode,
                               char **out_result_json);

/// Get version information
///
/// # Returns
/// A static string containing version information
///
/// # Safety
/// The returned string should NOT be freed
const char *specado_version();

/// Free a string allocated by Specado
///
/// # Safety
/// The pointer must have been allocated by `allocate_string` or similar
void specado_string_free(char *s);

/// Free a buffer allocated by Specado
///
/// # Safety
/// The pointer must have been allocated by `allocate_buffer`
void specado_buffer_free(SpecadoBuffer *buffer);

/// Get the last error message
///
/// # Safety
/// Returns a pointer that should NOT be freed by the caller
const char *specado_get_last_error();

/// Clear the last error message
void specado_clear_error();

} // extern "C"
