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
    BUILD_TYPE=""
    PROFILE=""
    HAS_NOSTD="false"
    HAS_TESTS="false"
    EXTRA_ARGS=""

    set -- {{flags}}
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --)         shift; EXTRA_ARGS="$*";            break ;;
            +*)         TOOLCHAIN="$1";                    shift ;;
            --target)   TARGET_TRIPLET="$2";               shift 2 ;;
            --target=*) TARGET_TRIPLET="${1#--target=}";   shift ;;
            --release)  BUILD_TYPE="--release";            shift ;;
            --profile)  PROFILE="$2";                      shift 2 ;;
            --no_std)   HAS_NOSTD="true";                  shift ;;
            --tests)    HAS_TESTS="true";                  shift ;;
            *)          echo "Invalid argument '$1'! Try 'just build --help' for options." >&2; exit 1 ;;
        esac
    done

    if [[ -n "$BUILD_TYPE" && -n "$PROFILE" ]]; then
        echo "Error: --release and --profile cannot be used together." >&2
        exit 1
    fi

    if [[ -n "$BUILD_TYPE" ]]; then
        PROFILE="release"
    elif [[ -z "$PROFILE" ]]; then
        PROFILE="dev"
    fi

    echo "TOOLCHAIN='$TOOLCHAIN'"
    echo "TARGET_TRIPLET='$TARGET_TRIPLET'"
    echo "PROFILE='$PROFILE'"
    echo "HAS_NOSTD='$HAS_NOSTD'"
    echo "HAS_TESTS='$HAS_TESTS'"
    echo "EXTRA_ARGS='$EXTRA_ARGS'"

[private]
_build-dispatch what *flags:
    #!/usr/bin/env bash
    set -euo pipefail

    show_help() {
        echo "Usage: just build <scope> [flags]"
        echo ""
        echo "Scope:"
        echo "  sdk                    build the main workspace"
        echo "  integrations-zenoh     build the zenoh workspace"
        echo "  integrations-ros2      build the ros2 workspace (requires a sourced ROS 2 env)"
        echo "  <package>              build a specific package"
        echo ""
        echo "Flags:"
        echo "  +<toolchain>           use a specific toolchain"
        echo "  --target <triplet>     cross-compile for target"
        echo "  --release              build in release mode (default: debug)"
        echo "                         note: cannot be used in combination with --profile <profile>"
        echo "  --profile <profile>    build with the specified cargo profile"
        echo "                         note: cannot be used in combination with --release"
        echo "  --no_std               build without std"
        echo "  --tests                also build tests"
        echo "  -- <args>              forward remaining args to cargo"
    }

    PARSED_ARGS=$(just _parse-build-args {{flags}}) || exit $?
    eval "$PARSED_ARGS"

    case "{{what}}" in
        "")
            show_help
            exit 1
            ;;
        -*|+*)
            echo "Invalid argument '{{what}}'!"
            echo ""
            show_help
            exit 1
            ;;
        --help|-h)
            show_help
            exit 0
            ;;
        sdk)
            just _build-sdk "$TOOLCHAIN" "$TARGET_TRIPLET" "$PROFILE" "$HAS_NOSTD" "$HAS_TESTS" "$EXTRA_ARGS"
            ;;
        integrations-zenoh)
            just _build-integrations-zenoh "$TOOLCHAIN" "$TARGET_TRIPLET" "$PROFILE" "$HAS_NOSTD" "$HAS_TESTS" "$EXTRA_ARGS"
            ;;
        integrations-ros2)
            just _build-integrations-ros2 "$TOOLCHAIN" "$TARGET_TRIPLET" "$PROFILE" "$HAS_NOSTD" "$HAS_TESTS" "$EXTRA_ARGS"
            ;;
        *)
            just _build-package "{{what}}" "$TOOLCHAIN" "$TARGET_TRIPLET" "$PROFILE" "$HAS_NOSTD" "$EXTRA_ARGS"
            ;;
    esac

[private]
_build-sdk toolchain target_triplet profile has_nostd has_tests extra_args:
    #!/usr/bin/env bash
    set -euo pipefail

    if [ "{{has_nostd}}" = "true" ]; then
        NOSTD_CRATES=$(just _discover-nostd-crates)

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
            just _build-package "$crate" "{{toolchain}}" "{{target_triplet}}" "{{profile}}" "true" "{{extra_args}}"
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
        TESTS_FLAG=""
        if [ "{{has_tests}}" = "true" ]; then
            TESTS_FLAG="--tests"
        fi

        TARGET_ARG=""
        if [ -n "{{target_triplet}}" ]; then
            TARGET_ARG="--target {{target_triplet}}"
        fi

        cargo {{toolchain}} build --workspace --profile {{profile}} $TESTS_FLAG $TARGET_ARG {{extra_args}}
    fi

[private]
_build-integrations-zenoh toolchain target_triplet profile has_nostd has_tests extra_args:
    #!/usr/bin/env bash
    set -euo pipefail

    if [ "{{has_nostd}}" = "true" ]; then
        echo "Error: the zenoh workspace cannot be built with --no_std" >&2
        exit 1
    fi

    TESTS_FLAG=""
    if [ "{{has_tests}}" = "true" ]; then
        TESTS_FLAG="--tests"
    fi

    TARGET_ARG=""
    if [ -n "{{target_triplet}}" ]; then
        TARGET_ARG="--target {{target_triplet}}"
    fi

    MANIFEST_ARG=$(just _workspace-manifest-arg integrations-zenoh)

    cargo {{toolchain}} build --workspace $MANIFEST_ARG --profile {{profile}} $TESTS_FLAG $TARGET_ARG {{extra_args}}

[private]
_build-integrations-ros2 toolchain target_triplet profile has_nostd has_tests extra_args:
    #!/usr/bin/env bash
    set -euo pipefail

    if [ "{{has_nostd}}" = "true" ]; then
        echo "Error: the ros2 workspace cannot be built with --no_std" >&2
        exit 1
    fi

    TESTS_FLAG=""
    if [ "{{has_tests}}" = "true" ]; then
        TESTS_FLAG="--tests"
    fi

    TARGET_ARG=""
    if [ -n "{{target_triplet}}" ]; then
        TARGET_ARG="--target {{target_triplet}}"
    fi

    MANIFEST_ARG=$(just _workspace-manifest-arg integrations-ros2)

    cargo {{toolchain}} build --workspace $MANIFEST_ARG --profile {{profile}} $TESTS_FLAG $TARGET_ARG {{extra_args}}

[private]
_build-package package toolchain target_triplet profile has_nostd extra_args:
    #!/usr/bin/env bash
    set -euo pipefail

    WORKSPACE=$(just _find-package-workspace "{{package}}")
    MANIFEST_ARG=$(just _workspace-manifest-arg "$WORKSPACE")

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
                    --profile {{profile}} \
                    $TARGET_ARG \
                    {{extra_args}}
        else
            cargo {{toolchain}} build --package {{package}} $MANIFEST_ARG --no-default-features --profile {{profile}} {{extra_args}}
        fi
    else
        TARGET_ARG=""
        if [ -n "{{target_triplet}}" ]; then
            TARGET_ARG="--target {{target_triplet}}"
        fi

        cargo {{toolchain}} build --package {{package}} $MANIFEST_ARG --profile {{profile}} $TARGET_ARG {{extra_args}}
    fi
