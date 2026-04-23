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
# Test Recipes
# =============================================================================

[private]
_parse-test-args *flags:
    #!/usr/bin/env bash
    set -euo pipefail

    TOOLCHAIN=""
    TARGET_TRIPLET=""
    HAS_NOSTD="false"
    HAS_DOC="false"
    EXTRA_ARGS=""

    set -- {{flags}}
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --)         shift; EXTRA_ARGS="$*";            break ;;
            +*)         TOOLCHAIN="$1";                    shift ;;
            --target)   TARGET_TRIPLET="$2";               shift 2 ;;
            --target=*) TARGET_TRIPLET="${1#--target=}";   shift ;;
            --no_std)   HAS_NOSTD="true";                  shift ;;
            --doc)      HAS_DOC="true";                    shift ;;
            *)          shift ;;
        esac
    done

    echo "TOOLCHAIN='$TOOLCHAIN'"
    echo "TARGET_TRIPLET='$TARGET_TRIPLET'"
    echo "HAS_NOSTD='$HAS_NOSTD'"
    echo "HAS_DOC='$HAS_DOC'"
    echo "EXTRA_ARGS='$EXTRA_ARGS'"

[private]
_test-dispatch what *flags:
    #!/usr/bin/env bash
    set -euo pipefail

    eval $(just _parse-test-args {{flags}})

    case "{{what}}" in
        "")
            echo "Usage: just test <scope> [flags]"
            echo ""
            echo "Scope:"
            echo "  sdk                    run all tests for the main workspace"
            echo "  integrations           run all tests for the integrations workspace"
            echo "  <package>              run tests for a specific package"
            echo ""
            echo "Flags:"
            echo "  +<toolchain>           use a specific toolchain"
            echo "  --target <triplet>     cross-compile for target"
            echo "  --no_std               test without std"
            echo "  --doc                  run documentation tests"
            exit 1
            ;;
        sdk)
            just _test-sdk "$TOOLCHAIN" "$TARGET_TRIPLET" "$HAS_NOSTD" "$HAS_DOC" "$EXTRA_ARGS"
            ;;
        integrations)
            just _test-integrations "$TOOLCHAIN" "$TARGET_TRIPLET" "$HAS_NOSTD" "$HAS_DOC" "$EXTRA_ARGS"
            ;;
        *)
            just _test-package "{{what}}" "$TOOLCHAIN" "$TARGET_TRIPLET" "$HAS_NOSTD" "$HAS_DOC" "$EXTRA_ARGS"
            ;;
    esac

[private]
_test-sdk toolchain target_triplet has_nostd has_doc extra_args:
    #!/usr/bin/env bash
    set -euo pipefail

    if [ "{{has_doc}}" = "true" ]; then
        if [ "{{has_nostd}}" = "true" ]; then
            echo "Error: --doc cannot be combined with --no_std" >&2
            exit 1
        fi
        cargo {{toolchain}} test --workspace --doc {{extra_args}}
        exit 0
    fi

    if [ "{{has_nostd}}" = "true" ]; then
        NOSTD_CRATES=$(just _discover-nostd-crates)

        CRATE_COUNT=$(echo "$NOSTD_CRATES" | wc -l)
        echo ""
        echo "=========================================="
        echo "Running $CRATE_COUNT no_std test crates"
        echo "=========================================="
        echo ""

        PASSED=0
        FAILED=0
        PASSED_CRATES=()
        FAILED_CRATES=()

        for crate in $NOSTD_CRATES; do
            echo "[$((PASSED + FAILED + 1))/$CRATE_COUNT] Running tests for $crate..."
            echo "------------------------------------------"

            if just _test-package "$crate" "{{toolchain}}" "{{target_triplet}}" "true" "false" "{{extra_args}}"; then
                echo "✓ $crate PASSED"
                PASSED=$((PASSED + 1))
                PASSED_CRATES+=("$crate")
            else
                echo "✗ $crate FAILED"
                FAILED=$((FAILED + 1))
                FAILED_CRATES+=("$crate")
            fi
            echo ""
        done

        echo "=========================================="
        echo "Test Summary"
        echo "=========================================="
        echo "Total:  $CRATE_COUNT"
        echo "Passed: $PASSED"
        echo "Failed: $FAILED"
        echo ""

        if [ $PASSED -gt 0 ]; then
            echo "Passed crates:"
            for passed_crate in "${PASSED_CRATES[@]}"; do
                echo "  ✓ $passed_crate"
            done
            echo ""
        fi

        if [ $FAILED -gt 0 ]; then
            echo "Failed crates:"
            for failed_crate in "${FAILED_CRATES[@]}"; do
                echo "  ✗ $failed_crate"
            done
            echo ""
            exit 1
        fi
    else
        cargo {{toolchain}} nextest run --workspace {{extra_args}}
    fi

[private]
_test-integrations toolchain target_triplet has_nostd has_doc extra_args:
    #!/usr/bin/env bash
    set -euo pipefail

    if [ "{{has_nostd}}" = "true" ]; then
        echo "Error: the integrations workspace cannot be built with --no_std" >&2
        exit 1
    fi

    MANIFEST_ARG=$(just _workspace-manifest-arg integrations)

    if [ "{{has_doc}}" = "true" ]; then
        cargo {{toolchain}} test --workspace --doc $MANIFEST_ARG {{extra_args}}
    else
        cargo {{toolchain}} nextest run --workspace $MANIFEST_ARG {{extra_args}}
    fi

[private]
_test-package package toolchain target_triplet has_nostd has_doc extra_args:
    #!/usr/bin/env bash
    set -euo pipefail

    if [ "{{has_doc}}" = "true" ]; then
        if [ "{{has_nostd}}" = "true" ]; then
            echo "Error: --doc cannot be combined with --no_std" >&2
            exit 1
        fi
        cargo {{toolchain}} test --package {{package}} --doc {{extra_args}}
        exit 0
    fi

    if [ "{{has_nostd}}" = "true" ]; then
        TARGET_ARG=""
        if [ -n "{{target_triplet}}" ]; then
            TARGET_ARG="--target {{target_triplet}}"
        else
            HOST_TARGET=$(just _detect-host-target "{{toolchain}}")
            TARGET_ARG="--target $HOST_TARGET"
        fi

        if [[ "{{package}}" == *-tests-nostd ]]; then
            TEST_PACKAGE="{{package}}"
        else
            TEST_PACKAGE="{{package}}-tests-nostd"
        fi

        RUSTC_BOOTSTRAP=1 RUSTFLAGS="-C panic=abort" \
            cargo {{toolchain}} run \
                --package "$TEST_PACKAGE" \
                --no-default-features \
                -Zbuild-std=core,alloc \
                $TARGET_ARG \
                {{extra_args}}
    else
        cargo {{toolchain}} nextest run --package {{package}} {{extra_args}}
    fi
