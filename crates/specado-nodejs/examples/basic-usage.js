/**
 * Basic usage example for Specado Node.js bindings
 */

const { translate, validate, run, SchemaType } = require('@specado/nodejs');

async function main() {
  try {
    console.log('🚀 Specado Node.js Bindings Example');
    
    // Initialize the library
    const initResult = require('@specado/nodejs').init();
    console.log('✅ Initialize:', initResult);
    
    // Get version information
    const version = require('@specado/nodejs').getVersion();
    console.log('📦 Version:', version);
    
    // Define a prompt
    const prompt = {
      model_class: "Chat",
      messages: [
        {
          role: "user",
          content: "Hello, world! Can you help me understand how LLM APIs work?"
        }
      ],
      sampling: {
        temperature: 0.7,
        max_tokens: 1000
      },
      strict_mode: "standard"
    };
    
    console.log('\n📝 Prompt:', JSON.stringify(prompt, null, 2));
    
    // Validate the prompt
    console.log('\n🔍 Validating prompt...');
    const validation = validate(prompt, SchemaType.Prompt);
    
    if (validation.valid) {
      console.log('✅ Prompt is valid!');
    } else {
      console.log('❌ Validation errors:', validation.errors);
      return;
    }
    
    // Define a provider specification
    const providerSpec = {
      name: "openai",
      version: "1.0.0",
      base_url: "https://api.openai.com/v1",
      auth: {
        auth_type: "bearer",
        header: "Authorization",
        env_var: "OPENAI_API_KEY"
      },
      models: [
        {
          id: "gpt-4",
          name: "GPT-4",
          capabilities: ["chat", "tools"],
          context_size: 128000,
          max_output: 4096,
          cost_per_input_token: 0.03,
          cost_per_output_token: 0.06
        }
      ],
      mappings: {
        request: {
          model: "{{ model_id }}",
          messages: "{{ messages }}",
          temperature: "{{ sampling.temperature }}",
          max_tokens: "{{ sampling.max_tokens }}"
        },
        response: {
          content: "{{ choices[0].message.content }}",
          usage: {
            input_tokens: "{{ usage.prompt_tokens }}",
            output_tokens: "{{ usage.completion_tokens }}",
            total_tokens: "{{ usage.total_tokens }}"
          }
        }
      }
    };
    
    console.log('\n🔍 Validating provider spec...');
    const providerValidation = validate(providerSpec, SchemaType.Provider);
    
    if (providerValidation.valid) {
      console.log('✅ Provider spec is valid!');
    } else {
      console.log('❌ Provider validation errors:', providerValidation.errors);
      return;
    }
    
    // Translate the prompt
    console.log('\n🔄 Translating prompt to provider format...');
    const translation = translate(prompt, providerSpec, "gpt-4", {
      mode: "standard",
      include_metadata: true
    });
    
    console.log('✅ Translation successful!');
    console.log('📋 Metadata:', translation.metadata);
    console.log('🔧 Translated request:', JSON.stringify(translation.request, null, 2));
    
    if (translation.warnings.length > 0) {
      console.log('⚠️  Warnings:', translation.warnings);
    }
    
    // Note: Actual execution would require valid API credentials
    console.log('\n🚀 To execute the request, you would call:');
    console.log(`
const response = await run({
  provider: providerSpec,
  request: translation.request,
  credentials: { api_key: process.env.OPENAI_API_KEY }
}, providerSpec, {
  timeout_seconds: 30,
  max_retries: 3
});

console.log('Response:', response.content);
console.log('Usage:', response.usage);
    `);
    
  } catch (error) {
    console.error('❌ Error:', error);
  }
}

// Run the example
if (require.main === module) {
  main().catch(console.error);
}

module.exports = { main };