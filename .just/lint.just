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
_parse-lint-args *flags:
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

    echo "MODE='$MODE'"

[private]
_lint-dispatch workspace target *flags:
    #!/usr/bin/env bash
    set -euo pipefail

    eval $(just _parse-lint-args {{flags}})

    if [ -z "{{workspace}}" ] || [ -z "{{target}}" ]; then
        echo "Usage: just lint <workspace> <target> [flags]"
        echo ""
        echo "Workspaces:"
        echo "  sdk             lint the main workspace"
        echo "  integrations    lint the integrations workspace"
        echo "  all             lint everything"
        echo ""
        echo "Targets:"
        echo "  rust            cargo fmt check / fix"
        echo "  markdown        markdownlint check / fix"
        echo ""
        echo "Flags:"
        echo "  --fix           fix linting issues when possible"
        exit 1
    fi

    case "{{target}}" in
        rust)     just _lint-rust "{{workspace}}" "$MODE" ;;
        markdown) just _lint-markdown "{{workspace}}" "$MODE" ;;
        *)
            echo "Error: unknown target '{{target}}' (expected: rust|markdown)" >&2
            exit 1
            ;;
    esac

# -----------------------------------------------------------------------------
# Rust
# -----------------------------------------------------------------------------

[private]
_lint-rust workspace mode:
    #!/usr/bin/env bash
    set -euo pipefail

    WORKSPACES=$(just _parse-workspace "{{workspace}}")

    for WS in $WORKSPACES; do
        case "$WS" in
            sdk)          MANIFEST_ARG="" ;;
            integrations) MANIFEST_ARG="--manifest-path {{INTEGRATIONS_DIR}}/Cargo.toml" ;;
        esac

        if [ "{{mode}}" = "fix" ]; then
            echo "Running cargo fmt --all for workspace '$WS'..."
            cargo fmt --all $MANIFEST_ARG
            echo "Rust fmt applied to '$WS'."
        else
            echo "Running cargo fmt --all -- --check for workspace '$WS'..."
            cargo fmt --all $MANIFEST_ARG -- --check
            echo "Rust fmt check passed for '$WS'."
        fi
    done

# -----------------------------------------------------------------------------
# Markdown
# -----------------------------------------------------------------------------

[private]
_lint-markdown workspace mode:
    #!/usr/bin/env bash
    set -euo pipefail

    just _parse-workspace "{{workspace}}" > /dev/null

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
        echo "No Markdown files to {{mode}} for workspace '{{workspace}}'."
        exit 0
    fi

    echo "Running markdownlint in {{mode}} mode for workspace '{{workspace}}'..."
    echo "$FILES"
    echo "-----------------------------------"

    if [ "{{mode}}" = "fix" ]; then
        echo "$FILES" | xargs markdownlint -c "$CONFIG" --fix
        echo "Markdown lint --fix completed. Review the changes (not all violations are auto-fixable)."
    else
        echo "$FILES" | xargs markdownlint -c "$CONFIG"
        echo "Markdown lint check completed successfully."
    fi
