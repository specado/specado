/**
 * Basic usage example for Specado Node.js bindings
 */

const specado = require('@specado/nodejs');
const fs = require('fs');
const path = require('path');

async function main() {
  try {
    console.log('🚀 Specado Node.js Bindings Example');
    
    // Initialize the library
    const initResult = specado.init();
    console.log('✅ Initialize:', initResult);
    
    // Get version information
    const version = specado.getVersion();
    console.log('📦 Version:', version);
    
    // Define a prompt with proper structure
    const prompt = {
      model_class: "Chat",
      messages: [
        {
          role: "user",
          content: "Hello, world! Can you help me understand how LLM APIs work?"
        }
      ],
      sampling: {
        temperature: 0.7
      },
      limits: {
        max_output_tokens: 1000
      },
      strict_mode: "standard"
    };
    
    console.log('\n📝 Prompt:', JSON.stringify(prompt, null, 2));
    
    // Validate the prompt
    console.log('\n🔍 Validating prompt...');
    const validation = specado.validate(prompt, specado.SchemaType.Prompt);
    
    if (validation.valid) {
      console.log('✅ Prompt is valid!');
    } else {
      console.log('❌ Validation errors:', validation.errors);
      return;
    }
    
    // Load a real provider specification from golden corpus
    const providerSpecPath = path.join(__dirname, '..', 'providers', 'openai', 'gpt-5.json');
    let providerSpec;
    
    try {
      const providerSpecJson = fs.readFileSync(providerSpecPath, 'utf8');
      providerSpec = JSON.parse(providerSpecJson);
      console.log('\n📋 Loaded provider spec:', providerSpec.provider.name);
    } catch (err) {
      console.error('❌ Failed to load provider spec:', err.message);
      console.log('Using fallback provider spec from golden corpus structure...');
      
      // Fallback to a valid provider spec structure
      providerSpec = {
        spec_version: "1.0.0",
        provider: {
          name: "openai",
          base_url: "https://api.openai.com/v1",
          headers: {
            "Authorization": "Bearer ${OPENAI_API_KEY}"
          }
        },
        models: [
          {
            id: "gpt-4",
            aliases: ["gpt-4-turbo"],
            family: "gpt",
            endpoints: {
              chat_completion: {
                method: "POST",
                path: "/chat/completions",
                protocol: "http"
              },
              streaming_chat_completion: {
                method: "POST",
                path: "/chat/completions",
                protocol: "sse"
              }
            },
            input_modes: {
              messages: true,
              single_text: false,
              images: false
            },
            tooling: {
              tools_supported: true,
              parallel_tool_calls_default: true,
              can_disable_parallel_tool_calls: true,
              disable_switch: {
                parallel_tool_calls: false
              }
            },
            json_output: {
              native_param: true,
              strategy: "response_format"
            },
            parameters: {
              temperature: {
                type: "number",
                minimum: 0.0,
                maximum: 2.0,
                default: 1.0
              },
              max_tokens: {
                type: "integer",
                minimum: 1,
                maximum: 128000
              }
            },
            constraints: {
              system_prompt_location: "first_message",
              forbid_unknown_top_level_fields: true,
              mutually_exclusive: [["temperature", "top_p"]],
              resolution_preferences: ["temperature"],
              limits: {
                max_tool_schema_bytes: 16384,
                max_system_prompt_bytes: 32768
              }
            },
            mappings: {
              paths: {
                "$.limits.max_output_tokens": "$.max_tokens",
                "$.sampling.temperature": "$.temperature"
              },
              flags: {}
            },
            response_normalization: {
              sync: {
                content_path: "$.choices[0].message.content",
                finish_reason_path: "$.choices[0].finish_reason",
                finish_reason_map: {
                  stop: "stop",
                  length: "length"
                }
              },
              stream: {
                protocol: "sse",
                event_selector: {
                  type_path: "$.choices[0].delta",
                  routes: []
                }
              }
            }
          }
        ]
      };
    }
    
    console.log('\n🔍 Validating provider spec...');
    const providerValidation = specado.validate(providerSpec, specado.SchemaType.Provider);
    
    if (providerValidation.valid) {
      console.log('✅ Provider spec is valid!');
    } else {
      console.log('❌ Provider validation errors:', providerValidation.errors);
      return;
    }
    
    // Translate the prompt
    console.log('\n🔄 Translating prompt to provider format...');
    const modelId = providerSpec.models[0].id;
    const translation = specado.translate(prompt, providerSpec, modelId, {
      mode: "standard",
      include_metadata: true
    });
    
    console.log('✅ Translation successful!');
    
    // Display the full TranslationResult including lossiness
    console.log('\n📊 Translation Result:');
    console.log('  Request:', JSON.stringify(translation.request, null, 2));
    
    if (translation.lossiness) {
      console.log('\n  Lossiness Report:');
      console.log('    Max Severity:', translation.lossiness.max_severity);
      if (translation.lossiness.items && translation.lossiness.items.length > 0) {
        console.log('    Items:');
        translation.lossiness.items.forEach(item => {
          console.log(`      - ${item.severity}: ${item.message} (${item.path})`);
        });
      } else {
        console.log('    ✅ No lossiness detected');
      }
    }
    
    if (translation.metadata) {
      console.log('\n  Metadata:', translation.metadata);
    }
    
    // Note: Actual execution would require valid API credentials
    console.log('\n🚀 To execute the request, you would call:');
    console.log(`
const response = await specado.run(translation.request, {
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