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
    EXTRA_ARGS=""

    set -- {{flags}}
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --)      shift; EXTRA_ARGS="$*"; break ;;
            --fix)   MODE="fix";   shift ;;
            --check) MODE="check"; shift ;;
            *)       shift ;;
        esac
    done

    echo "MODE='$MODE'"
    echo "EXTRA_ARGS='$EXTRA_ARGS'"

[private]
_lint-dispatch workspace target *flags:
    #!/usr/bin/env bash
    set -euo pipefail

    case "{{workspace}}" in
        "")
            echo "Usage: just lint <workspace> <target> [flags]"
            echo ""
            echo "Workspace-targeted (require <target>):"
            echo "  sdk             lint the main workspace"
            echo "  integrations    lint the integrations workspace"
            echo "  all             lint everything"
            echo ""
            echo "  Targets:"
            echo "    rust            cargo fmt check / fix"
            echo "    markdown        markdownlint check / fix"
            echo ""
            echo "  Flags:"
            echo "    --fix           fix linting issues when possible"
            echo ""
            echo "Workspace-orthogonal:"
            echo "  commits [base]  lint commit messages on the current branch"
            echo "                  (base defaults to origin/main, then main)"
            exit 1
            ;;
        commits)
            just _lint-commits "{{target}}"
            ;;
        *)
            just _lint-workspace-dispatch "{{workspace}}" "{{target}}" {{flags}}
            ;;
    esac

[private]
_lint-workspace-dispatch workspace target *flags:
    #!/usr/bin/env bash
    set -euo pipefail

    eval $(just _parse-lint-args {{flags}})

    if [ -z "{{target}}" ]; then
        echo "Error: '{{workspace}}' requires a target (rust|markdown)" >&2
        echo "Run 'just lint' for usage." >&2
        exit 1
    fi

    case "{{target}}" in
        rust)     just _lint-rust "{{workspace}}" "$MODE" "$EXTRA_ARGS" ;;
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
_lint-rust workspace mode extra_args:
    #!/usr/bin/env bash
    set -euo pipefail

    WORKSPACES=$(just _parse-workspace "{{workspace}}")

    for WS in $WORKSPACES; do
        MANIFEST_ARG=$(just _workspace-manifest-arg "$WS")

        if [ "{{mode}}" = "fix" ]; then
            echo "Running cargo fmt --all for workspace '$WS'..."
            cargo fmt --all $MANIFEST_ARG {{extra_args}}
            echo "Rust fmt applied to '$WS'."
        else
            echo "Running cargo fmt --all -- --check for workspace '$WS'..."
            cargo fmt --all $MANIFEST_ARG {{extra_args}} -- --check
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

# -----------------------------------------------------------------------------
# Commits
# -----------------------------------------------------------------------------

IMPERATIVE_CHECK_SCRIPT := justfile_directory() / "internal/scripts/rust/check_imperative_mood.rs"

[private]
_lint-commits base="":
    #!/usr/bin/env bash
    set -euo pipefail

    GUIDELINES_URL='https://tbaggery.com/2008/04/19/a-note-about-git-commit-messages.html'

    if ! which rust-script >/dev/null 2>&1; then
        echo "error: 'rust-script' not found. Run 'cargo install rust-script'" >&2
        exit 1
    fi

    resolve_base() {
        local arg="$1"
        if [ -n "$arg" ]; then
            if ! git rev-parse --verify "$arg" >/dev/null 2>&1; then
                echo "error: base ref '$arg' not found" >&2
                exit 1
            fi
            echo "$arg"
            return
        fi
        if git rev-parse --verify origin/main >/dev/null 2>&1; then
            echo "origin/main"
        elif git rev-parse --verify main >/dev/null 2>&1; then
            echo "main"
        else
            echo "error: no base ref found (tried origin/main, main); pass one as argument" >&2
            exit 1
        fi
    }

    check_imperative() {
        local word="$1"
        rust-script {{IMPERATIVE_CHECK_SCRIPT}} "$word" 2>&1
    }

    check_subject() {
        local subject="$1"
        local violations=()

        if [[ ! "$subject" =~ ^\[#[0-9]+\]\ .+ ]]; then
            violations+=("subject must match: [#<issue-number>] <message>")
        else
            local rest="${subject#*] }"
            local first_char="${rest:0:1}"
            local first_word="${rest%% *}"

            if [[ ! "$first_char" =~ [A-Z] ]]; then
                violations+=("message after [#...] must start with a capital letter (got: '$first_char')")
            fi

            if ! check_imperative "$first_word" >/dev/null; then
                violations+=("message must be in imperative mood (got first word: '$first_word'); see $GUIDELINES_URL")
            fi
        fi

        if [ "${#violations[@]}" -gt 0 ]; then
            printf '%s\n' "${violations[@]}"
        fi
    }

    BASE=$(resolve_base "{{base}}")
    COMMITS=$(git log "$BASE..HEAD" --no-merges --format=%H)

    if [ -z "$COMMITS" ]; then
        echo "no commits between $BASE and HEAD"
        exit 0
    fi

    TOTAL=0
    FAILED=0
    while IFS= read -r SHA; do
        TOTAL=$((TOTAL + 1))
        SHORT=$(git rev-parse --short "$SHA")
        SUBJECT=$(git log -1 --format=%s "$SHA")

        VIOLATIONS=$(check_subject "$SUBJECT")
        if [ -n "$VIOLATIONS" ]; then
            FAILED=$((FAILED + 1))
            echo "✗ $SHORT \"$SUBJECT\""
            while IFS= read -r v; do
                echo "    - $v"
            done <<< "$VIOLATIONS"
        else
            echo "✓ $SHORT \"$SUBJECT\""
        fi
    done <<< "$COMMITS"

    echo ""
    echo "Checked $TOTAL commit(s) between $BASE and HEAD"
    if [ "$FAILED" -gt 0 ]; then
        echo "$FAILED commit(s) failed validation"
        echo ""
        echo "See $GUIDELINES_URL for guidelines"
        exit 1
    fi
    echo "All commits passed"
