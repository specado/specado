#!/bin/bash

# Story generator using Specado properly

# Colors for better output
CYAN='\033[0;36m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Load environment variables
if [ -f .env ]; then
  export $(cat .env | grep -v '^#' | xargs)
else
  echo -e "${RED}Error: .env file not found${NC}"
  exit 1
fi

SPECADO="./target/release/specado"

# Build if needed
if [ ! -f "$SPECADO" ]; then
  echo "Building Specado..."
  cargo build --release -p specado-cli
fi

clear
echo -e "${CYAN}╔══════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║     📚 Story Generator (via Specado) 📚          ║${NC}"
echo -e "${CYAN}╚══════════════════════════════════════════════════╝${NC}"
echo ""

# Interactive mode - get topic from user
if [ -z "$1" ]; then
  echo -e "${GREEN}What would you like your story to be about?${NC}"
  echo -e "${YELLOW}(e.g., 'a dragon afraid of flying', 'robots in love', 'the last tree on Earth')${NC}"
  echo -n "> "
  read TOPIC
else
  TOPIC="$1"
fi

# Ask for additional preferences
echo ""
echo -e "${GREEN}Choose a story style:${NC}"
echo "1) Adventure"
echo "2) Mystery"
echo "3) Comedy"
echo "4) Drama"
echo "5) Sci-Fi"
echo "6) Fantasy"
echo "7) Horror"
echo "8) Romance"
echo "9) Surprise me!"
echo -n "> "
read STYLE_CHOICE

case $STYLE_CHOICE in
  1) STYLE="adventure";;
  2) STYLE="mystery";;
  3) STYLE="comedy";;
  4) STYLE="drama";;
  5) STYLE="science fiction";;
  6) STYLE="fantasy";;
  7) STYLE="horror";;
  8) STYLE="romance";;
  *) STYLE="any style you choose";;
esac

echo ""
echo -e "${GREEN}Story length:${NC}"
echo "1) Short (300-500 words)"
echo "2) Medium (500-800 words)"  
echo "3) Long (800-1200 words)"
echo -n "> "
read LENGTH_CHOICE

case $LENGTH_CHOICE in
  1) 
    LENGTH="300-500 words"
    MAX_TOKENS=800
    ;;
  2) 
    LENGTH="500-800 words"
    MAX_TOKENS=1500
    ;;
  3) 
    LENGTH="800-1200 words"
    MAX_TOKENS=2500
    ;;
  *)
    LENGTH="500-800 words"
    MAX_TOKENS=1500
    ;;
esac

# First, we need to create a proper provider spec that combines base provider + model
# Let's create a combined provider spec for OpenAI with GPT-5 (or GPT-4 as fallback)
cat > /tmp/openai_provider.yaml << 'ENDOFYAML'
spec_version: "1.0.0"
provider:
  name: "openai"
  base_url: "https://api.openai.com/v1"
  headers:
    Authorization: "Bearer ${OPENAI_API_KEY}"

models:
  - id: "gpt-5"
    aliases: ["gpt-5-turbo"]
    family: "gpt5"
    endpoints:
      chat_completion:
        method: "POST"
        path: "/chat/completions"
        protocol: "https"
      streaming_chat_completion:
        method: "POST"
        path: "/chat/completions"
        protocol: "https"
    input_modes:
      messages: true
      single_text: false
      images: true
    tooling:
      tools_supported: true
      parallel_tool_calls_default: true
      can_disable_parallel_tool_calls: true
      disable_switch: "parallel_tool_calls"
    json_output:
      native_param: true
      strategy: "response_format"
    parameters:
      type: "object"
      properties:
        temperature:
          type: "number"
          minimum: 0
          maximum: 2
        max_tokens:
          type: "integer"
        top_p:
          type: "number"
        frequency_penalty:
          type: "number"
        presence_penalty:
          type: "number"
    constraints:
      system_prompt_location: "first"
      forbid_unknown_top_level_fields: false
      mutually_exclusive: []
      resolution_preferences: []
      limits:
        max_tool_schema_bytes: 100000
        max_system_prompt_bytes: 100000
    mappings:
      paths:
        "sampling.temperature": "temperature"
        "sampling.max_tokens": "max_tokens"
        "sampling.top_p": "top_p"
        "sampling.frequency_penalty": "frequency_penalty"
        "sampling.presence_penalty": "presence_penalty"
        "limits.max_output_tokens": "max_tokens"
      flags: {}
    response_normalization:
      sync:
        content_path: "$.choices[0].message.content"
        finish_reason_path: "$.choices[0].finish_reason"
        finish_reason_map:
          "stop": "Stop"
          "length": "Length"
          "content_filter": "ContentFilter"
      stream:
        protocol: "sse"
        event_selector:
          type_path: "$.object"
          routes: []
  
  - id: "gpt-4o"
    aliases: ["gpt-4-turbo"]
    family: "gpt4"
    endpoints:
      chat_completion:
        method: "POST"
        path: "/chat/completions"
        protocol: "https"
      streaming_chat_completion:
        method: "POST"
        path: "/chat/completions"
        protocol: "https"
    input_modes:
      messages: true
      single_text: false
      images: true
    tooling:
      tools_supported: true
      parallel_tool_calls_default: true
      can_disable_parallel_tool_calls: true
      disable_switch: "parallel_tool_calls"
    json_output:
      native_param: true
      strategy: "response_format"
    parameters:
      type: "object"
      properties:
        temperature:
          type: "number"
        max_tokens:
          type: "integer"
    constraints:
      system_prompt_location: "first"
      forbid_unknown_top_level_fields: false
      mutually_exclusive: []
      resolution_preferences: []
      limits:
        max_tool_schema_bytes: 100000
        max_system_prompt_bytes: 100000
    mappings:
      paths:
        "sampling.temperature": "temperature"
        "sampling.max_tokens": "max_tokens"
        "limits.max_output_tokens": "max_tokens"
      flags: {}
    response_normalization:
      sync:
        content_path: "$.choices[0].message.content"
        finish_reason_path: "$.choices[0].finish_reason"
        finish_reason_map:
          "stop": "Stop"
          "length": "Length"
      stream:
        protocol: "sse"
        event_selector:
          type_path: "$.object"
          routes: []
