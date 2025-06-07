#!/bin/bash

# Script to check Tera templates for common syntax issues

echo "Checking Tera templates for common syntax issues..."

# Ensure we're in the project root directory
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$DIR"

TEMPLATES_DIR="src/templates"
ERROR_COUNT=0

# Check for ternary filter
echo "Checking for ternary filters..."
if grep -r "| ternary(" "$TEMPLATES_DIR"; then
  echo "ERROR: Found ternary filter which is not supported in Tera."
  echo "Replace with {% if condition %}value_if_true{% else %}value_if_false{% endif %}"
  ERROR_COUNT=$((ERROR_COUNT + 1))
fi

# Check for ternary operator
echo "Checking for ternary operators..."
if grep -r "?" "$TEMPLATES_DIR" | grep -E "[^\"']+\s+\?\s+[^\"']+\s+:\s+[^\"']+"; then
  echo "ERROR: Found ternary operator (? :) which is not supported in Tera."
  echo "Replace with {% if condition %}value_if_true{% else %}value_if_false{% endif %}"
  ERROR_COUNT=$((ERROR_COUNT + 1))
fi

# Check for round with parameters
echo "Checking for round filter with parameters..."
if grep -r "| round(" "$TEMPLATES_DIR"; then
  echo "ERROR: Found round filter with parameters which is not supported in Tera."
  echo "Replace with | round (without parameters)"
  ERROR_COUNT=$((ERROR_COUNT + 1))
fi

# Summary
if [ $ERROR_COUNT -eq 0 ]; then
  echo "✅ No common Tera template issues found."
  exit 0
else
  echo "❌ Found $ERROR_COUNT issue(s) in templates. Please fix before running the application."
  exit 1
fi