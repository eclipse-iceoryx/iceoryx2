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
_detect-host-target toolchain:
    #!/usr/bin/env bash
    set -euo pipefail

    if [ -n "{{toolchain}}" ]; then
        rustc {{toolchain}} -vV | sed -n 's|host: ||p'
    else
        rustc -vV | sed -n 's|host: ||p'
    fi

[private]
_find-crate-dir package:
    #!/usr/bin/env bash
    set -euo pipefail
    MANIFEST=$(cargo metadata --format-version=1 --no-deps \
        | jq -r --arg pkg "{{package}}" \
          '.packages[] | select(.name == $pkg) | .manifest_path')
    if [[ -z "$MANIFEST" ]]; then
        echo "Error: crate '{{package}}' not found in workspace" >&2; exit 1
    fi
    dirname "$MANIFEST"

[private]
_find-crate-manifest package:
    @echo "$(just _find-crate-dir '{{package}}')/Cargo.toml"

[private]
_find-crate-src package:
    @echo "$(just _find-crate-dir '{{package}}')/src"
