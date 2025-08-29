#!/usr/bin/env python3

import os
import re
import yaml
import base64
import hashlib
import subprocess
import tempfile
from pathlib import Path

def generate_fake_secret(key_name, original_value):
    """Generate fake secrets based on key patterns"""
    key_lower = key_name.lower()
    
    # Generate deterministic but fake values based on key name
    seed = hashlib.md5(key_name.encode()).hexdigest()[:16]
    
    if 'private_key' in key_lower:
        return f"""-----BEGIN PRIVATE KEY-----
MIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQC+test+fake+key
{seed}+fake+private+key+data+for+testing+purposes+only
-----END PRIVATE KEY-----"""
    elif 'client_id' in key_lower:
        return f"test_client_id_{seed}.googleusercontent.com"
    elif any(pattern in key_lower for pattern in ['api_key', 'access_key']):
        return f"test_api_key_{seed}"
    elif any(pattern in key_lower for pattern in ['secret', 'password', 'pass']):
        return f"test_secret_{seed}"
    elif 'token' in key_lower:
        return f"test_token_{seed}_fake_value"
    elif 'username' in key_lower:
        return f"test_user_{seed[:8]}"
    elif 'url' in key_lower or 'endpoint' in key_lower:
        return "https://test-api.example.com/v1"
    else:
        return f"test_value_{seed[:12]}"

def mangle_data_recursively(data, parent_key=""):
    """Recursively mangle sensitive data in a dictionary"""
    if isinstance(data, dict):
        result = {}
        for key, value in data.items():
            if isinstance(value, (dict, list)):
                result[key] = mangle_data_recursively(value, key)
            elif isinstance(value, str):
                # Check if this looks like a sensitive key
                full_key = f"{parent_key}_{key}".lower() if parent_key else key.lower()
                if any(pattern in full_key for pattern in [
                    'api_key', 'secret', 'password', 'pass', 'token', 
                    'client_secret', 'private_key', 'client_id', 'username'
                ]):
                    result[key] = generate_fake_secret(full_key, value)
                else:
                    result[key] = value
            else:
                result[key] = value
        return result
    elif isinstance(data, list):
        return [mangle_data_recursively(item, parent_key) for item in data]
    else:
        return data

def process_sops_file(file_path):
    """Process a single SOPS file"""
    print(f"Processing: {file_path}")
    
    # Set environment variable for SOPS
    env = os.environ.copy()
    env['SOPS_AGE_KEY_FILE'] = 'test-age-key.txt'
    
    # Decrypt the file
    with tempfile.NamedTemporaryFile(mode='w+', suffix='.yaml', delete=False) as temp_file:
        try:
            result = subprocess.run(
                ['sops', '--decrypt', str(file_path)],
                capture_output=True,
                text=True,
                env=env,
                check=True
            )
            
            # Parse the decrypted YAML
            data = yaml.safe_load(result.stdout)
            
            # Mangle the secrets
            if 'data' in data:
                # Kubernetes Secret format
                data['data'] = mangle_data_recursively(data['data'])
            else:
                # Generic YAML format
                data = mangle_data_recursively(data)
            
            # Write mangled data to temp file
            yaml.dump(data, temp_file, default_flow_style=False)
            temp_file.flush()
            
            # Re-encrypt with SOPS
            subprocess.run(
                ['sops', '--encrypt', '--in-place', temp_file.name],
                env=env,
                check=True
            )
            
            # Replace original file with encrypted mangled version
            subprocess.run(['cp', temp_file.name, str(file_path)], check=True)
            
            print(f"  Secrets mangled and re-encrypted")
            
        except subprocess.CalledProcessError as e:
            print(f"  Error processing file: {e}")
        finally:
            # Clean up temp file
            try:
                os.unlink(temp_file.name)
            except:
                pass

def main():
    print("Mangling secrets in SOPS files for testing...")
    print("=============================================")
    
    # Find all .enc.yaml files
    for file_path in Path('.').rglob('*.enc.yaml'):
        process_sops_file(file_path)
    
    print("=============================================")
    print("Secret mangling complete!")
    print("")
    print("All SOPS files now contain fake/test values instead of real secrets.")

if __name__ == "__main__":
    main()