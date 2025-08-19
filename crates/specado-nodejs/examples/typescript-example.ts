/**
 * TypeScript usage example for Specado Node.js bindings
 */

import { 
  translate, 
  validate, 
  run, 
  init,
  getVersionInfo,
  SchemaType,
  PromptSpec,
  ProviderSpec,
  TranslateOptions,
  RunOptions,
  SpecadoErrorKind
} from '@specado/nodejs';

async function demonstrateBasicUsage(): Promise<void> {
  console.log('🚀 Specado TypeScript Example');
  
  // Initialize and get version info
  const initMessage = init();
  const versionInfo = getVersionInfo();
  
  console.log('✅ Initialized:', initMessage);
  console.log('📦 Version Info:', versionInfo);
  
  // Define a typed prompt specification
  const prompt: PromptSpec = {
    modelClass: "Chat",
    messages: [
      {
        role: "user",
        content: "Explain quantum computing in simple terms",
        toolCalls: undefined,
        toolResults: undefined,
        metadata: undefined
      }
    ],
    tools: [
      {
        name: "search_web",
        description: "Search the web for information",
        parameters: {
          type: "object",
          properties: {
            query: {
              type: "string",
              description: "The search query"
            }
          },
          required: ["query"]
        },
        required: false
      }
    ],
    toolChoice: {
      choiceType: "auto",
      toolName: undefined
    },
    responseFormat: {
      formatType: "json_object",
      schema: undefined
    },
    sampling: {
      temperature: 0.7,
      topP: 0.9,
      maxTokens: 1500,
      topK: undefined,
      frequencyPenalty: undefined,
      presencePenalty: undefined,
      stop: undefined,
      seed: undefined
    },
    limits: {
      maxTokens: 1500,
      maxInputTokens: undefined,
      maxTotalTokens: undefined
    },
    media: {
      inputTypes: ["text"],
      outputTypes: ["text"],
      maxFileSize: undefined
    },
    strictMode: "standard"
  };
  
  // Define a typed provider specification
  const providerSpec: ProviderSpec = {
    name: "anthropic",
    version: "1.0.0",
    baseUrl: "https://api.anthropic.com/v1",
    auth: {
      authType: "header",
      header: "x-api-key",
      envVar: "ANTHROPIC_API_KEY"
    },
    models: [
      {
        id: "claude-3-5-sonnet-20241022",
        name: "Claude 3.5 Sonnet",
        capabilities: ["chat", "tools", "vision"],
        contextSize: 200000,
        maxOutput: 8192,
        costPerInputToken: 0.003,
        costPerOutputToken: 0.015
      }
    ],
    rateLimits: {
      requestsPerMinute: 1000,
      tokensPerMinute: 40000,
      concurrentRequests: 5
    },
    mappings: {
      request: {
        model: "{{ model_id }}",
        messages: "{{ messages }}",
        tools: "{{ tools }}",
        tool_choice: "{{ tool_choice }}",
        max_tokens: "{{ limits.max_tokens }}",
        temperature: "{{ sampling.temperature }}"
      },
      response: {
        content: "{{ content[0].text }}",
        usage: {
          input_tokens: "{{ usage.input_tokens }}",
          output_tokens: "{{ usage.output_tokens }}"
        },
        tool_calls: "{{ content[?type=='tool_use'] }}"
      },
      errors: {
        rate_limit: "{{ error.type == 'rate_limit_error' }}",
        invalid_request: "{{ error.type == 'invalid_request_error' }}"
      }
    }
  };
  
  console.log('\n🔍 Validating specifications...');
  
  // Validate with proper error handling
  try {
    const promptValidation = validate(prompt, SchemaType.Prompt);
    const providerValidation = validate(providerSpec, SchemaType.Provider);
    
    if (!promptValidation.valid) {
      console.error('❌ Prompt validation failed:');
      promptValidation.errors.forEach(error => {
        console.error(`  - ${error.path}: ${error.message} (${error.code})`);
      });
      return;
    }
    
    if (!providerValidation.valid) {
      console.error('❌ Provider validation failed:');
      providerValidation.errors.forEach(error => {
        console.error(`  - ${error.path}: ${error.message} (${error.code})`);
      });
      return;
    }
    
    console.log('✅ All validations passed!');
    
    // Show warnings if any
    if (promptValidation.warnings.length > 0) {
      console.log('⚠️  Prompt warnings:');
      promptValidation.warnings.forEach(warning => {
        console.log(`  - ${warning.path}: ${warning.message}`);
      });
    }
    
  } catch (error) {
    console.error('❌ Validation error:', error);
    return;
  }
  
  console.log('\n🔄 Performing translation...');
  
  // Translate with options
  try {
    const translateOptions: TranslateOptions = {
      mode: "standard",
      includeMetadata: true,
      customRules: undefined
    };
    
    const translation = translate(
      prompt, 
      providerSpec, 
      "claude-3-5-sonnet-20241022",
      translateOptions
    );
    
    console.log('✅ Translation completed!');
    console.log('📊 Features used:', translation.metadata.featuresUsed);
    console.log('⚠️  Unsupported features:', translation.metadata.unsupportedFeatures);
    console.log('🕒 Timestamp:', translation.metadata.timestamp);
    
    if (translation.warnings.length > 0) {
      console.log('⚠️  Translation warnings:');
      translation.warnings.forEach(warning => console.log(`  - ${warning}`));
    }
    
    // Demonstrate async execution (would need real credentials)
    console.log('\n🚀 Execution example (requires credentials):');
    const runOptions: RunOptions = {
      timeoutSeconds: 60,
      maxRetries: 3,
      followRedirects: true,
      userAgent: "Specado TypeScript Example/1.0"
    };
    
    console.log(`
// This would execute the actual request:
const response = await run({
  provider: providerSpec,
  request: translation.request,
  credentials: { 
    api_key: process.env.ANTHROPIC_API_KEY 
  },
  headers: {
    'User-Agent': 'MyApp/1.0'
  }
}, providerSpec, runOptions);

// Response would contain:
// - content: string
// - usage: { inputTokens, outputTokens, totalTokens, estimatedCost }
// - metadata: { provider, model, timestamp, requestId, responseTimeMs }
// - toolCalls: ToolCall[] (if any)
// - finishReason: string
    `);
    
  } catch (error: any) {
    console.error('❌ Translation failed:', error);
    
    // Demonstrate typed error handling
    if (error.kind) {
      switch (error.kind) {
        case SpecadoErrorKind.InvalidInput:
          console.error('🔍 Input validation issue:', error.message);
          break;
        case SpecadoErrorKind.JsonError:
          console.error('📄 JSON parsing issue:', error.message);
          break;
        case SpecadoErrorKind.ProviderNotFound:
          console.error('🔍 Provider issue:', error.message);
          break;
        default:
          console.error('❓ Unknown error type:', error.kind);
      }
      
      if (error.details) {
        console.error('📋 Details:', error.details);
      }
    }
  }
}

