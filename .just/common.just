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
# Utility Recipes
# =============================================================================

[private]
_discover-nostd-test-crates:
    #!/usr/bin/env bash
    set -euo pipefail

    if ! which jq &> /dev/null; then
        echo "Error: jq is required but not found on your system" >&2
        echo "" >&2
        echo "Please install jq to use no_std test commands:" >&2
        echo "  • Arch Linux:    sudo pacman -S jq" >&2
        echo "  • Ubuntu/Debian: sudo apt install jq" >&2
        echo "  • macOS:         brew install jq" >&2
        echo "" >&2
        exit 1
    fi

    cargo metadata --format-version=1 --no-deps \
        | jq -r '.packages[].name' \
        | grep -- '-tests-nostd$' \
        | sort

[private]
_discover-nostd-crates:
    #!/usr/bin/env bash
    set -euo pipefail

    NOSTD_TEST_CRATES=$(just _discover-nostd-test-crates)
    ALL_CRATES=$(cargo metadata --format-version=1 --no-deps | jq -r '.packages[].name')

    # For most crates, `<lib>-tests-nostd` tests `<lib>` (strip `-tests-nostd`).
    # For crates whose own name ends in `-tests` (e.g. `iceoryx2-bb-trait-tests`),
    # the nostd test crate is named `<lib>-nostd` and tests `<lib>` itself.
    for test_crate in $NOSTD_TEST_CRATES; do
        if echo "$ALL_CRATES" | grep -qx "${test_crate%-tests-nostd}"; then
            echo "${test_crate%-tests-nostd}"
        elif echo "$ALL_CRATES" | grep -qx "${test_crate%-nostd}"; then
            echo "${test_crate%-nostd}"
        else
            echo "Error: could not resolve base crate for '$test_crate'" >&2
            exit 1
        fi
    done

[private]
_detect-host-target toolchain:
    #!/usr/bin/env bash
    set -euo pipefail

    if [ -n "{{toolchain}}" ]; then
        rustc {{toolchain}} -vV | sed -n 's|host: ||p'
    else
        rustc -vV | sed -n 's|host: ||p'
    fi

[private]
_find-crate-manifest package:
    #!/usr/bin/env bash
    set -euo pipefail

    MANIFEST=$(cargo metadata --format-version=1 --no-deps 2>/dev/null \
        | jq -r --arg pkg "{{package}}" \
          '.packages[] | select(.name == $pkg) | .manifest_path')

    if [[ -z "$MANIFEST" ]]; then
        MANIFEST=$(cargo metadata --format-version=1 --no-deps --manifest-path {{INTEGRATIONS_DIR}}/Cargo.toml 2>/dev/null \
            | jq -r --arg pkg "{{package}}" \
              '.packages[] | select(.name == $pkg) | .manifest_path')
    fi

    if [[ -z "$MANIFEST" ]]; then
        echo "Error: crate '{{package}}' not found in sdk or integrations workspace" >&2
        exit 1
    fi

    echo "$MANIFEST"

[private]
_find-crate-dir package:
    @dirname "$(just _find-crate-manifest '{{package}}')"

[private]
_find-crate-src package:
    @echo "$(just _find-crate-dir '{{package}}')/src"

[private]
_find-package-workspace package:
    #!/usr/bin/env bash
    set -euo pipefail

    MANIFEST=$(just _find-crate-manifest "{{package}}")

    case "$MANIFEST" in
        {{INTEGRATIONS_DIR}}/*) echo "integrations" ;;
        *)                      echo "sdk" ;;
    esac

[private]
_parse-workspace workspace:
    #!/usr/bin/env bash
    case "{{workspace}}" in
        sdk|integrations) echo "{{workspace}}" ;;
        all)              echo "sdk integrations" ;;
        *)
            echo "Error: unknown workspace '{{workspace}}' (expected: sdk|integrations|all)" >&2
            exit 1
            ;;
    esac

[private]
_workspace-manifest-arg workspace:
    #!/usr/bin/env bash
    case "{{workspace}}" in
        sdk)          echo "" ;;
        integrations) echo "--manifest-path {{INTEGRATIONS_DIR}}/Cargo.toml" ;;
        *)
            echo "Error: unknown workspace '{{workspace}}' (expected: sdk|integrations)" >&2
            exit 1
            ;;
    esac
