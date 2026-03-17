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

VERIFY_STD_PROPAGATION_SCRIPT := justfile_directory() / "internal/scripts/rust/verify_std_propagation.rs"

[private]
_verify-dispatch what *flags:
    #!/usr/bin/env bash
    case "{{what}}" in
        "")
            echo "Usage: just verify <what> [flags]"
            echo ""
            echo "  std-propagation workspace    verify std feature propagation for all crates"
            echo "  std-propagation <crate>      verify std feature propagation for a specific crate"
            exit 1
            ;;
        std-propagation) just _verify-std-propagation-dispatch {{flags}} ;;
        *) echo "Unknown verify check: {{what}}"; exit 1 ;;
    esac

[private]
_verify-std-propagation-dispatch target:
    #!/usr/bin/env bash
    case "{{target}}" in
        workspace) just _verify-std-propagation-workspace ;;
        *)         just _verify-std-propagation-package "{{target}}" ;;
    esac

[private]
_verify-std-propagation-package target:
    #!/usr/bin/env bash
    if [[ -f "{{target}}" ]]; then
        rust-script {{VERIFY_STD_PROPAGATION_SCRIPT}} "{{target}}"
    elif [[ -d "{{target}}" ]]; then
        rust-script {{VERIFY_STD_PROPAGATION_SCRIPT}} "{{target}}/Cargo.toml"
    else
        MANIFEST=$(just _find-crate-manifest "{{target}}")
        rust-script {{VERIFY_STD_PROPAGATION_SCRIPT}} "$MANIFEST"
    fi

[private]
_verify-std-propagation-workspace:
    #!/usr/bin/env bash
    set -euo pipefail

    TOMLS=$(find . -name "Cargo.toml" -not -path "./target/*" -not -path "./.git/*" \
        | grep -v "^\./Cargo\.toml$" \
        | sort)

    TOTAL=$(echo "$TOMLS" | wc -l)
    COUNT=0
    FAILED=0

    for toml in $TOMLS; do
        COUNT=$((COUNT + 1))
        OUTPUT=$(rust-script {{VERIFY_STD_PROPAGATION_SCRIPT}} "$toml" 2>&1) || {
            FAILED=$((FAILED + 1))
            echo "[$COUNT/$TOTAL] FAILED: $toml"
            echo "$OUTPUT" | sed 's/^/   /'
            echo ""
        }
    done

    if [ $FAILED -eq 0 ]; then
        echo "All $TOTAL checks passed"
    else
        echo "$FAILED/$TOTAL checks failed"
        exit 1
    fi
