#!/bin/bash

FILE="src/main.rs"
REQUIRED_LINE="#![deny(warnings)]"

# Read the first line
FIRST_LINE=$(head -n 1 "$FILE")

# Check if the first line matches exactly and is not commented out
if [[ "$FIRST_LINE" == "$REQUIRED_LINE" ]]; then
    echo "OK: $REQUIRED_LINE is the first line and not commented out."
    exit 0
else
    echo "ERROR: $REQUIRED_LINE must be the first line and not commented out in $FILE."
    exit 1
fi
