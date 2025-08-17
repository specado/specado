#!/usr/bin/env python3
"""Validate a provider specification against the ProviderSpec schema."""

import json
import yaml
import sys
from pathlib import Path
import jsonschema

def validate_provider_spec(spec_file: Path, schema_file: Path) -> bool:
    """Validate a provider specification file against the schema."""
    
    # Load the schema
    with open(schema_file, 'r') as f:
        schema = json.load(f)
    
    # Load the specification (YAML or JSON)
    with open(spec_file, 'r') as f:
        if spec_file.suffix == '.json':
            spec = json.load(f)
        else:
            spec = yaml.safe_load(f)
    
    try:
        # Validate against schema
        jsonschema.validate(instance=spec, schema=schema)
        print(f"✅ {spec_file} is valid according to the ProviderSpec schema")
        return True
    except jsonschema.exceptions.ValidationError as e:
        print(f"❌ Validation error in {spec_file}:")
        print(f"   Path: {' -> '.join(str(p) for p in e.path)}")
        print(f"   Error: {e.message}")
        return False
    except Exception as e:
        print(f"❌ Unexpected error: {e}")
        return False

if __name__ == "__main__":
    # Validate all GPT-5 model specs
    spec_files = [
        Path("providers/openai/gpt-5.json"),
        Path("providers/openai/gpt-5-mini.json"),
        Path("providers/openai/gpt-5-nano.json")
    ]
    schema_file = Path("schemas/provider-spec.schema.json")
    
    all_valid = True
    for spec_file in spec_files:
        if not validate_provider_spec(spec_file, schema_file):
            all_valid = False
    
    sys.exit(0 if all_valid else 1)