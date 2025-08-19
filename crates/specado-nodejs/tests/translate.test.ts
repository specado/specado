/**
 * Translation function tests
 */

import { translate, PromptSpec, ProviderSpec, TranslateOptions } from '../index';

describe('Translation Function', () => {
  const mockPrompt: PromptSpec = {
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
    sampling: {
      temperature: 0.7,
      topP: undefined,
      topK: undefined,
      frequencyPenalty: undefined,
      presencePenalty: undefined,
      stop: undefined,
      seed: undefined
    },
    limits: {
      maxTokens: 1000,
      maxInputTokens: undefined,
      maxTotalTokens: undefined
    },
    media: undefined
  };

  const mockProvider: ProviderSpec = {
    name: 'openai',
    version: '1.0.0',
    baseUrl: 'https://api.openai.com/v1',
    auth: {
      authType: 'bearer',
      header: 'Authorization',
      envVar: 'OPENAI_API_KEY'
    },
    models: [
      {
        id: 'gpt-4',
        name: 'GPT-4',
        capabilities: ['chat', 'tools'],
        contextSize: 128000,
        maxOutput: 4096,
        costPerInputToken: 0.03,
        costPerOutputToken: 0.06
      }
    ],
    rateLimits: {
      requestsPerMinute: 500,
      tokensPerMinute: 10000,
      concurrentRequests: 100
    },
    mappings: {
      request: {
        model: '{{ model_id }}',
        messages: '{{ messages }}',
        max_tokens: '{{ limits.max_tokens }}',
        temperature: '{{ sampling.temperature }}'
      },
      response: {
        content: '{{ choices[0].message.content }}',
        usage: {
          input_tokens: '{{ usage.prompt_tokens }}',
          output_tokens: '{{ usage.completion_tokens }}',
          total_tokens: '{{ usage.total_tokens }}'
        }
      }
    }
  };

  describe('Basic Translation', () => {
    test('should translate a basic prompt successfully', () => {
      const result = translate(mockPrompt, mockProvider, 'gpt-4');
      
      expect(result).toBeDefined();
      expect(result.request).toBeDefined();
      expect(result.metadata).toBeDefined();
      expect(result.warnings).toBeDefined();
      
      expect(result.metadata.targetProvider).toBe('openai');
      expect(result.metadata.targetModel).toBe('gpt-4');
      expect(result.metadata.sourceVersion).toBe('1.0');
      expect(result.metadata.featuresUsed).toContain('messages');
    });

    test('should include sampling features in metadata', () => {
      const result = translate(mockPrompt, mockProvider, 'gpt-4');
      
      expect(result.metadata.featuresUsed).toContain('sampling');
      expect(result.metadata.featuresUsed).toContain('limits');
    });

    test('should handle translation with tools', () => {
      const promptWithTools: PromptSpec = {
        ...mockPrompt,
        tools: [
          {
            name: 'get_weather',
            description: 'Get weather information',
            parameters: {
              type: 'object',
              properties: {
                location: { type: 'string' }
              },
              required: ['location']
            },
            required: false
          }
        ],
        toolChoice: {
          choiceType: 'auto',
          toolName: undefined
        }
      };
      
      const result = translate(promptWithTools, mockProvider, 'gpt-4');
      
      expect(result.metadata.featuresUsed).toContain('tools');
      expect(result.metadata.featuresUsed).toContain('tool_choice');
    });

    test('should handle response format specification', () => {
      const promptWithFormat: PromptSpec = {
        ...mockPrompt,
        responseFormat: {
          formatType: 'json_object',
          schema: undefined
        }
      };
      
      const result = translate(promptWithFormat, mockProvider, 'gpt-4');
      
      expect(result.metadata.featuresUsed).toContain('response_format');
    });
  });

  describe('Translation Options', () => {
    test('should handle standard mode', () => {
      const options: TranslateOptions = {
        mode: 'standard',
        includeMetadata: true,
        customRules: undefined
      };
      
      const result = translate(mockPrompt, mockProvider, 'gpt-4', options);
      
      expect(result).toBeDefined();
      expect(result.metadata).toBeDefined();
    });

    test('should handle strict mode', () => {
      const options: TranslateOptions = {
        mode: 'strict',
        includeMetadata: true,
        customRules: undefined
      };
      
      const result = translate(mockPrompt, mockProvider, 'gpt-4', options);
      
      expect(result).toBeDefined();
    });

    test('should use default options when none provided', () => {
      const result = translate(mockPrompt, mockProvider, 'gpt-4');
      
      expect(result).toBeDefined();
      expect(result.metadata).toBeDefined();
    });
  });

  describe('Error Handling', () => {
    test('should handle invalid prompt format', () => {
      const invalidPrompt = {
        ...mockPrompt,
        messages: 'invalid' // Should be array
      } as any;
      
      expect(() => {
        translate(invalidPrompt, mockProvider, 'gpt-4');
      }).toThrow();
    });

    test('should handle invalid provider format', () => {
      const invalidProvider = {
        ...mockProvider,
        name: null // Should be string
      } as any;
      
      expect(() => {
        translate(mockPrompt, invalidProvider, 'gpt-4');
      }).toThrow();
    });

    test('should handle empty model ID', () => {
      expect(() => {
        translate(mockPrompt, mockProvider, '');
      }).toThrow();
    });
  });

  describe('Metadata Generation', () => {
    test('should generate accurate timestamp', () => {
      const before = new Date();
      const result = translate(mockPrompt, mockProvider, 'gpt-4');
      const after = new Date();
      
      const timestamp = new Date(result.metadata.timestamp);
      expect(timestamp.getTime()).toBeGreaterThanOrEqual(before.getTime());
      expect(timestamp.getTime()).toBeLessThanOrEqual(after.getTime());
    });

    test('should track all used features', () => {
      const complexPrompt: PromptSpec = {
        ...mockPrompt,
        tools: [{ name: 'test', description: 'test', parameters: {}, required: false }],
        toolChoice: { choiceType: 'auto', toolName: undefined },
        responseFormat: { formatType: 'json_object', schema: undefined },
        media: { inputTypes: ['image'], outputTypes: undefined, maxFileSize: undefined }
      };
      
      const result = translate(complexPrompt, mockProvider, 'gpt-4');
      
      expect(result.metadata.featuresUsed).toContain('messages');
      expect(result.metadata.featuresUsed).toContain('tools');
      expect(result.metadata.featuresUsed).toContain('tool_choice');
      expect(result.metadata.featuresUsed).toContain('response_format');
      expect(result.metadata.featuresUsed).toContain('sampling');
      expect(result.metadata.featuresUsed).toContain('limits');
      expect(result.metadata.featuresUsed).toContain('media');
    });
  });
});