ENDOFYAML

# Create the prompt for story generation with proper format
cat > /tmp/story_prompt.json << ENDOFJSON
{
  "model_class": "Chat",
  "messages": [
    {
      "role": "system",
      "content": "You are an acclaimed creative writer known for vivid storytelling. Write engaging stories with memorable characters, rich descriptions, and satisfying narrative arcs."
    },
    {
      "role": "user",
      "content": "Write an original ${STYLE} story about: ${TOPIC}\n\nThe story should be ${LENGTH} long. Start with an engaging opening and end with a satisfying conclusion."
    }
  ],
  "sampling": {
    "temperature": 0.85,
    "max_tokens": ${MAX_TOKENS},
    "top_p": 0.95
  },
  "strict_mode": "Warn"
}
ENDOFJSON

echo ""
echo -e "${CYAN}✨ Creating your ${STYLE} story about: ${TOPIC}${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Step 1: Translate the prompt using Specado preview
echo "Step 1: Translating prompt with Specado..."
FULL_TRANSLATION=$($SPECADO preview /tmp/story_prompt.json \
  --provider /tmp/openai_provider.yaml \
  --model gpt-4o \
  --output json 2>&1)

if [[ $? -ne 0 ]]; then
  echo -e "${RED}Translation failed. Error:${NC}"
  echo "$FULL_TRANSLATION"
  echo ""
  echo "Trying with simpler provider spec..."
  
  # Try with a minimal provider spec
  MODEL="gpt-4o"
else
  echo -e "${GREEN}✓ Translation successful${NC}"
  
  # Extract just the provider_request_json part (which includes the model field)
  TRANSLATED_REQUEST=$(echo "$FULL_TRANSLATION" | jq -r '.provider_request_json')
  
  # Save the translated request
  echo "$TRANSLATED_REQUEST" > /tmp/translated_request.json
  
  echo ""
  echo "Step 2: Executing request via OpenAI API..."
  
  # Now call the OpenAI API with the translated request
  STORY_OUTPUT=$(curl -s https://api.openai.com/v1/chat/completions \
    -H "Content-Type: application/json" \
    -H "Authorization: Bearer $OPENAI_API_KEY" \
    -d "$TRANSLATED_REQUEST")
  
  # Extract and display the story
  STORY=$(echo "$STORY_OUTPUT" | jq -r '.choices[0].message.content' 2>/dev/null)
  
  if [ ! -z "$STORY" ] && [ "$STORY" != "null" ]; then
    echo ""
    echo "$STORY" | fold -s -w 80
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo -e "${GREEN}✓ Story generated successfully via Specado translation!${NC}"
  else
    echo -e "${RED}Failed to extract story from response${NC}"
    echo "Raw response:"
    echo "$STORY_OUTPUT" | jq . 2>/dev/null || echo "$STORY_OUTPUT"
  fi
fi

# Show lossiness report if available
echo ""
echo -e "${YELLOW}Step 3: Checking for lossiness...${NC}"
$SPECADO preview /tmp/story_prompt.json \
  --provider /tmp/openai_provider.yaml \
  --model gpt-4o \
  --show-lossiness \
  --output human 2>/dev/null | grep -A20 "Lossiness" || echo "No lossiness detected"

echo ""
echo -e "${GREEN}Would you like to:${NC}"
echo "1) Save this story"
echo "2) Generate another story"
echo "3) Exit"
echo -n "> "
read CHOICE

case $CHOICE in
  1)
    FILENAME="story_$(date +%Y%m%d_%H%M%S).txt"
    if [ ! -z "$STORY" ]; then
      echo "$STORY" > "$FILENAME"
      echo -e "${GREEN}Story saved to: $FILENAME${NC}"
    fi
    echo ""
    echo "Generate another? (y/n)"
    read -n 1 AGAIN
    if [ "$AGAIN" = "y" ] || [ "$AGAIN" = "Y" ]; then
      exec "$0"
    fi
    ;;
  2)
    exec "$0"
    ;;
  *)
    echo -e "${CYAN}Thanks for using the Specado story generator!${NC}"
    ;;
esac

# Clean up
rm -f /tmp/story_prompt.json /tmp/openai_provider.yaml /tmp/translated_request.json