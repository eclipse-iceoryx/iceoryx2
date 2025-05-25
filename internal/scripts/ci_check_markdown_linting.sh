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

get_changed_files() {
    if [ -n "$GITHUB_EVENT_NAME" ]; then
        # We're in GitHub Actions
        if [ "$GITHUB_EVENT_NAME" == "pull_request" ]; then
            gh pr view "$GITHUB_PR_NUMBER" --json files --jq '.files[].path' | grep '\.md$' || true
        elif [ "$GITHUB_EVENT_NAME" == "push" ]; then
            git diff --name-only "$GITHUB_BEFORE" "$GITHUB_AFTER" | grep '\.md$' || true
        else
            echo "Unsupported GitHub event: $GITHUB_EVENT_NAME"
            exit 1
        fi
    else
        # We're running locally
        # Get the name of the current branch
        current_branch=$(git rev-parse --abbrev-ref HEAD)
        
        if [ "$current_branch" = "main" ]; then
            # If we're on main, just check uncommitted changes
            git diff --name-only --diff-filter=ACMRT HEAD
        else
            merge_base=$(git merge-base main HEAD)
            
            {
                git diff --name-only --diff-filter=ACMRT $merge_base...HEAD
                git ls-files --others --exclude-standard  # New files not yet committed
                git diff --name-only --diff-filter=ACMRT  # Uncommitted changes to tracked files
            } | sort -u
        fi
    fi
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
CHECK_ALL=false
REPO_ROOT=$(get_repo_root)
MARKDOWNLINT_CONFIG="$REPO_ROOT/.markdownlint.yaml"

# Check required tools are installed
if ! command -v markdownlint &> /dev/null; then
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
        *) echo "Unknown parameter passed: $1"; exit 1 ;;
    esac
    shift
done

echo "Running Markdown lint in $MODE mode..."

# Get the list of files to process
if [ "$CHECK_ALL" = true ]; then
    md_files=$(get_all_md_files)
    echo "Checking all Markdown files in the repository..."
else
    changed_files=$(get_changed_files)
    md_files=$(echo "$changed_files" | grep -E '\.md$' || true)
    echo "Checking only changed Markdown files..."
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
