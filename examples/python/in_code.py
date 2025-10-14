#!/usr/bin/env python3
"""
Specado Python Example: Define Prompts in Code

Define a prompt specification directly in your code.
"""

import json
from specado import Client


def main():
    # Initialize the client with a friendly provider name and custom model
    client = Client("openai", model="gpt-5")

    print("Executing in-code prompt specification...")
    print("-" * 60)

    response = client.complete_text(
        "Explain what a closure is in JavaScript in one paragraph.",
        system_message="You are a helpful assistant that explains programming concepts clearly.",
        temperature=0.5,
    )

    # Display the results
    print(json.dumps(response, indent=2))


if __name__ == "__main__":
    main()
