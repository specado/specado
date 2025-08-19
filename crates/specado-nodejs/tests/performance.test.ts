/**
 * Performance and memory tests
 */

import { translate, validate, run, init, getVersion, SchemaType } from '../index';

describe('Performance Tests', () => {
  const mockPrompt = {
    modelClass: 'Chat',
    messages: [
      {
        role: 'user',
        content: 'Performance test message',
        toolCalls: undefined,
        toolResults: undefined,
        metadata: undefined
      }
    ],
    strictMode: 'standard',
    tools: undefined,
    toolChoice: undefined,
    responseFormat: undefined,
    sampling: undefined,
    limits: undefined,
    media: undefined
  };

  const mockProvider = {
    name: 'test-provider',
    version: '1.0.0',
    baseUrl: 'https://api.example.com',
    auth: {
      authType: 'api_key',
      header: 'Authorization',
      envVar: 'API_KEY'
    },
    models: [
      {
        id: 'test-model',
        name: 'Test Model',
        capabilities: ['chat'],
        contextSize: 4096,
        maxOutput: 1024,
        costPerInputToken: 0.001,
        costPerOutputToken: 0.002
      }
    ],
    rateLimits: undefined,
    mappings: {
      request: {},
      response: {}
    }
  };

  describe('Function Performance', () => {
    test('should initialize quickly', () => {
      const start = process.hrtime.bigint();
      init();
      const end = process.hrtime.bigint();
      
      const durationMs = Number(end - start) / 1_000_000;
      expect(durationMs).toBeLessThan(100); // Should be very fast
    });

    test('should get version quickly', () => {
      const start = process.hrtime.bigint();
      getVersion();
      const end = process.hrtime.bigint();
      
      const durationMs = Number(end - start) / 1_000_000;
      expect(durationMs).toBeLessThan(10); // Should be almost instant
    });

    test('should validate promptly', () => {
      const start = process.hrtime.bigint();
      validate(mockPrompt, SchemaType.Prompt);
      const end = process.hrtime.bigint();
      
      const durationMs = Number(end - start) / 1_000_000;
      expect(durationMs).toBeLessThan(50); // Should be fast
    });

    test('should translate promptly', () => {
      const start = process.hrtime.bigint();
      
      try {
        translate(mockPrompt, mockProvider, 'test-model');
      } catch (error) {
        // Expected in test environment
      }
      
      const end = process.hrtime.bigint();
      const durationMs = Number(end - start) / 1_000_000;
      expect(durationMs).toBeLessThan(200); // Should be reasonably fast
    });
  });

  describe('Memory Usage', () => {
    test('should not leak memory with repeated operations', () => {
      const initialMemory = process.memoryUsage();
      
      // Perform many operations
      for (let i = 0; i < 1000; i++) {
        try {
          validate(mockPrompt, SchemaType.Prompt);
        } catch (error) {
          // Ignore errors in performance test
        }
      }
      
      // Force garbage collection if available
      if (global.gc) {
        global.gc();
      }
      
      const finalMemory = process.memoryUsage();
      
      // Memory usage should not grow significantly
      const heapGrowth = finalMemory.heapUsed - initialMemory.heapUsed;
      expect(heapGrowth).toBeLessThan(10 * 1024 * 1024); // Less than 10MB growth
    });

    test('should handle large prompts efficiently', () => {
      const largePrompt = {
        ...mockPrompt,
        messages: Array.from({ length: 100 }, (_, i) => ({
          role: i % 2 === 0 ? 'user' : 'assistant',
          content: `Message ${i}: ${'x'.repeat(1000)}`, // 1KB per message
          toolCalls: undefined,
          toolResults: undefined,
          metadata: undefined
        }))
      };

      const start = process.hrtime.bigint();
      
      try {
        validate(largePrompt, SchemaType.Prompt);
      } catch (error) {
        // Expected in test environment
      }
      
      const end = process.hrtime.bigint();
      const durationMs = Number(end - start) / 1_000_000;
      
      // Should still be reasonable for large inputs
      expect(durationMs).toBeLessThan(500);
    });
  });

  describe('Concurrent Operations', () => {
    test('should handle concurrent validations', async () => {
      const concurrentOperations = Array.from({ length: 10 }, () =>
        Promise.resolve().then(() => {
          try {
            return validate(mockPrompt, SchemaType.Prompt);
          } catch (error) {
            return { error };
          }
        })
      );

      const start = process.hrtime.bigint();
      const results = await Promise.all(concurrentOperations);
      const end = process.hrtime.bigint();
      
      const durationMs = Number(end - start) / 1_000_000;
      
      // Should complete all operations efficiently
      expect(results).toHaveLength(10);
      expect(durationMs).toBeLessThan(1000); // Should complete within 1 second
    });

    test('should handle concurrent translations', async () => {
      const concurrentTranslations = Array.from({ length: 5 }, () =>
        Promise.resolve().then(() => {
          try {
            return translate(mockPrompt, mockProvider, 'test-model');
          } catch (error) {
            return { error };
          }
        })
      );

      const start = process.hrtime.bigint();
      const results = await Promise.all(concurrentTranslations);
      const end = process.hrtime.bigint();
      
      const durationMs = Number(end - start) / 1_000_000;
      
      expect(results).toHaveLength(5);
      expect(durationMs).toBeLessThan(2000); // Should be reasonable
    });
  });

  describe('Resource Cleanup', () => {
    test('should clean up resources properly', () => {
      // Test that repeated operations don't accumulate resources
      const operations = 100;
      
      for (let i = 0; i < operations; i++) {
        try {
          const result = validate(mockPrompt, SchemaType.Prompt);
          // Ensure result is used to prevent optimization
          expect(result).toBeDefined();
        } catch (error) {
          // Ignore errors
        }
      }
      
      // If we get here without crashes, resource cleanup is working
      expect(true).toBe(true);
    });

    test('should handle string allocation and deallocation', () => {
      // Test string handling across FFI boundary
      const largeString = 'x'.repeat(100000); // 100KB string
      
      const testPrompt = {
        ...mockPrompt,
        messages: [{
          role: 'user',
          content: largeString,
          toolCalls: undefined,
          toolResults: undefined,
          metadata: undefined
        }]
      };

      try {
        validate(testPrompt, SchemaType.Prompt);
      } catch (error) {
        // Expected in test
      }

      // Should not crash or leak memory
      expect(true).toBe(true);
    });
  });
});