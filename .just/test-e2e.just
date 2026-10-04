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
# End-to-End Test Recipes
# =============================================================================

[private]
[no-exit-message]
_parse-test-e2e-args *flags:
    #!/usr/bin/env bash
    set -euo pipefail
    # Keeps a glob passed to --filter from expanding against the working directory.
    set -f

    CATEGORIES=""
    FILTER=""
    HAS_LIST="false"
    HAS_NO_BUILD="false"
    HAS_NO_RUN="false"

    set -- {{flags}}
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --category)   CATEGORIES="$CATEGORIES $2";              shift 2 ;;
            --category=*) CATEGORIES="$CATEGORIES ${1#--category=}"; shift ;;
            --filter)     FILTER="$2";                              shift 2 ;;
            --filter=*)   FILTER="${1#--filter=}";                  shift ;;
            --list)       HAS_LIST="true";                          shift ;;
            --no-build)   HAS_NO_BUILD="true";                      shift ;;
            --no-run)     HAS_NO_RUN="true";                        shift ;;
            *)            echo "Invalid argument '$1'! Try 'just test-e2e --help' for options." >&2; exit 1 ;;
        esac
    done

    if [[ "$HAS_NO_BUILD" == "true" && "$HAS_NO_RUN" == "true" ]]; then
        echo "Error: --no-build and --no-run cannot be used together." >&2
        exit 1
    fi

    echo "CATEGORIES='${CATEGORIES# }'"
    echo "FILTER='$FILTER'"
    echo "HAS_LIST='$HAS_LIST'"
    echo "HAS_NO_BUILD='$HAS_NO_BUILD'"
    echo "HAS_NO_RUN='$HAS_NO_RUN'"

[private]
[no-exit-message]
_test-e2e-dispatch what *flags:
    #!/usr/bin/env bash
    set -euo pipefail

    show_help() {
        echo "Usage: just test-e2e <scope> [flags]"
        echo ""
        echo "Scope:"
        echo "  integrations-ros2      run end-to-end tests for the ros2 workspace (requires a sourced ROS 2 env)"
        echo ""
        echo "Flags:"
        echo "  --category <name>      select tests of this category, repeatable"
        echo "  --filter <glob>        select tests whose file name matches the glob"
        echo "  --list                 list the selected tests instead of running them"
        echo "  --no-build             skip building what the selected tests need"
        echo "  --no-run               build what the selected tests need without running them"
        echo ""
        echo "Test suites:"
        just _test-e2e-suites | just _test-e2e-print-suite-table
        echo ""
        echo "Artifacts:"
        just _test-e2e-artifacts | just _test-e2e-print-artifact-table
    }

    PARSED_ARGS=$(just _parse-test-e2e-args {{flags}}) || exit $?
    eval "$PARSED_ARGS"

    case "{{what}}" in
        "")
            show_help
            exit 1
            ;;
        --help|-h)
            show_help
            exit 0
            ;;
        -*)
            echo "Invalid argument '{{what}}'!"
            echo ""
            show_help
            exit 1
            ;;
        integrations-ros2)
            ;;
        *)
            echo "Error: unknown scope '{{what}}' (expected: integrations-ros2)" >&2
            exit 1
            ;;
    esac

    TESTS=$(just _test-e2e-find-tests "{{what}}" "$CATEGORIES" "$FILTER")

    if [ -z "$TESTS" ]; then
        echo -e "\033[1;33mNo end-to-end tests selected\033[0m"
        exit 0
    fi

    if [ "$HAS_LIST" = "true" ]; then
        echo "$TESTS" | just _test-e2e-print-test-table
        exit 0
    fi

    if [ "$HAS_NO_BUILD" != "true" ]; then
        echo "$TESTS" | just "_test-e2e-build-{{what}}"
    fi
    if [ "$HAS_NO_RUN" != "true" ]; then
        echo "$TESTS" | just "_test-e2e-run-{{what}}"
    fi

# Maps artifacts required by test suites to recipes to build them.
[private]
_test-e2e-artifacts:
    #!/usr/bin/env bash

    # Artifact              Recipe                                  Description
    cat <<'EOF'
    sdk-examples            _test-e2e-build-sdk-examples            Rust examples of the sdk workspace
    sdk-nostd-examples      _test-e2e-build-sdk-nostd-examples      Rust examples of the no_std workspaces
    sdk-cli                 _test-e2e-build-sdk-cli                 iox2 command line tools of the sdk workspace
    sdk-component-tests     _test-e2e-build-sdk-component-tests     Rust component tests of the sdk workspace
    sdk-c-cxx               _test-e2e-build-sdk-c-cxx               C and C++ bindings, examples and tests of the sdk workspace
    sdk-python              _test-e2e-build-sdk-python              Python bindings of the sdk workspace
    zenoh-tunnel            _test-e2e-build-zenoh-tunnel            Link tunnel of the zenoh workspace
    ros2-gateway            _test-e2e-build-ros2-gateway            Link gateway of the ros2 workspace
    ros2-examples           _test-e2e-build-ros2-examples           Rust examples of the ros2 workspace
    ros2-colcon-examples    _setup-integrations-ros2-examples       ROS 2 packages of the ros2 workspace
    EOF

