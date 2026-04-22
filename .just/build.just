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
    PROFILE=""
    HAS_NOSTD="false"
    HAS_TESTS="false"
    HAS_ALL_TARGETS="false"
    EXCLUDES=""

    set -- {{flags}}
    while [[ $# -gt 0 ]]; do
        case "$1" in
            +*)            TOOLCHAIN="$1";                    shift ;;
            --target=*)    TARGET_TRIPLET="${1#--target=}";   shift ;;
            --release)     PROFILE="--release";               shift ;;
            --no_std)      HAS_NOSTD="true";                  shift ;;
            --tests)       HAS_TESTS="true";                  shift ;;
            --all-targets) HAS_ALL_TARGETS="true";            shift ;;
            --exclude)     EXCLUDES="$EXCLUDES $2";           shift 2 ;;
            *)             shift ;;
        esac
    done

    echo "TOOLCHAIN='$TOOLCHAIN'"
    echo "TARGET_TRIPLET='$TARGET_TRIPLET'"
    echo "PROFILE='$PROFILE'"
    echo "HAS_NOSTD='$HAS_NOSTD'"
    echo "HAS_TESTS='$HAS_TESTS'"
    echo "HAS_ALL_TARGETS='$HAS_ALL_TARGETS'"
    echo "EXCLUDES='$EXCLUDES'"

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
            echo "  --release              build in release profile (default: debug)"
            echo "  --no_std               build without std"
            echo "  --tests                also build tests"
            echo "  --all-targets          build libs, bins, examples, tests, benches"
            echo "  --exclude <pkg>        exclude a package from workspace build (repeatable)"
            exit 1
            ;;
        sdk)
            just _build-sdk "$TOOLCHAIN" "$TARGET_TRIPLET" "$PROFILE" "$HAS_NOSTD" "$HAS_TESTS" "$HAS_ALL_TARGETS" "$EXCLUDES"
            ;;
        integrations)
            just _build-integrations "$TOOLCHAIN" "$TARGET_TRIPLET" "$PROFILE" "$HAS_NOSTD" "$HAS_TESTS" "$HAS_ALL_TARGETS" "$EXCLUDES"
            ;;
        *)
            just _build-package "{{what}}" "$TOOLCHAIN" "$TARGET_TRIPLET" "$PROFILE" "$HAS_NOSTD" "$HAS_ALL_TARGETS"
            ;;
    esac

[private]
_build-sdk toolchain target_triplet profile has_nostd has_tests has_all_targets excludes:
    #!/usr/bin/env bash
    set -euo pipefail

    if [ "{{has_nostd}}" = "true" ]; then
        CRATE_COUNT=$(echo "$NOSTD_CRATES" | wc -l)

        echo ""
        echo "=========================================="
        echo "Building $CRATE_COUNT crates"
        echo "=========================================="
        echo ""

        BUILT_COUNT=0
        BUILT_CRATES=()
        for crate in $NOSTD_CRATES; do
            echo "[$((BUILT_COUNT + 1))/$CRATE_COUNT] Building $crate..."
            just _build-package "$crate" "{{toolchain}}" "{{target_triplet}}" "{{profile}}" "true" "false"
            BUILT_COUNT=$((BUILT_COUNT + 1))
            BUILT_CRATES+=("$crate")
        done

        echo ""
        echo "=========================================="
        echo "Build Summary"
        echo "=========================================="
        echo "Successfully built $BUILT_COUNT crates:"
        for crate in "${BUILT_CRATES[@]}"; do
            echo "  ✓ $crate"
        done
        echo ""
    else
        TARGETS_FLAG=""
        if [ "{{has_all_targets}}" = "true" ]; then
            TARGETS_FLAG="--all-targets"
        elif [ "{{has_tests}}" = "true" ]; then
            TARGETS_FLAG="--tests"
        fi

        TARGET_ARG=""
        if [ -n "{{target_triplet}}" ]; then
            TARGET_ARG="--target {{target_triplet}}"
        fi

        EXCLUDE_ARGS=""
        for pkg in {{excludes}}; do
            EXCLUDE_ARGS="$EXCLUDE_ARGS --exclude $pkg"
        done

        cargo {{toolchain}} build --workspace {{profile}} $TARGETS_FLAG $TARGET_ARG $EXCLUDE_ARGS
    fi

[private]
_build-integrations toolchain target_triplet profile has_nostd has_tests has_all_targets excludes:
    #!/usr/bin/env bash
    set -euo pipefail

    if [ "{{has_nostd}}" = "true" ]; then
        echo "Error: the integrations workspace cannot be built with --no_std" >&2
        exit 1
    fi

    TARGETS_FLAG=""
    if [ "{{has_all_targets}}" = "true" ]; then
        TARGETS_FLAG="--all-targets"
    elif [ "{{has_tests}}" = "true" ]; then
        TARGETS_FLAG="--tests"
    fi

    TARGET_ARG=""
    if [ -n "{{target_triplet}}" ]; then
        TARGET_ARG="--target {{target_triplet}}"
    fi

    EXCLUDE_ARGS=""
    for pkg in {{excludes}}; do
        EXCLUDE_ARGS="$EXCLUDE_ARGS --exclude $pkg"
    done

    cargo {{toolchain}} build --workspace --manifest-path {{INTEGRATIONS_DIR}}/Cargo.toml {{profile}} $TARGETS_FLAG $TARGET_ARG $EXCLUDE_ARGS

[private]
_build-package package toolchain target_triplet profile has_nostd has_all_targets:
    #!/usr/bin/env bash
    set -euo pipefail

    WORKSPACE=$(just _find-package-workspace "{{package}}")
    MANIFEST_ARG=""
    if [ "$WORKSPACE" = "integrations" ]; then
        MANIFEST_ARG="--manifest-path {{INTEGRATIONS_DIR}}/Cargo.toml"
    fi

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
                    $MANIFEST_ARG \
                    --no-default-features \
                    -Zbuild-std=core,alloc \
                    {{profile}} \
                    $TARGET_ARG
        else
            cargo {{toolchain}} build --package {{package}} $MANIFEST_ARG --no-default-features {{profile}}
        fi
    else
        TARGETS_FLAG=""
        if [ "{{has_all_targets}}" = "true" ]; then
            TARGETS_FLAG="--all-targets"
        fi

        TARGET_ARG=""
        if [ -n "{{target_triplet}}" ]; then
            TARGET_ARG="--target {{target_triplet}}"
        fi

        cargo {{toolchain}} build --package {{package}} $MANIFEST_ARG {{profile}} $TARGETS_FLAG $TARGET_ARG
    fi
