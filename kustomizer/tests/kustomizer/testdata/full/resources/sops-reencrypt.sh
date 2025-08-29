#!/bin/bash

# Script to decode SOPS files and re-encrypt them using an age key
# Usage: ./sops-reencrypt.sh <age_public_key>

set -euo pipefail

# Check if age public key is provided
if [ $# -ne 1 ]; then
    echo "Usage: $0 <age_public_key>"
    echo "Example: $0 age1ql3z7hjy54pw3hyww5ayyfg7zqgvc7w3j2elw8zmrj2kg5sfn9aqmcac8p"
    exit 1
fi

AGE_PUBLIC_KEY="$1"

# Validate age public key format
if [[ ! "$AGE_PUBLIC_KEY" =~ ^age1[a-z0-9]{58}$ ]]; then
    echo "Error: Invalid age public key format"
    echo "Expected format: age1<58 characters>"
    exit 1
fi

# Check if sops is installed
if ! command -v sops &> /dev/null; then
    echo "Error: sops is not installed"
    echo "Install with: go install go.mozilla.org/sops/v3/cmd/sops@latest"
    exit 1
fi

echo "Re-encrypting SOPS files with age key: $AGE_PUBLIC_KEY"
echo "============================================================"

# Find all .enc.yaml files
SOPS_FILES=$(find . -name "*.enc.yaml" -type f)

if [ -z "$SOPS_FILES" ]; then
    echo "No SOPS encrypted files found (*.enc.yaml)"
    exit 0
fi

# Process each file
for file in $SOPS_FILES; do
    echo "Processing: $file"
    
    # Create backup
    backup_file="${file}.backup.$(date +%s)"
    cp "$file" "$backup_file"
    echo "  Backup created: $backup_file"
    
    # Decrypt to temporary file
    temp_file=$(mktemp)
    
    if sops --decrypt "$file" > "$temp_file" 2>/dev/null; then
        echo "  Successfully decrypted"
        
        # Re-encrypt with age key
        if sops --encrypt --age "$AGE_PUBLIC_KEY" "$temp_file" > "${file}.new" 2>/dev/null; then
            echo "  Successfully re-encrypted with age key"
            
            # Replace original file
            mv "${file}.new" "$file"
            echo "  File updated"
        else
            echo "  Error: Failed to re-encrypt with age key"
            rm -f "${file}.new"
        fi
    else
        echo "  Error: Failed to decrypt (check SOPS configuration)"
    fi
    
    # Clean up temp file
    rm -f "$temp_file"
    echo ""
done

echo "============================================================"
echo "Re-encryption complete!"
echo ""
echo "Next steps:"
echo "1. Update .sops.yaml to use the age key"
echo "2. Verify files can be decrypted with your age private key"
echo "3. Remove backup files when satisfied"
echo ""
echo "To restore from backups if needed:"
echo "find . -name '*.backup.*' -exec bash -c 'mv \"\$1\" \"\${1%.backup.*}\"' _ {} \\;"