# Formats artifact rows from stdin as a table with a heading.
[private]
_test-e2e-print-artifact-table:
    #!/usr/bin/env bash
    ARTIFACT_WIDTH=24
    RECIPE_WIDTH=40
    { echo "ARTIFACT RECIPE DESCRIPTION"; cat; } \
        | just _test-e2e-print-table $ARTIFACT_WIDTH $RECIPE_WIDTH


# The end-to-end test suites. Each is a directory relative to the repository
# root that is searched for tests, with the artifacts those tests need.
[private]
_test-e2e-suites:
    #!/usr/bin/env bash

    # Scope                     Category          Root                                        Artifacts
    cat <<'EOF'
    sdk                         examples          examples/rust                               sdk-examples
    sdk                         examples-c        examples/c                                  sdk-c-cxx
    sdk                         examples-cxx      examples/cxx                                sdk-c-cxx
    sdk                         examples-python   examples/python                             sdk-python
    sdk                         examples-nostd    examples/nostd                              sdk-nostd-examples
    sdk                         cross-language    examples/cross-language-end-to-end-tests    sdk-examples,sdk-c-cxx,sdk-python
    sdk                         iox2-cli          iceoryx2-cli                                sdk-cli,sdk-examples,zenoh-tunnel
    sdk                         component-tests   component-tests                             sdk-component-tests,sdk-c-cxx
    integrations-zenoh          tunnel-cli        integrations/zenoh/link-tunnel-cli          sdk-cli,sdk-examples,zenoh-tunnel
    integrations-ros2           gateway-cli       integrations/ros2/link-gateway-cli          sdk-cli,ros2-gateway,ros2-examples
    integrations-ros2           examples          integrations/ros2/examples                  ros2-gateway,ros2-examples
    integrations-ros2           examples-colcon   integrations/ros2/colcon/examples           ros2-colcon-examples
    EOF

# Formats suite rows from stdin as a table with a heading.
[private]
_test-e2e-print-suite-table:
    #!/usr/bin/env bash
    SCOPE_WIDTH=28
    CATEGORY_WIDTH=18
    DIRECTORY_WIDTH=44
    { echo "SCOPE CATEGORY DIRECTORY ARTIFACTS"; cat; } \
        | just _test-e2e-print-table $SCOPE_WIDTH $CATEGORY_WIDTH $DIRECTORY_WIDTH

# Formats test rows from stdin as a table with a heading.
[private]
_test-e2e-print-test-table:
    #!/usr/bin/env bash
    SCOPE_WIDTH=28
    CATEGORY_WIDTH=18
    { echo "SCOPE CATEGORY TEST"; cat; } \
        | just _test-e2e-print-table $SCOPE_WIDTH $CATEGORY_WIDTH

# Prints the rows from stdin as columns of the given widths.
[private]
_test-e2e-print-table *widths:
    @awk -v widths="{{widths}}" 'BEGIN { n = split(widths, w, " ") } { line = " "; for (i = 1; i <= n; i++) line = line sprintf(" %-" w[i] "s", $i); for (i = n + 1; i <= NF; i++) line = line " " $i; print line }'


# Finds every `*.exp` in the given scope whose category is listed and whose
# file name matches the filter.
[private]
[no-exit-message]
_test-e2e-find-tests scope categories filter:
    #!/usr/bin/env bash
    set -euo pipefail
    set -f

    SUITES=$(just _test-e2e-suites)
    CATEGORIES="{{categories}}"
    FILTER="{{filter}}"

    # Rejects a scope or category that no suite declares.
    if [ "{{scope}}" != "all" ] && ! echo "$SUITES" | awk '{ print $1 }' | grep -qx "{{scope}}"; then
        echo "Error: unknown scope '{{scope}}'" >&2
        exit 1
    fi
    for CATEGORY in $CATEGORIES; do
        if ! echo "$SUITES" | awk '{ print $2 }' | grep -qx "$CATEGORY"; then
            echo "Error: unknown category '$CATEGORY'" >&2
            exit 1
        fi
    done

    # Walks the roots matching the scope and categories and finds the test files.
    while read -r SCOPE CATEGORY DIRECTORY ARTIFACTS; do
        if [ "{{scope}}" != "all" ] && [ "$SCOPE" != "{{scope}}" ]; then
            continue
        fi
        if [ -n "$CATEGORIES" ] && [[ " $CATEGORIES " != *" $CATEGORY "* ]]; then
            continue
        fi
        find "$DIRECTORY" -type f -name '*.exp' -not -path '*/target/*' | sort | while read -r FILE; do
            if [ -n "$FILTER" ] && [[ "$(basename "$FILE")" != $FILTER ]]; then
                continue
            fi
            echo "$SCOPE $CATEGORY $FILE"
        done
    done <<< "$SUITES"

