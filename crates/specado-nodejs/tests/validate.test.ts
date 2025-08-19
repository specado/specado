/**
 * Validation function tests
 */

import { validate, SchemaType, PromptSpec, ValidationResult } from '../index';

describe('Validation Function', () => {
  const validPrompt: PromptSpec = {
    modelClass: 'Chat',
    messages: [
      {
        role: 'user',
        content: 'Hello, world!',
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

  const validProvider = {
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
    mappings: {
      request: {},
      response: {}
    }
  };

  describe('Prompt Validation', () => {
    test('should validate a valid prompt spec', () => {
      const result = validate(validPrompt, SchemaType.Prompt);
      
      expect(result.valid).toBe(true);
      expect(result.errors).toHaveLength(0);
      expect(result.schemaVersion).toBe('1.0');
    });

    test('should reject prompt without model_class', () => {
      const invalidPrompt = { ...validPrompt };
      delete (invalidPrompt as any).modelClass;
      
      const result = validate(invalidPrompt, SchemaType.Prompt);
      
      expect(result.valid).toBe(false);
      expect(result.errors.length).toBeGreaterThan(0);
      expect(result.errors.some(e => e.path === 'model_class')).toBe(true);
    });

    test('should reject prompt without messages', () => {
      const invalidPrompt = { ...validPrompt };
      delete (invalidPrompt as any).messages;
      
      const result = validate(invalidPrompt, SchemaType.Prompt);
      
      expect(result.valid).toBe(false);
      expect(result.errors.length).toBeGreaterThan(0);
      expect(result.errors.some(e => e.path === 'messages')).toBe(true);
    });

    test('should reject prompt with empty messages array', () => {
      const invalidPrompt = { ...validPrompt, messages: [] };
      
      const result = validate(invalidPrompt, SchemaType.Prompt);
      
      expect(result.valid).toBe(false);
      expect(result.errors.length).toBeGreaterThan(0);
      expect(result.errors.some(e => e.path === 'messages' && e.code === 'EMPTY_ARRAY')).toBe(true);
    });

    test('should reject prompt with invalid strict_mode', () => {
      const invalidPrompt = { ...validPrompt, strictMode: 'invalid' };
      
      const result = validate(invalidPrompt, SchemaType.Prompt);
      
      expect(result.valid).toBe(false);
      expect(result.errors.length).toBeGreaterThan(0);
      expect(result.errors.some(e => e.path === 'strict_mode')).toBe(true);
    });

    test('should validate prompt with sampling parameters', () => {
      const promptWithSampling = {
        ...validPrompt,
        sampling: {
          temperature: 0.7,
          topP: 0.9,
          maxTokens: 1000
        }
      };
      
      const result = validate(promptWithSampling, SchemaType.Prompt);
      
      expect(result.valid).toBe(true);
    });

    test('should reject prompt with invalid temperature range', () => {
      const invalidPrompt = {
        ...validPrompt,
        sampling: {
          temperature: 3.0 // Invalid: > 2.0
        }
      };
      
      const result = validate(invalidPrompt, SchemaType.Prompt);
      
      expect(result.valid).toBe(false);
      expect(result.errors.some(e => e.path === 'sampling.temperature')).toBe(true);
    });

    test('should warn about conflicting sampling parameters', () => {
      const promptWithConflict = {
        ...validPrompt,
        sampling: {
          temperature: 0.7,
          topP: 0.9
        }
      };
      
      const result = validate(promptWithConflict, SchemaType.Prompt);
      
      expect(result.valid).toBe(true);
      expect(result.warnings.length).toBeGreaterThan(0);
      expect(result.warnings.some(w => w.code === 'CONFLICTING_PARAMS')).toBe(true);
    });
  });

  describe('Provider Validation', () => {
    test('should validate a valid provider spec', () => {
      const result = validate(validProvider, SchemaType.Provider);
      
      expect(result.valid).toBe(true);
      expect(result.errors).toHaveLength(0);
    });

    test('should reject provider without required fields', () => {
      const invalidProvider = { ...validProvider };
      delete (invalidProvider as any).name;
      delete (invalidProvider as any).version;
      
      const result = validate(invalidProvider, SchemaType.Provider);
      
      expect(result.valid).toBe(false);
      expect(result.errors.length).toBeGreaterThan(0);
      expect(result.errors.some(e => e.path === 'name')).toBe(true);
      expect(result.errors.some(e => e.path === 'version')).toBe(true);
    });

    test('should reject provider with empty models array', () => {
      const invalidProvider = { ...validProvider, models: [] };
      
      const result = validate(invalidProvider, SchemaType.Provider);
      
      expect(result.valid).toBe(false);
      expect(result.errors.some(e => e.path === 'models' && e.code === 'EMPTY_ARRAY')).toBe(true);
    });
  });

  describe('Error Handling', () => {
    test('should handle invalid JSON input', () => {
      expect(() => {
        validate('invalid json', SchemaType.Prompt);
      }).toThrow();
    });

    test('should handle non-object input', () => {
      const result = validate('string', SchemaType.Prompt);
      
      expect(result.valid).toBe(false);
      expect(result.errors.some(e => e.code === 'INVALID_TYPE')).toBe(true);
    });
  });
});