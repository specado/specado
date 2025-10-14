#!/usr/bin/env bash
# Quick demo script - runs all examples in sequence
# Usage: ./demo.sh [python|node|rust|all]

set -e

BOLD="\033[1m"
GREEN="\033[0;32m"
BLUE="\033[0;34m"
YELLOW="\033[0;33m"
RED="\033[0;31m"
RESET="\033[0m"

DEMO_TYPE="${1:-all}"

echo -e "${BOLD}Specado Examples Demo${RESET}"
echo "========================================"
echo ""

# Check API key
if [ -z "$OPENAI_API_KEY" ]; then
  # Try to load from .env
  if [ -f ".env" ]; then
    # Source .env file, ignoring comments and empty lines
    set -a
    source <(grep -v '^#' .env | grep -v '^$')
    set +a
  fi

  if [ -z "$OPENAI_API_KEY" ]; then
    echo -e "${RED}✗ Error: OPENAI_API_KEY is not set${RESET}"
    echo ""
    echo "Please set your API key:"
    echo "  1. cp .env.example .env"
    echo "  2. Edit .env and add your key"
    echo "  OR"
    echo "  export OPENAI_API_KEY=sk-..."
    echo ""
    exit 1
  fi
fi

run_python_demo() {
  echo -e "${BLUE}Running Python Examples${RESET}"
  echo "----------------------------------------"
  echo ""

  # Check if venv exists
  if [ ! -d "python/.venv" ]; then
    echo -e "${YELLOW}⚠ Virtual environment not found. Creating it now...${RESET}"
    python3 -m venv python/.venv
    source python/.venv/bin/activate
    pip install --upgrade pip > /dev/null 2>&1
    pip install -r python/requirements.txt
    deactivate
    echo -e "${GREEN}✓${RESET} Virtual environment created"
    echo ""
  fi

  # Activate venv
  source python/.venv/bin/activate

  echo -e "${BOLD}1. from_spec.py${RESET} - Load prompt from YAML"
  cd python
  python from_spec.py
  cd ..
  echo ""

  echo -e "${BOLD}2. in_code.py${RESET} - Define prompt in code"
  cd python
  python in_code.py
  cd ..
  echo ""

  # Deactivate venv
  deactivate

  echo -e "${GREEN}✓ Python examples complete${RESET}"
  echo ""
}

run_node_demo() {
  echo -e "${BLUE}Running Node.js Examples${RESET}"
  echo "----------------------------------------"
  echo ""

  echo -e "${BOLD}1. from_spec.js${RESET} - Load prompt from YAML"
  cd node
  node from_spec.js
  cd ..
  echo ""

  echo -e "${BOLD}2. in_code.js${RESET} - Define prompt in code"
  cd node
  node in_code.js
  cd ..
  echo ""
  echo -e "${GREEN}✓ Node.js examples complete${RESET}"
  echo ""
}

run_rust_demo() {
  echo -e "${BLUE}Running Rust Example${RESET}"
  echo "----------------------------------------"
  echo ""

  cd rust_basic
  cargo run --quiet
  cd ..
  echo ""
  echo -e "${GREEN}✓ Rust example complete${RESET}"
  echo ""
}

case "$DEMO_TYPE" in
  python)
    run_python_demo
    ;;
  node)
    if command -v node >/dev/null 2>&1; then
      run_node_demo
    else
      echo -e "${RED}✗ Error: Node.js not found${RESET}"
      echo "Please install Node.js or run: ./demo.sh python"
      exit 1
    fi
    ;;
  rust)
    if command -v cargo >/dev/null 2>&1; then
      run_rust_demo
    else
      echo -e "${RED}✗ Error: Rust not found${RESET}"
      echo "Please install Rust or run: ./demo.sh python"
      exit 1
    fi
    ;;
  all)
    # Python is required
    run_python_demo

    # Node.js is optional
    if command -v node >/dev/null 2>&1; then
      run_node_demo
    else
      echo -e "${YELLOW}⚠ Skipping Node.js examples (node not found)${RESET}"
      echo ""
    fi

    # Rust is optional
    if command -v cargo >/dev/null 2>&1; then
      run_rust_demo
    else
      echo -e "${YELLOW}⚠ Skipping Rust examples (cargo not found)${RESET}"
      echo ""
    fi
    ;;
  *)
    echo -e "${RED}Error: Invalid demo type '${DEMO_TYPE}'${RESET}"
    echo "Usage: ./demo.sh [python|node|rust|all]"
    exit 1
    ;;
esac

echo "========================================"
echo -e "${GREEN}Demo complete! 🎉${RESET}"
echo ""
echo "Next steps:"
echo "  - Modify the prompts in prompts/"
echo "  - Try different providers"
echo "  - Check out README.md for more info"