# Builds what the ros2 tests from stdin need.
[private]
[no-exit-message]
_test-e2e-build-integrations-ros2:
    #!/usr/bin/env bash
    set -euo pipefail

    # Separate selection by their build system.
    TESTS=$(cat)
    CARGO_TESTS=$(echo "$TESTS" | grep -v ' examples-colcon ' || true)
    COLCON_TESTS=$(echo "$TESTS" | grep ' examples-colcon ' || true)

    if [ -n "$CARGO_TESTS" ]; then
        echo "$CARGO_TESTS" | just _test-e2e-setup-integrations-ros2-cargo _test-e2e-build-artifacts
    fi
    if [ -n "$COLCON_TESTS" ]; then
        echo "$COLCON_TESTS" | just _test-e2e-build-artifacts
    fi

# Runs the ros2 tests from stdin.
[private]
[no-exit-message]
_test-e2e-run-integrations-ros2:
    #!/usr/bin/env bash
    set -euo pipefail

    # Separate selection by their build system.
    TESTS=$(cat)
    CARGO_TESTS=$(echo "$TESTS" | grep -v ' examples-colcon ' || true)
    COLCON_TESTS=$(echo "$TESTS" | grep ' examples-colcon ' || true)

    # Run tests for both build systems.
    STATUS=0
    if [ -n "$CARGO_TESTS" ]; then
        echo "$CARGO_TESTS" | just _test-e2e-setup-integrations-ros2-cargo _test-e2e-run-tests || STATUS=1
    fi
    if [ -n "$COLCON_TESTS" ]; then
        echo "$COLCON_TESTS" | just _test-e2e-setup-integrations-ros2-colcon _test-e2e-run-tests || STATUS=1
    fi
    exit $STATUS

# Runs `recipe` on the rows from stdin in the cargo environment of the ros2
# workspace.
[private]
[no-exit-message]
_test-e2e-setup-integrations-ros2-cargo recipe:
    #!/usr/bin/env bash
    set -eo pipefail

    just _ros2-require-sourced
    # Message crates built from source take precedence over the distribution's.
    if [ -f "{{INTEGRATIONS_ROS2_MESSAGES_INSTALL_BASE}}/setup.bash" ]; then
        source "{{INTEGRATIONS_ROS2_MESSAGES_INSTALL_BASE}}/setup.bash"
    fi
    export CARGO_TARGET_DIR="{{INTEGRATIONS_ROS2_CARGO_TARGET_DIR}}"

    # Honor a caller-provided RMW (e.g. the CI matrix), defaulting to Fast DDS otherwise.
    export RMW_IMPLEMENTATION="${RMW_IMPLEMENTATION:-rmw_fastrtps_cpp}"

    just {{recipe}}

# Runs `recipe` on the rows from stdin in the colcon environment of the ros2
# examples.
[private]
[no-exit-message]
_test-e2e-setup-integrations-ros2-colcon recipe:
    #!/usr/bin/env bash
    set -eo pipefail

    if [ ! -f "{{INTEGRATIONS_ROS2_EXAMPLES_INSTALL_BASE}}/setup.bash" ]; then
        echo "Error: the ROS 2 examples are not built" >&2
        echo "  Build them first: just setup integrations-ros2-examples" >&2
        exit 1
    fi

    # Honor a caller-provided RMW (e.g. the CI matrix), defaulting to Fast DDS otherwise.
    export RMW_IMPLEMENTATION="${RMW_IMPLEMENTATION:-rmw_fastrtps_cpp}"

    # Puts the example nodes on the path of `ros2 run`, so the tests themselves
    # stay free of any workspace knowledge.
    source "{{INTEGRATIONS_ROS2_EXAMPLES_INSTALL_BASE}}/setup.bash"
    export CARGO_TARGET_DIR="{{INTEGRATIONS_ROS2_CARGO_TARGET_DIR}}"

    just {{recipe}}

