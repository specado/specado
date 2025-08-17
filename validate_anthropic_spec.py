#!/usr/bin/env python3
"""Validate Anthropic provider specification against the ProviderSpec schema."""

import json
import sys
from pathlib import Path
import jsonschema

def validate_provider_spec(spec_file: Path, schema_file: Path) -> bool:
    """Validate a provider specification file against the schema."""
    
    # Load the schema
    with open(schema_file, 'r') as f:
        schema = json.load(f)
    
    # Load the specification (JSON)
    with open(spec_file, 'r') as f:
        spec = json.load(f)
    
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
    spec_file = Path("providers/anthropic/claude-opus-4.1.json")
    schema_file = Path("schemas/provider-spec.schema.json")
    
    if validate_provider_spec(spec_file, schema_file):
        sys.exit(0)
    else:
        sys.exit(1)