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
# Build Recipes
# =============================================================================

[private]
_parse-build-args *flags:
    #!/usr/bin/env bash
    set -euo pipefail

    TOOLCHAIN=""
    TARGET_TRIPLET=""
    HAS_NOSTD="false"
    HAS_TESTS="false"

    set -- {{flags}}
    while [[ $# -gt 0 ]]; do
        case "$1" in
            +*)         TOOLCHAIN="$1";                    shift ;;
            --target=*) TARGET_TRIPLET="${1#--target=}";   shift ;;
            --no_std)   HAS_NOSTD="true";                  shift ;;
            --tests)    HAS_TESTS="true";                  shift ;;
            *)          shift ;;
        esac
    done

    echo "TOOLCHAIN='$TOOLCHAIN'"
    echo "TARGET_TRIPLET='$TARGET_TRIPLET'"
    echo "HAS_NOSTD='$HAS_NOSTD'"
    echo "HAS_TESTS='$HAS_TESTS'"

[private]
_build-dispatch what *flags:
    #!/usr/bin/env bash
    set -euo pipefail

    eval $(just _parse-build-args {{flags}})

    case "{{what}}" in
        "")
            echo "Usage: just build <scope> [flags]"
            echo ""
            echo "Scope:"
            echo "  sdk                    build the main workspace"
            echo "  integrations           build the integrations workspace"
            echo "  <package>              build a specific package"
            echo ""
            echo "Flags:"
            echo "  +<toolchain>           use a specific toolchain"
            echo "  --target=<triplet>     cross-compile for target"
            echo "  --no_std               build without std"
            echo "  --tests                also build tests"
            exit 1
            ;;
        sdk)
            just _build-sdk "$TOOLCHAIN" "$TARGET_TRIPLET" "$HAS_NOSTD" "$HAS_TESTS"
            ;;
        integrations)
            just _build-integrations "$TOOLCHAIN" "$TARGET_TRIPLET" "$HAS_NOSTD" "$HAS_TESTS"
            ;;
        *)
            just _build-package "{{what}}" "$TOOLCHAIN" "$TARGET_TRIPLET" "$HAS_NOSTD"
            ;;
    esac

[private]
_build-sdk toolchain target_triplet has_nostd has_tests:
    #!/usr/bin/env bash
    set -euo pipefail

    # TODO: Move this branch into helper function
    if [ "{{has_nostd}}" = "true" ]; then
        if [ "{{has_tests}}" = "false" ]; then
            # TODO: Update this to build all crates that are no_std
            echo "Error: the sdk workspace cannot be built with --no_std" >&2
            echo "Use 'just build <package> --no_std' to build compatible packages without std" >&2
            exit 1
        fi

        NOSTD_CRATES=$(just _discover-nostd-test-crates)

        CRATE_COUNT=$(echo "$NOSTD_CRATES" | wc -l)
        echo ""
        echo "=========================================="
        echo "Building $CRATE_COUNT no_std test crates"
        echo "=========================================="
        echo ""

        BUILT_COUNT=0
        BUILT_CRATES=()
        for crate in $NOSTD_CRATES; do
            echo "[$((BUILT_COUNT + 1))/$CRATE_COUNT] Building $crate..."
            just _build-package "$crate" "{{toolchain}}" "{{target_triplet}}" "true"
            BUILT_COUNT=$((BUILT_COUNT + 1))
            BUILT_CRATES+=("$crate")
        done

        echo ""
        echo "=========================================="
        echo "Build Summary"
        echo "=========================================="
        echo "Successfully built $BUILT_COUNT no_std test crates:"
        for crate in "${BUILT_CRATES[@]}"; do
            echo "  ✓ $crate"
        done
        echo ""
    else
        TESTS_FLAG=""
        if [ "{{has_tests}}" = "true" ]; then
            TESTS_FLAG="--tests"
        fi

        cargo {{toolchain}} build --workspace $TESTS_FLAG
    fi

[private]
_build-integrations toolchain target_triplet has_nostd has_tests:
    #!/usr/bin/env bash
    set -euo pipefail

    if [ "{{has_nostd}}" = "true" ]; then
        echo "Error: the integrations workspace cannot be built with --no_std" >&2
        exit 1
    fi

    TESTS_FLAG=""
    if [ "{{has_tests}}" = "true" ]; then
        TESTS_FLAG="--tests"
    fi

    cargo {{toolchain}} build --workspace --manifest-path {{INTEGRATIONS_DIR}}/Cargo.toml $TESTS_FLAG

[private]
_build-package package toolchain target_triplet has_nostd:
    #!/usr/bin/env bash
    set -euo pipefail

    if [ "{{has_nostd}}" = "true" ]; then
        TARGET_ARG=""
        if [ -n "{{target_triplet}}" ]; then
            TARGET_ARG="--target {{target_triplet}}"
        else
            HOST_TARGET=$(just _detect-host-target "{{toolchain}}")
            TARGET_ARG="--target $HOST_TARGET"
        fi

        if [[ "{{package}}" == *"-tests-nostd" ]]; then
            RUSTC_BOOTSTRAP=1 RUSTFLAGS="-C panic=abort" \
                cargo {{toolchain}} build \
                    --package {{package}} \
                    --no-default-features \
                    -Zbuild-std=core,alloc \
                    $TARGET_ARG
        else
            cargo {{toolchain}} build --package {{package}} --no-default-features
        fi
    else
        cargo {{toolchain}} build --package {{package}}
    fi
