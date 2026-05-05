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
# Verify Recipes
# =============================================================================

[private]
_parse-verify-args *flags:
    #!/usr/bin/env bash
    set -euo pipefail

    PROFILE=""
    HAS_NOSTD="false"
    EXTRA_ARGS=""

    set -- {{flags}}
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --)        shift; EXTRA_ARGS="$*"; break ;;
            --release) PROFILE="--release"; shift ;;
            --no_std)  HAS_NOSTD="true";    shift ;;
            *)         shift ;;
        esac
    done

    echo "PROFILE='$PROFILE'"
    echo "HAS_NOSTD='$HAS_NOSTD'"
    echo "EXTRA_ARGS='$EXTRA_ARGS'"

[private]
_verify-dispatch what tool *flags:
    #!/usr/bin/env bash
    set -euo pipefail

    case "{{what}}" in
        "")
            echo "Usage: just verify <what> <tool> [flags]"
            echo ""
            echo "Workspace-targeted (require <tool>):"
            echo "  sdk                    target the main workspace"
            echo "  integrations           target the integrations workspace"
            echo "  all                    target all workspaces"
            echo "  <package>              target a specific package (std-propagation only)"
            echo ""
            echo "  Tools:"
            echo "    clippy               run cargo clippy"
            echo "    miri                 run cargo miri on allowlisted crates"
            echo "    std-propagation      verify std feature propagation"
            echo ""
            echo "  Flags (clippy):"
            echo "    --release            run in release profile (default: debug)"
            echo "    --no_std             run with --no-default-features (sdk only)"
            echo ""
            echo "Workspace-orthogonal:"
            echo "  commits [base]         verify commit messages on the current branch"
            echo "                         (base defaults to origin/main, then main)"
            exit 1
            ;;
        commits)
            just _verify-commits "{{tool}}"
            ;;
        *)
            just _verify-workspace-dispatch "{{what}}" "{{tool}}" {{flags}}
            ;;
    esac

[private]
_verify-workspace-dispatch what tool *flags:
    #!/usr/bin/env bash
    set -euo pipefail

    eval $(just _parse-verify-args {{flags}})

    if [ -z "{{tool}}" ]; then
        echo "Error: '{{what}}' requires a tool (clippy|miri|std-propagation)" >&2
        echo "Run 'just verify' for usage." >&2
        exit 1
    fi

    case "{{tool}}" in
        clippy)          just _verify-clippy "{{what}}" "$PROFILE" "$HAS_NOSTD" "$EXTRA_ARGS" ;;
        miri)            just _verify-miri "{{what}}" "$EXTRA_ARGS" ;;
        std-propagation) just _verify-std-propagation "{{what}}" ;;
        *)
            echo "Error: unknown tool '{{tool}}' (expected: clippy|miri|std-propagation)" >&2
            exit 1
            ;;
    esac

# -----------------------------------------------------------------------------
# Clippy
# -----------------------------------------------------------------------------

[private]
_verify-clippy workspace profile has_nostd extra_args:
    #!/usr/bin/env bash
    set -euo pipefail

    WORKSPACES=$(just _parse-workspace "{{workspace}}")

    for WS in $WORKSPACES; do
        if [ "$WS" = "integrations" ] && [ "{{has_nostd}}" = "true" ]; then
            echo "Error: the integrations workspace cannot be built with --no_std" >&2
            exit 1
        fi

        MANIFEST_ARG=$(just _workspace-manifest-arg "$WS")

        if [ "{{has_nostd}}" = "true" ]; then
            echo "Running clippy for '$WS' (no_std {{profile}})..."
            cargo clippy --no-deps --workspace --all-targets {{profile}} \
                --no-default-features \
                --exclude iceoryx2-ffi-c \
                --exclude iceoryx2-cli \
                {{extra_args}} \
                -- -D warnings
        else
            echo "Running clippy for '$WS' (std {{profile}})..."
            cargo clippy --no-deps --workspace --all-targets $MANIFEST_ARG {{profile}} {{extra_args}} -- -D warnings
        fi
    done

# -----------------------------------------------------------------------------
# Miri
# -----------------------------------------------------------------------------

