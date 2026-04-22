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

    set -- {{flags}}
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --release) PROFILE="--release"; shift ;;
            --no_std)  HAS_NOSTD="true";    shift ;;
            *)         shift ;;
        esac
    done

    echo "PROFILE='$PROFILE'"
    echo "HAS_NOSTD='$HAS_NOSTD'"

[private]
_verify-dispatch what tool *flags:
    #!/usr/bin/env bash
    set -euo pipefail

    eval $(just _parse-verify-args {{flags}})

    if [ -z "{{what}}" ] || [ -z "{{tool}}" ]; then
        echo "Usage: just verify <what> <tool> [flags]"
        echo ""
        echo "Scope:"
        echo "  sdk                    target the main workspace"
        echo "  integrations           target the integrations workspace"
        echo "  all                    target all workspaces"
        echo "  <package>              target a specific package (std-propagation only)"
        echo ""
        echo "Static Analysis Tools:"
        echo "  clippy                 run cargo clippy"
        echo "  miri                   run cargo miri on allowlisted crates"
        echo "  std-propagation        verify std feature propagation"
        echo ""
        echo "Flags (clippy):"
        echo "  --release              run in release profile (default: debug)"
        echo "  --no_std               run with --no-default-features (sdk only)"
        exit 1
    fi

    case "{{tool}}" in
        clippy)          just _verify-clippy "{{what}}" "$PROFILE" "$HAS_NOSTD" ;;
        miri)            just _verify-miri "{{what}}" ;;
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
_verify-clippy workspace profile has_nostd:
    #!/usr/bin/env bash
    set -euo pipefail

    WORKSPACES=$(just _parse-workspace "{{workspace}}")

    for WS in $WORKSPACES; do
        case "$WS" in
            sdk)          MANIFEST_ARG="" ;;
            integrations)
                if [ "{{has_nostd}}" = "true" ]; then
                    echo "Error: the integrations workspace cannot be built with --no_std" >&2
                    exit 1
                fi
                MANIFEST_ARG="--manifest-path {{INTEGRATIONS_DIR}}/Cargo.toml"
                ;;
        esac

        if [ "{{has_nostd}}" = "true" ]; then
            echo "Running clippy for '$WS' (no_std {{profile}})..."
            cargo clippy --no-deps --workspace --all-targets {{profile}} \
                --no-default-features \
                --exclude iceoryx2-ffi-c \
                --exclude iceoryx2-tunnel-zenoh \
                --exclude iceoryx2-cli \
                -- -D warnings
        else
            echo "Running clippy for '$WS' (std {{profile}})..."
            cargo clippy --no-deps --workspace --all-targets $MANIFEST_ARG {{profile}} -- -D warnings
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
_verify-miri workspace:
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
            (cd "{{justfile_directory()}}/$CRATE" && cargo +nightly miri test)
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
