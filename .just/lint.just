# Copyright (c) 2026 Contributors to the Eclipse Foundation
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

# =============================================================================
# Lint Recipes
# =============================================================================

[private]
_lint-dispatch workspace *flags:
    #!/usr/bin/env bash
    set -euo pipefail

    case "{{workspace}}" in
        "")
            echo "Usage: just lint <workspace> [flags]"
            echo ""
            echo "  sdk             lint the main workspace"
            echo "  integrations    lint the integrations workspace"
            echo "  all             lint everything"
            echo ""
            echo "Flags:"
            echo "  --fix           fix linting issues when possible"
            exit 1
            ;;
        sdk|integrations|all) ;;
        *)
            echo "Error: unknown workspace '{{workspace}}'" >&2
            exit 1
            ;;
    esac

    just _lint-markdown "{{workspace}}" {{flags}}

# -----------------------------------------------------------------------------
# Markdown
# -----------------------------------------------------------------------------

[private]
_lint-markdown workspace *flags:
    #!/usr/bin/env bash
    set -euo pipefail

    MODE="check"
    set -- {{flags}}
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --fix)   MODE="fix";   shift ;;
            --check) MODE="check"; shift ;;
            *)       shift ;;
        esac
    done

    if ! command -v markdownlint &>/dev/null; then
        echo "Error: markdownlint-cli is not installed." >&2
        echo "  Install with: npm install -g markdownlint-cli" >&2
        exit 1
    fi

    CONFIG="{{justfile_directory()}}/.markdownlint.yaml"
    if [ ! -f "$CONFIG" ]; then
        echo "Error: markdownlint config not found at $CONFIG" >&2
        exit 1
    fi

    case "{{workspace}}" in
        sdk)          FILES=$(git ls-files '*.md' | grep -v '^integrations/' || true) ;;
        integrations) FILES=$(git ls-files '*.md' | grep    '^integrations/' || true) ;;
        all)          FILES=$(git ls-files '*.md') ;;
    esac

    if [ -z "$FILES" ]; then
        echo "No Markdown files to $MODE for workspace '{{workspace}}'."
        exit 0
    fi

    echo "Running markdownlint in $MODE mode for workspace '{{workspace}}'..."
    echo "$FILES"
    echo "-----------------------------------"

    if [ "$MODE" = "fix" ]; then
        echo "$FILES" | xargs markdownlint -c "$CONFIG" --fix
        echo "Markdown lint --fix completed. Review the changes (not all violations are auto-fixable)."
    else
        echo "$FILES" | xargs markdownlint -c "$CONFIG"
        echo "Markdown lint check completed successfully."
    fi
