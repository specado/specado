#!/usr/bin/env bash
# Setup script for Specado examples
# This script helps you quickly set up and run the examples

set -e

BOLD="\033[1m"
GREEN="\033[0;32m"
BLUE="\033[0;34m"
YELLOW="\033[0;33m"
RED="\033[0;31m"
RESET="\033[0m"

echo -e "${BOLD}Specado Examples Setup${RESET}"
echo "========================================"
echo ""

# Check if API key is set
if [ -z "$OPENAI_API_KEY" ]; then
  # Check if .env exists
  if [ -f ".env" ]; then
    echo -e "${GREEN}✓${RESET} Found .env file"
  else
    echo -e "${YELLOW}⚠ Warning: OPENAI_API_KEY is not set and no .env file found${RESET}"
    echo ""
    echo "You can set it by:"
    echo "  1. cp .env.example .env"
    echo "  2. Edit .env and add your OPENAI_API_KEY"
    echo "  OR"
    echo "  export OPENAI_API_KEY=sk-..."
    echo ""
    read -p "Do you want to create .env now? (y/N) " -n 1 -r
    echo ""
    if [[ $REPLY =~ ^[Yy]$ ]]; then
      if [ -f ".env.example" ]; then
        cp .env.example .env
        echo -e "${GREEN}✓${RESET} Created .env from .env.example"
        echo "Please edit .env and add your API keys, then run this script again"
        exit 0
      else
        echo -e "${RED}✗${RESET} .env.example not found"
        exit 1
      fi
    fi

    read -p "Continue without API key? (y/N) " -n 1 -r
    echo ""
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
      exit 1
    fi
  fi
fi

# Function to check if command exists
command_exists() {
  command -v "$1" >/dev/null 2>&1
}

echo -e "${BLUE}Checking prerequisites...${RESET}"
echo ""

# Check Python
if command_exists python3; then
  PYTHON_VERSION=$(python3 --version 2>&1 | awk '{print $2}')
  echo -e "${GREEN}✓${RESET} Python ${PYTHON_VERSION}"
else
  echo -e "${RED}✗${RESET} Python 3 not found"
  exit 1
fi

# Check Node.js
if command_exists node; then
  NODE_VERSION=$(node --version)
  echo -e "${GREEN}✓${RESET} Node.js ${NODE_VERSION}"
else
  echo -e "${YELLOW}⚠${RESET} Node.js not found (optional for Node examples)"
fi

# Check Rust
if command_exists cargo; then
  RUST_VERSION=$(cargo --version | awk '{print $2}')
  echo -e "${GREEN}✓${RESET} Rust ${RUST_VERSION}"
else
  echo -e "${YELLOW}⚠${RESET} Rust not found (optional for Rust examples)"
fi

# Function to setup Python venv
setup_python_venv() {
  local venv_path="python/.venv"

  if [ ! -d "$venv_path" ]; then
    echo "Creating Python virtual environment..."
    python3 -m venv "$venv_path"
    echo -e "${GREEN}✓${RESET} Virtual environment created"
  else
    echo -e "${GREEN}✓${RESET} Virtual environment already exists"
  fi

  echo "Installing dependencies..."
  source "$venv_path/bin/activate"
  pip install --upgrade pip > /dev/null 2>&1
  pip install -r python/requirements.txt
  deactivate
}

echo ""
echo -e "${BOLD}What would you like to set up?${RESET}"
echo ""
echo "  1) Python examples"
echo "  2) Node.js examples"
echo "  3) Rust examples"
echo "  4) All examples"
echo "  5) Exit"
echo ""
read -p "Enter your choice (1-5): " choice

case $choice in
  1)
    echo ""
    echo -e "${BLUE}Setting up Python examples...${RESET}"
    setup_python_venv
    echo -e "${GREEN}✓ Python setup complete!${RESET}"
    echo ""
    echo "Run examples with:"
    echo "  cd python && source .venv/bin/activate && python from_spec.py"
    echo "  Or use: ./demo.sh python"
    ;;
  2)
    echo ""
    echo -e "${BLUE}Setting up Node.js examples...${RESET}"
    if ! command_exists node; then
      echo -e "${RED}✗${RESET} Node.js not found. Please install Node.js first."
      exit 1
    fi
    cd node
    npm install
    cd ..
    echo -e "${GREEN}✓ Node.js setup complete!${RESET}"
    echo ""
    echo "Run examples with:"
    echo "  cd node && npm run from-spec"
    echo "  cd node && npm run in-code"
    echo "  Or use: ./demo.sh node"
    ;;
  3)
    echo ""
    echo -e "${BLUE}Setting up Rust examples...${RESET}"
    if ! command_exists cargo; then
      echo -e "${RED}✗${RESET} Rust not found. Please install Rust first."
      exit 1
    fi
    echo "No setup needed - Rust examples use Cargo workspaces"
    echo -e "${GREEN}✓ Rust setup complete!${RESET}"
    echo ""
    echo "Run examples with:"
    echo "  cd rust_basic && cargo run"
    echo "  Or use: ./demo.sh rust"
    ;;
  4)
    echo ""
    echo -e "${BLUE}Setting up all examples...${RESET}"
    echo ""

    # Python
    echo "Setting up Python..."
    setup_python_venv
    echo -e "${GREEN}✓ Python ready${RESET}"

    # Node.js (if available)
    if command_exists node; then
      echo "Setting up Node.js..."
      cd node
      npm install
      cd ..
      echo -e "${GREEN}✓ Node.js ready${RESET}"
    else
      echo -e "${YELLOW}⚠ Skipping Node.js (not installed)${RESET}"
    fi

    # Rust (if available)
    if command_exists cargo; then
      echo "Rust examples ready (no setup needed)"
      echo -e "${GREEN}✓ Rust ready${RESET}"
    else
      echo -e "${YELLOW}⚠ Skipping Rust (not installed)${RESET}"
    fi

    echo ""
    echo -e "${GREEN}✓ All examples set up successfully!${RESET}"
    ;;
  5)
    echo "Exiting..."
    exit 0
    ;;
  *)
    echo -e "${RED}Invalid choice${RESET}"
    exit 1
    ;;
esac

echo ""
echo -e "${BOLD}Next steps:${RESET}"
echo "  1. Set your API key (if not done): export OPENAI_API_KEY=sk-..."
echo "  2. Run ./demo.sh to test all examples"
echo "  3. Check out README.md for more details"
echo ""
echo -e "${GREEN}Happy prompting! 🚀${RESET}"
