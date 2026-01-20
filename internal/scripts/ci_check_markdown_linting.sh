#!/usr/bin/env bash
# Copyright (c) 2024 Contributors to the Eclipse Foundation
#
# See the NOTICE file(s) distributed with this work for additional
# information regarding copyright ownership.
#
# This program and the accompanying materials are made available under the
# terms of the Apache Software License 2.0 which is available at
# https://www.apache.org/licenses/LICENSE-2.0, or the MIT license
# which is available at https://opensource.org/licenses/MIT.
#
# SPDX-License-Identifier: Apache-2.0 OR MIT

set -e

# Helper functions
get_repo_root() {
    git rev-parse --show-toplevel
}

get_all_md_files() {
    git ls-files '*.md'
}

print_file_list() {
    local action=$1
    local files=$2
    echo "The following Markdown files will be ${action}ed:"
    echo "$files"
    echo "-----------------------------------"
}

# Set defaults
MODE="check"
CHECK_ALL=true
REPO_ROOT=$(get_repo_root)
MARKDOWNLINT_CONFIG="$REPO_ROOT/.markdownlint.yaml"

# Check required tools are installed
if ! command -v markdownlint &>/dev/null; then
    echo "Error: markdownlint-cli is not installed. Please install it using npm:"
    echo "npm install -g markdownlint-cli"
    exit 1
fi

# Check for markdownlint config existence
if [ ! -f "$MARKDOWNLINT_CONFIG" ]; then
    echo "Error: .markdownlint.yaml file not found at $MARKDOWNLINT_CONFIG"
    exit 1
fi

# Parse command line arguments
while [[ "$#" -gt 0 ]]; do
    case $1 in
    --fix) MODE="fix" ;;
    --check) MODE="check" ;;
    --all) CHECK_ALL=true ;;
    *)
        echo "Unknown parameter passed: $1"
        exit 1
        ;;
    esac
    shift
done

echo "Running Markdown lint in $MODE mode..."

# Get the list of files to process
if [ "$CHECK_ALL" = true ]; then
    md_files=$(get_all_md_files)
    echo "Checking all Markdown files in the repository..."
fi

if [ -z "$md_files" ]; then
    echo "No Markdown files to process."
    exit 0
fi

# Print the list of files being processed
print_file_list "$MODE" "$md_files"

# Execute
if [ "$MODE" = "fix" ]; then
    echo "\nRunning markdownlint to fix other issues..."
    echo "$md_files" | xargs markdownlint -c "$MARKDOWNLINT_CONFIG" --fix

    echo "\nMarkdown lint and format fix completed. Please review the changes."
    echo "NOTE: Not all violations may be automatically fixable."
else
    echo "\nRunning markdownlint to check for issues..."
    echo "$md_files" | xargs markdownlint -c "$MARKDOWNLINT_CONFIG"
    echo "\nMarkdown lint check completed successfully."
fi
