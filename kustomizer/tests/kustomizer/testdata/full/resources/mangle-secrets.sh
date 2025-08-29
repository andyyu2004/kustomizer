#!/bin/bash

# Script to mangle/anonymize secrets in SOPS files for testing
# This script decrypts SOPS files, replaces sensitive values with fake data, and re-encrypts

set -euo pipefail

export SOPS_AGE_KEY_FILE="test-age-key.txt"

echo "Mangling secrets in SOPS files for testing..."
echo "============================================="

# Function to generate fake secrets based on key patterns
generate_fake_secret() {
    local key="$1"
    
    case "$key" in
        *API_KEY*|*ACCESS_KEY*)
            echo "test_api_key_$(openssl rand -hex 16)"
            ;;
        *SECRET*|*PASSWORD*|*PASS*)
            echo "test_secret_$(openssl rand -hex 12)"
            ;;
        *TOKEN*)
            echo "test_token_$(openssl rand -hex 20)"
            ;;
        *CLIENT_SECRET*)
            echo "test_client_secret_$(openssl rand -hex 16)"
            ;;
        *PRIVATE_KEY*)
            # Generate a fake private key structure
            echo "-----BEGIN PRIVATE KEY-----\nMIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQC+test+fake+key\n-----END PRIVATE KEY-----"
            ;;
        *CLIENT_ID*)
            echo "test_client_id_$(openssl rand -hex 16).googleusercontent.com"
            ;;
        *USERNAME*)
            echo "test_user_$(openssl rand -hex 8)"
            ;;
        *URL*|*ENDPOINT*)
            echo "https://test-api.example.com/v1"
            ;;
        *)
            echo "test_value_$(openssl rand -hex 8)"
            ;;
    esac
}

# Process each SOPS file
for file in $(find . -name "*.enc.yaml" -type f); do
    echo "Processing: $file"
    
    # Decrypt to temporary YAML file
    temp_yaml=$(mktemp --suffix=.yaml)
    sops --decrypt "$file" > "$temp_yaml"
    
    # Create a mangled version
    temp_mangled=$(mktemp --suffix=.yaml)
    
    # Use yq to process the YAML and replace secret values
    if command -v yq &> /dev/null; then
        # Process with yq - walk through all leaf values and replace them
        yq eval-all '
            def mangle_value(key):
                if key | test("(?i)(api_key|access_key|secret|password|pass|token|client_secret|private_key|client_id|username)") then
                    if key | test("(?i)private_key") then
                        "-----BEGIN PRIVATE KEY-----\nMIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQC+test+fake+key\n-----END PRIVATE KEY-----"
                    elif key | test("(?i)client_id") then
                        "test_client_id_" + (now | tostring)[0:16] + ".googleusercontent.com"
                    elif key | test("(?i)(api_key|access_key)") then
                        "test_api_key_" + (now | tostring)[0:16]
                    elif key | test("(?i)(secret|password|pass)") then
                        "test_secret_" + (now | tostring)[0:12]
                    elif key | test("(?i)token") then
                        "test_token_" + (now | tostring)[0:20]
                    elif key | test("(?i)username") then
                        "test_user_" + (now | tostring)[0:8]
                    else
                        "test_value_" + (now | tostring)[0:8]
                    end
                else
                    .
                end;
            
            def process_object:
                if type == "object" then
                    with_entries(.value |= if type == "string" then mangle_value(.key) else process_object end)
                elif type == "array" then
                    map(if type == "object" or type == "array" then process_object else . end)
                else
                    .
                end;
            
            . as $root |
            if $root.data then
                $root | .data |= process_object
            else
                $root | process_object
            end
        ' "$temp_yaml" > "$temp_mangled"
    else
        # Fallback: simple sed replacements for common patterns
        cp "$temp_yaml" "$temp_mangled"
        
        # Replace base64 encoded values (typical in k8s secrets)
        sed -i 's/[A-Za-z0-9+\/]\{20,\}=*/dGVzdF9zZWNyZXRfZmFrZV92YWx1ZQ==/g' "$temp_mangled"
        
        # Replace obvious secret patterns
        sed -i 's/sk-[a-zA-Z0-9]\{40,\}/test_openai_key_fake_value/g' "$temp_mangled"
        sed -i 's/xoxb-[0-9]\{11\}-[0-9]\{11\}-[a-zA-Z0-9]\{24\}/test_slack_token_fake_value/g' "$temp_mangled"
    fi
    
    # Re-encrypt the mangled file
    sops --encrypt "$temp_mangled" > "$file"
    
    # Clean up temp files
    rm -f "$temp_yaml" "$temp_mangled"
    
    echo "  Secrets mangled and re-encrypted"
done

echo "============================================="
echo "Secret mangling complete!"
echo ""
echo "All SOPS files now contain fake/test values instead of real secrets."