# Crates whitelisted for which `cargo miri test` should be run.
# Paths should be relative to repository root.
MIRI_WHITELIST_SDK          := ""
MIRI_WHITELIST_INTEGRATIONS := ""

[private]
_verify-miri workspace extra_args:
    #!/usr/bin/env bash
    set -euo pipefail

    WORKSPACES=$(just _parse-workspace "{{workspace}}")

    DID_RUN=false

    for WS in $WORKSPACES; do
        case "$WS" in
            sdk)          WHITELIST="{{MIRI_WHITELIST_SDK}}" ;;
            integrations) WHITELIST="{{MIRI_WHITELIST_INTEGRATIONS}}" ;;
        esac

        for CRATE in $WHITELIST; do
            echo "Running cargo +nightly miri test under: $CRATE"
            (cd "{{justfile_directory()}}/$CRATE" && cargo +nightly miri test {{extra_args}})
            DID_RUN=true
        done
    done

    if [ "$DID_RUN" = false ]; then
        echo "No whitelisted crates for workspace '{{workspace}}'; nothing to run."
    fi

# -----------------------------------------------------------------------------
# Std propagation
# -----------------------------------------------------------------------------

VERIFY_STD_PROPAGATION_SCRIPT := justfile_directory() / "internal/scripts/rust/verify_std_propagation.rs"

[private]
_verify-std-propagation what:
    #!/usr/bin/env bash
    set -euo pipefail

    case "{{what}}" in
        sdk|integrations|all) just _verify-std-propagation-workspace "{{what}}" ;;
        *)                    just _verify-std-propagation-package "{{what}}" ;;
    esac

[private]
_verify-std-propagation-workspace workspace:
    #!/usr/bin/env bash
    set -euo pipefail

    WORKSPACES=$(just _parse-workspace "{{workspace}}")

    for WS in $WORKSPACES; do
        case "$WS" in
            sdk)
                TOMLS=$(find . -name "Cargo.toml" \
                    -not -path "./target/*" \
                    -not -path "./.git/*" \
                    -not -path "./integrations/*" \
                    | grep -v "^\./Cargo\.toml$" \
                    | sort)
                ;;
            integrations)
                TOMLS=$(find ./integrations -name "Cargo.toml" \
                    -not -path "*/target/*" \
                    | grep -v "^\./integrations/Cargo\.toml$" \
                    | sort)
                ;;
        esac

        TOTAL=$(echo "$TOMLS" | wc -l)
        COUNT=0
        FAILED=0

        echo "Verifying std propagation for '$WS' ($TOTAL crates)..."

        for TOML in $TOMLS; do
            COUNT=$((COUNT + 1))
            OUTPUT=$(rust-script {{VERIFY_STD_PROPAGATION_SCRIPT}} "$TOML" 2>&1) || {
                FAILED=$((FAILED + 1))
                echo "[$COUNT/$TOTAL] FAILED: $TOML"
                echo "$OUTPUT" | sed 's/^/   /'
                echo ""
            }
        done

        if [ $FAILED -eq 0 ]; then
            echo "All $TOTAL checks passed for '$WS'."
        else
            echo "$FAILED/$TOTAL checks failed for '$WS'."
            exit 1
        fi
    done

[private]
_verify-std-propagation-package package:
    #!/usr/bin/env bash
    set -euo pipefail

    if [[ -f "{{package}}" ]]; then
        rust-script {{VERIFY_STD_PROPAGATION_SCRIPT}} "{{package}}"
    elif [[ -d "{{package}}" ]]; then
        rust-script {{VERIFY_STD_PROPAGATION_SCRIPT}} "{{package}}/Cargo.toml"
    else
        MANIFEST=$(just _find-crate-manifest "{{package}}")
        rust-script {{VERIFY_STD_PROPAGATION_SCRIPT}} "$MANIFEST"
    fi

# -----------------------------------------------------------------------------
# Commits
# -----------------------------------------------------------------------------

IMPERATIVE_CHECK_SCRIPT := justfile_directory() / "internal/scripts/rust/check_imperative_mood.rs"

[private]
_verify-commits base="":
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
