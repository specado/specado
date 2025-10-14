#!/usr/bin/env python3
"""
Specado Python Example: Execute a Prompt from a File

Load a prompt from a YAML file and execute it.
"""

import json
import pathlib
from specado import Client


def main():
    # Initialize the client with a friendly provider name
    client = Client("openai")

    # Load the prompt specification from a YAML file
    prompt_path = pathlib.Path(__file__).parent.parent / "prompts" / "summarize_article.yaml"
    print("Executing prompt from:", prompt_path)
    print("-" * 60)

    # Execute the prompt
    response = client.complete_file(prompt_path)

    # Display the results
    print(json.dumps(response, indent=2))


if __name__ == "__main__":
    main()
