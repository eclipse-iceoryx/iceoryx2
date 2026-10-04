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
_parse-test-e2e-args *flags:
    #!/usr/bin/env bash
    set -euo pipefail
    # Keeps a glob passed to --filter from expanding against the working directory.
    set -f

    CATEGORIES=""
    FILTER=""
    HAS_LIST="false"

    set -- {{flags}}
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --category)   CATEGORIES="$CATEGORIES $2";              shift 2 ;;
            --category=*) CATEGORIES="$CATEGORIES ${1#--category=}"; shift ;;
            --filter)     FILTER="$2";                              shift 2 ;;
            --filter=*)   FILTER="${1#--filter=}";                  shift ;;
            --list)       HAS_LIST="true";                          shift ;;
            *)            echo "Invalid argument '$1'! Try 'just test-e2e --help' for options." >&2; exit 1 ;;
        esac
    done

    echo "CATEGORIES='${CATEGORIES# }'"
    echo "FILTER='$FILTER'"
    echo "HAS_LIST='$HAS_LIST'"

[private]
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
        echo ""
        echo "Test roots (scope, category, directory):"
        just _test-e2e-roots | just _test-e2e-print-rows
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
        echo "$TESTS" | just _test-e2e-print-rows
        exit 0
    fi

    echo "$TESTS" | just "_test-e2e-{{what}}"

# The directories searched for end-to-end tests relative to the repository
# root.
[private]
_test-e2e-roots:
    #!/usr/bin/env bash

    # Scope                     Category          Root
    cat <<'EOF'
    sdk                         examples          examples/rust
    sdk                         examples-c        examples/c
    sdk                         examples-cxx      examples/cxx
    sdk                         examples-python   examples/python
    sdk                         examples-nostd    examples/nostd
    sdk                         cross-language    examples/cross-language-end-to-end-tests
    sdk                         cli               iceoryx2-cli
    sdk                         component-tests   component-tests
    integrations-zenoh          cli               integrations/zenoh/link-tunnel-cli
    integrations-ros2           cli               integrations/ros2/link-gateway-cli
    integrations-ros2           examples          integrations/ros2/examples
    integrations-ros2           examples-colcon   integrations/ros2/colcon/examples
    EOF

# Prints `scope category path` rows from stdin as aligned columns.
[private]
_test-e2e-print-rows:
    @awk '{ printf "  %-28s %-18s %s\n", $1, $2, $3 }'

# Finds every `*.exp` in the given scope whose category is listed and whose
# file name matches the filter.
[private]
_test-e2e-find-tests scope categories filter:
    #!/usr/bin/env bash
    set -euo pipefail
    set -f

    ROOTS=$(just _test-e2e-roots)
    CATEGORIES="{{categories}}"
    FILTER="{{filter}}"

    # Rejects names that match no root before scanning.
    if [ "{{scope}}" != "all" ] && ! echo "$ROOTS" | awk '{ print $1 }' | grep -qx "{{scope}}"; then
        echo "Error: unknown scope '{{scope}}'" >&2
        exit 1
    fi
    for CATEGORY in $CATEGORIES; do
        if ! echo "$ROOTS" | awk '{ print $2 }' | grep -qx "$CATEGORY"; then
            echo "Error: unknown category '$CATEGORY'" >&2
            exit 1
        fi
    done

    # Walks the roots matching the scope and categories and finds the test files.
    while read -r SCOPE CATEGORY ROOT; do
        if [ "{{scope}}" != "all" ] && [ "$SCOPE" != "{{scope}}" ]; then
            continue
        fi
        if [ -n "$CATEGORIES" ] && [[ " $CATEGORIES " != *" $CATEGORY "* ]]; then
            continue
        fi
        find "$ROOT" -type f -name '*.exp' -not -path '*/target/*' | sort | while read -r FILE; do
            if [ -n "$FILTER" ] && [[ "$(basename "$FILE")" != $FILTER ]]; then
                continue
            fi
            echo "$SCOPE $CATEGORY $FILE"
        done
    done <<< "$ROOTS"

# Runs the ros2 tests from stdin.
[private]
_test-e2e-integrations-ros2:
    #!/usr/bin/env bash
    set -euo pipefail

    # Separate selection by their build system.
    TESTS=$(cat)
    CARGO_TESTS=$(echo "$TESTS" | grep -v ' examples-colcon ' || true)
    COLCON_TESTS=$(echo "$TESTS" | grep ' examples-colcon ' || true)

    # Run tests for both build systems.
    STATUS=0
    if [ -n "$CARGO_TESTS" ]; then
        echo "$CARGO_TESTS" | just _test-e2e-integrations-ros2-cargo || STATUS=1
    fi
    if [ -n "$COLCON_TESTS" ]; then
        echo "$COLCON_TESTS" | just _test-e2e-integrations-ros2-colcon || STATUS=1
    fi
    exit $STATUS

# Runs the ros2 tests from stdin against the cargo target directory.
[private]
_test-e2e-integrations-ros2-cargo:
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

    just _test-e2e-run-tests

# Runs the ros2 tests from stdin against the colcon install space.
[private]
_test-e2e-integrations-ros2-colcon:
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

    just _test-e2e-run-tests

# Runs the tests from stdin with `expect`, one `scope category path` row each.
[private]
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
