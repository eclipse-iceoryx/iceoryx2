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
_test-e2e-dispatch what *flags:
    #!/usr/bin/env bash
    set -euo pipefail

    case "{{what}}" in
        "")
            echo "Usage: just test-e2e <scope>"
            echo ""
            echo "Scope:"
            echo "  integrations-ros2      run end-to-end tests for the ros2 workspace (requires a sourced ROS 2 env)"
            exit 1
            ;;
        integrations-ros2)
            just _test-e2e-integrations-ros2
            ;;
        *)
            echo "Error: unknown scope '{{what}}' (expected: integrations-ros2)" >&2
            exit 1
            ;;
    esac

[private]
_test-e2e-integrations-ros2:
    #!/usr/bin/env bash
    set -euo pipefail

    if [ -z "${ROS_DISTRO:-}" ]; then
        echo "Error: no sourced ROS 2 environment" >&2
        echo "  Source your ROS 2 setup first, e.g.: source /opt/ros/jazzy/setup.bash" >&2
        exit 1
    fi

    # Honor a caller-provided RMW (e.g. the CI matrix), defaulting to Fast DDS otherwise.
    export RMW_IMPLEMENTATION="${RMW_IMPLEMENTATION:-rmw_fastrtps_cpp}"

    just _test-e2e-run-files "{{INTEGRATIONS_ROS2_DIR}}"

# Lists every `*.exp` under `root`
[private]
_test-e2e-find-files root:
    @find "{{root}}" -type f -name '*.exp' -not -path '*/target/*' | sort

# Discovers every `*.exp` under `root` and runs each with `expect`.
[private]
_test-e2e-run-files root:
    #!/usr/bin/env bash
    set -euo pipefail

    C_RED='\033[1;31m'
    C_GREEN='\033[1;32m'
    C_YELLOW='\033[1;33m'
    C_OFF='\033[0m'

    if ! command -v expect &>/dev/null; then
        echo -e "${C_RED}Error: 'expect' is not installed.${C_OFF}" >&2
        exit 1
    fi

    mapfile -t FILES < <(just _test-e2e-find-files "{{root}}")

    if [ "${#FILES[@]}" -eq 0 ]; then
        echo -e "${C_YELLOW}No end-to-end tests found under {{root}}${C_OFF}"
        exit 0
    fi

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

    echo ""
    echo "-----------------------------------"
    echo "Ran ${#FILES[@]} end-to-end test(s), ${#FAILED[@]} failed."
    if [ "${#FAILED[@]}" -ne 0 ]; then
        printf "${C_RED}FAILED${C_OFF} %s\n" "${FAILED[@]}"
        exit 1
    fi
    echo -e "${C_GREEN}All end-to-end tests passed!${C_OFF}"
