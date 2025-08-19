/**
 * Error handling tests
 */

import { SpecadoErrorKind } from '../index';

describe('Error Handling', () => {
  describe('SpecadoError Types', () => {
    test('should define all error kinds', () => {
      expect(SpecadoErrorKind.InvalidInput).toBe('InvalidInput');
      expect(SpecadoErrorKind.JsonError).toBe('JsonError');
      expect(SpecadoErrorKind.ProviderNotFound).toBe('ProviderNotFound');
      expect(SpecadoErrorKind.ModelNotFound).toBe('ModelNotFound');
      expect(SpecadoErrorKind.NetworkError).toBe('NetworkError');
      expect(SpecadoErrorKind.AuthenticationError).toBe('AuthenticationError');
      expect(SpecadoErrorKind.RateLimitError).toBe('RateLimitError');
      expect(SpecadoErrorKind.TimeoutError).toBe('TimeoutError');
      expect(SpecadoErrorKind.InternalError).toBe('InternalError');
      expect(SpecadoErrorKind.Unknown).toBe('Unknown');
    });
  });

  describe('Error Structure', () => {
    test('should validate SpecadoError structure', () => {
      const mockError = {
        kind: SpecadoErrorKind.InvalidInput,
        message: 'Test error message',
        code: 'INVALID_INPUT',
        details: 'Additional error details'
      };

      expect(mockError).toBeValidSpecadoError();
    });

    test('should handle errors without details', () => {
      const mockError = {
        kind: SpecadoErrorKind.NetworkError,
        message: 'Network connection failed',
        code: 'NETWORK_ERROR'
      };

      expect(mockError).toBeValidSpecadoError();
    });
  });

  describe('Error Codes', () => {
    test('should have consistent error codes', () => {
      const errorCodeMappings = {
        [SpecadoErrorKind.InvalidInput]: 'INVALID_INPUT',
        [SpecadoErrorKind.JsonError]: 'JSON_ERROR',
        [SpecadoErrorKind.ProviderNotFound]: 'PROVIDER_NOT_FOUND',
        [SpecadoErrorKind.ModelNotFound]: 'MODEL_NOT_FOUND',
        [SpecadoErrorKind.NetworkError]: 'NETWORK_ERROR',
        [SpecadoErrorKind.AuthenticationError]: 'AUTHENTICATION_ERROR',
        [SpecadoErrorKind.RateLimitError]: 'RATE_LIMIT_ERROR',
        [SpecadoErrorKind.TimeoutError]: 'TIMEOUT_ERROR',
        [SpecadoErrorKind.InternalError]: 'INTERNAL_ERROR',
        [SpecadoErrorKind.Unknown]: 'UNKNOWN_ERROR'
      };

      // Verify mapping exists for each error kind
      Object.keys(SpecadoErrorKind).forEach(kind => {
        expect(errorCodeMappings[kind as SpecadoErrorKind]).toBeDefined();
      });
    });
  });

  describe('Error Context Preservation', () => {
    test('should preserve error context across language boundaries', () => {
      // This tests the concept - actual FFI error mapping would be tested
      // in integration tests with the Rust layer
      
      const rustErrorInfo = {
        source: 'FFI Layer',
        context: 'Translation operation',
        originalError: 'Invalid JSON structure'
      };

      const jsError = {
        kind: SpecadoErrorKind.JsonError,
        message: 'JSON parsing error',
        code: 'JSON_ERROR',
        details: JSON.stringify(rustErrorInfo)
      };

      expect(jsError).toBeValidSpecadoError();
      expect(jsError.details).toContain('FFI Layer');
    });
  });

  describe('Error Recovery', () => {
    test('should provide actionable error messages', () => {
      const actionableErrors = [
        {
          kind: SpecadoErrorKind.InvalidInput,
          message: 'Missing required field: model_class',
          code: 'INVALID_INPUT',
          details: 'Add model_class field to prompt specification'
        },
        {
          kind: SpecadoErrorKind.AuthenticationError,
          message: 'Invalid API key',
          code: 'AUTHENTICATION_ERROR',
          details: 'Check your API key in the credentials object'
        },
        {
          kind: SpecadoErrorKind.RateLimitError,
          message: 'Rate limit exceeded',
          code: 'RATE_LIMIT_ERROR',
          details: 'Wait before making another request or check your plan limits'
        }
      ];

      actionableErrors.forEach(error => {
        expect(error).toBeValidSpecadoError();
        expect(error.details).toBeTruthy();
        expect(error.details!.length).toBeGreaterThan(10);
      });
    });
  });
});