# Builds the artifacts of the tests from stdin, each once, in table order.
[private]
[no-exit-message]
_test-e2e-build-artifacts:
    #!/usr/bin/env bash
    set -euo pipefail

    SELECTED=$(awk '{ print $1, $2 }' | sort -u)

    # Collect the artifacts of every selected scope and category without
    # duplicates.
    ARTIFACTS=$(while read -r SCOPE CATEGORY; do
        just _test-e2e-suites | awk -v scope="$SCOPE" -v category="$CATEGORY" \
            '$1 == scope && $2 == category { print $4 }'
    done <<< "$SELECTED" | tr ',' '\n' | awk '!seen[$0]++')

    for ARTIFACT in $ARTIFACTS; do
        BUILD_RECIPE=$(just _test-e2e-artifacts | awk -v artifact="$ARTIFACT" '$1 == artifact { print $2 }')
        if [ -z "$BUILD_RECIPE" ]; then
            echo "Error: unknown artifact '$ARTIFACT'" >&2
            exit 1
        fi
        echo "Building $ARTIFACT ..."
        just "$BUILD_RECIPE"
    done

[private]
_test-e2e-build-sdk-examples:
    cargo build --examples

[private]
_test-e2e-build-sdk-nostd-examples:
    cd examples/nostd/posix/rust && cargo build --examples
    cd examples/nostd/bare-metal/rust && cargo build --examples --features semihosting

[private]
_test-e2e-build-sdk-cli:
    cargo build --bin iox2 --bin iox2-config --bin iox2-service --bin iox2-node --bin iox2-link

[private]
_test-e2e-build-sdk-component-tests:
    cargo build --bin component-tests_rust

# The C and C++ bindings with their examples and tests.
[private]
_test-e2e-build-sdk-c-cxx:
    #!/usr/bin/env bash
    set -euo pipefail

    cargo build --package iceoryx2-ffi-c
    cmake -S . -B target/ff/cc/build \
        -DRUST_BUILD_ARTIFACT_PATH="$(pwd)/target/debug" \
        -DCMAKE_BUILD_TYPE=Debug \
        -DBUILD_CXX=ON \
        -DBUILD_EXAMPLES=ON \
        -DBUILD_TESTING=ON \
        -DIOX2_FEATURE_FLATBUFFERS=ON
    cmake --build target/ff/cc/build --parallel "$(nproc 2>/dev/null || sysctl -n hw.ncpu)"

[private]
_test-e2e-build-sdk-python:
    poetry --project iceoryx2-ffi/python install
    poetry --project iceoryx2-ffi/python build-into-venv

[private]
_test-e2e-build-zenoh-tunnel:
    cargo build --manifest-path {{INTEGRATIONS_ZENOH_DIR}}/Cargo.toml --bin iox2-link-tunnel-zenoh

# Expects the cargo environment of the ros2 workspace.
[private]
_test-e2e-build-ros2-gateway:
    cargo build --manifest-path {{INTEGRATIONS_ROS2_DIR}}/Cargo.toml --bin iox2-link-gateway-ros2

# Expects the cargo environment of the ros2 workspace.
[private]
_test-e2e-build-ros2-examples:
    cargo build --manifest-path {{INTEGRATIONS_ROS2_DIR}}/Cargo.toml --package iceoryx2-integrations-ros2-examples --examples

# Runs the tests from stdin with `expect`, one `scope category path` row each.
[private]
[no-exit-message]
_test-e2e-run-tests:
    #!/usr/bin/env bash
    set -euo pipefail

    C_RED='\033[1;31m'
    C_GREEN='\033[1;32m'
    C_YELLOW='\033[1;33m'
    C_OFF='\033[0m'

    # Ensure expect is installed
    if ! command -v expect &>/dev/null; then
        echo -e "${C_RED}Error: 'expect' is not installed.${C_OFF}" >&2
        exit 1
    fi

    # Reads the path column.
    mapfile -t FILES < <(awk '{ print $3 }')

    # Execute each discovered test file
    FAILED=()
    for FILE in "${FILES[@]}"; do
        echo -e "${C_YELLOW}RUN${C_OFF} ${FILE}"
        if expect "${FILE}"; then
            echo -e "${C_GREEN}PASS${C_OFF} ${FILE}"
        else
            echo -e "${C_RED}FAIL${C_OFF} ${FILE}"
            FAILED+=("${FILE}")
        fi
    done

    # Summarize results
    echo ""
    echo "-----------------------------------"
    echo "Ran ${#FILES[@]} end-to-end test(s), ${#FAILED[@]} failed."
    if [ "${#FAILED[@]}" -ne 0 ]; then
        printf "${C_RED}FAILED${C_OFF} %s\n" "${FAILED[@]}"
        exit 1
    fi
    echo -e "${C_GREEN}All end-to-end tests passed!${C_OFF}"
