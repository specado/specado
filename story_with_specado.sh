#!/bin/bash

# Story generator using Specado with gpt-5-mini

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

# Load the existing gpt-5-mini provider spec
PROVIDER_SPEC_CONTENT=$(cat providers/openai/gpt-5-mini.json)

# Create the request directly for gpt-5-mini Responses API
# Combine system and user messages into a single input string with proper formatting
INPUT_TEXT="System: You are an acclaimed creative writer known for vivid storytelling. Write engaging stories with memorable characters, rich descriptions, and satisfying narrative arcs.

User: Write an original ${STYLE} story about: ${TOPIC}

The story should be ${LENGTH} long. Start with an engaging opening and end with a satisfying conclusion."

# Escape the input text for JSON (this handles newlines, quotes, and all special characters)
ESCAPED_INPUT=$(echo "$INPUT_TEXT" | jq -Rs .)

# Create the provider-specific request for gpt-5-mini
# Note: The Responses API for gpt-5-mini doesn't use temperature/top_p
# It uses reasoning.effort and text.verbosity instead
PROVIDER_REQUEST=$(cat << 'ENDOFJSON'
{
  "model": "gpt-5-mini",
  "input": INPUT_PLACEHOLDER,
  "max_output_tokens": MAX_TOKENS_PLACEHOLDER,
  "reasoning": {
    "effort": "low"
  },
  "text": {
    "verbosity": "medium"
  }
}
ENDOFJSON
)

# Replace placeholders with actual values
PROVIDER_REQUEST="${PROVIDER_REQUEST//INPUT_PLACEHOLDER/$ESCAPED_INPUT}"
PROVIDER_REQUEST="${PROVIDER_REQUEST//MAX_TOKENS_PLACEHOLDER/$MAX_TOKENS}"

# Create the combined request file for the run command
# We need to properly escape the provider spec JSON too
ESCAPED_PROVIDER_SPEC=$(echo "$PROVIDER_SPEC_CONTENT" | jq -c .)

# Create the final request JSON
cat > /tmp/story_request.json << ENDOFJSON
{
  "provider_spec": ${ESCAPED_PROVIDER_SPEC},
  "model_id": "gpt-5-mini",
  "request_body": ${PROVIDER_REQUEST}
}
ENDOFJSON

# Debug: Optionally show the request being sent (uncomment for debugging)
# echo "Debug: Request being sent:"
# jq . /tmp/story_request.json 2>/dev/null || cat /tmp/story_request.json

echo ""
echo -e "${CYAN}✨ Creating your ${STYLE} story about: ${TOPIC}${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Generate story using Specado run command
echo "Generating story using Specado..."
STORY_OUTPUT=$($SPECADO run /tmp/story_request.json \
  --output json 2>&1)

if [[ $? -ne 0 ]]; then
  echo -e "${RED}Story generation failed. Error:${NC}"
  echo "$STORY_OUTPUT"
  exit 1
else
  echo -e "${GREEN}✓ Story generation successful${NC}"
  
  # Extract the story content from the JSON response
  # For gpt-5-mini, the content is in raw_metadata.output[1].content[0].text
  STORY=$(echo "$STORY_OUTPUT" | jq -r '.raw_metadata.output[1].content[0].text // .output[1].content[0].text // .content' 2>/dev/null)
  
  if [ ! -z "$STORY" ] && [ "$STORY" != "null" ]; then
    echo ""
    echo "$STORY" | fold -s -w 80
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo -e "${GREEN}✓ Story generated successfully via Specado!${NC}"
  else
    echo -e "${RED}Failed to extract story from response${NC}"
    echo "Raw response:"
    echo "$STORY_OUTPUT" | jq . 2>/dev/null || echo "$STORY_OUTPUT"
  fi
fi

# Note: lossiness check would require preview command which needs translate implementation
# Skipping lossiness check for now since preview with --show-lossiness is part of L2 translate feature

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
rm -f /tmp/story_request.json