async function demonstrateErrorHandling(): Promise<void> {
  console.log('\n🧪 Error Handling Demonstration');
  
  // Demonstrate validation errors
  const invalidPrompt = {
    // Missing required fields
    messages: "invalid" // Should be array
  };
  
  try {
    const result = validate(invalidPrompt, SchemaType.Prompt);
    console.log('📊 Validation result:', {
      valid: result.valid,
      errorCount: result.errors.length,
      warningCount: result.warnings.length
    });
    
    if (!result.valid) {
      console.log('❌ Expected validation errors:');
      result.errors.slice(0, 3).forEach(error => {
        console.log(`  - ${error.code}: ${error.message}`);
      });
    }
  } catch (error) {
    console.error('❌ Unexpected error during validation:', error);
  }
  
  // Demonstrate translation errors
  try {
    const validPrompt: PromptSpec = {
      modelClass: "Chat",
      messages: [{ 
        role: "user", 
        content: "test",
        toolCalls: undefined,
        toolResults: undefined,
        metadata: undefined
      }],
      strictMode: "standard",
      tools: undefined,
      toolChoice: undefined,
      responseFormat: undefined,
      sampling: undefined,
      limits: undefined,
      media: undefined
    };
    
    const invalidProvider = {
      name: "invalid-provider"
      // Missing required fields
    };
    
    translate(validPrompt, invalidProvider as any, "test-model");
  } catch (error: any) {
    console.log('✅ Caught expected translation error:', error.message);
  }
}

// Main execution
async function main(): Promise<void> {
  try {
    await demonstrateBasicUsage();
    await demonstrateErrorHandling();
    
    console.log('\n🎉 TypeScript example completed successfully!');
  } catch (error) {
    console.error('💥 Unexpected error:', error);
    process.exit(1);
  }
}

// Run if this is the main module
if (require.main === module) {
  main().catch(console.error);
}

export { main, demonstrateBasicUsage, demonstrateErrorHandling };