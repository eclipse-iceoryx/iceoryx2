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
# Documentation Recipes
# =============================================================================

[private]
_doc-dispatch workspace target *flags:
    #!/usr/bin/env bash
    set -euo pipefail

    if [ -z "{{workspace}}" ] || [ -z "{{target}}" ]; then
        echo "Usage: just doc <workspace> <target>"
        echo ""
        echo "Workspace:"
        echo "  sdk             build docs for the main workspace"
        echo "  integrations    build docs for the integrations workspace"
        echo "  all             build docs for all workspaces"
        echo ""
        echo "Target:"
        echo "  rust            build Rust API docs"
        echo "  c               build C API docs"
        echo "  cxx             build C++ API docs"
        echo "  python          build Python API docs"
        exit 1
    fi

    if [ "{{workspace}}" = "integrations" ] && [ "{{target}}" != "rust" ]; then
        echo "Error: the integrations workspace only has Rust docs (given: {{target}})" >&2
        exit 1
    fi

    case "{{target}}" in
        rust)   just _doc-rust "{{workspace}}" ;;
        c)      just _doc-c "{{workspace}}" ;;
        cxx)    just _doc-cxx "{{workspace}}" ;;
        python) just _doc-python "{{workspace}}" ;;
        *)
            echo "Error: unknown target '{{target}}' (expected: rust|c|cxx|python)" >&2
            exit 1
            ;;
    esac

# -----------------------------------------------------------------------------
# Rust
# -----------------------------------------------------------------------------

[private]
_doc-rust workspace:
    #!/usr/bin/env bash
    set -euo pipefail

    WORKSPACES=$(just _parse-workspace "{{workspace}}")

    for WS in $WORKSPACES; do
        case "$WS" in
            sdk)
                cargo doc --no-deps --workspace

                echo ""
                echo "Available at: {{justfile_directory()}}/target/doc/"
                ;;
            integrations)
                cargo doc --no-deps --workspace --manifest-path {{INTEGRATIONS_DIR}}/Cargo.toml

                echo ""
                echo "Available at: {{INTEGRATIONS_DIR}}/target/doc/"
                ;;
        esac
    done

# -----------------------------------------------------------------------------
# C
# -----------------------------------------------------------------------------

[private]
_doc-c workspace:
    #!/usr/bin/env bash
    set -euo pipefail

    just _parse-workspace "{{workspace}}" > /dev/null

    cargo build --release --package iceoryx2-ffi-c
    make -C {{justfile_directory()}}/doc/api/c html

    echo ""
    echo "Available at: {{justfile_directory()}}/doc/api/c/target/html/"

# -----------------------------------------------------------------------------
# C++
# -----------------------------------------------------------------------------

[private]
_doc-cxx workspace:
    #!/usr/bin/env bash
    set -euo pipefail

    just _parse-workspace "{{workspace}}" > /dev/null

    make -C {{justfile_directory()}}/doc/api/cxx html

    echo ""
    echo "Available at: {{justfile_directory()}}/doc/api/cxx/target/html/"

# -----------------------------------------------------------------------------
# Python
# -----------------------------------------------------------------------------

[private]
_doc-python workspace:
    #!/usr/bin/env bash
    set -euo pipefail

    just _parse-workspace "{{workspace}}" > /dev/null

    poetry --project {{justfile_directory()}}/iceoryx2-ffi/python install
    poetry --project {{justfile_directory()}}/iceoryx2-ffi/python build-into-venv
    poetry --project {{justfile_directory()}}/doc/api/python install
    poetry --project {{justfile_directory()}}/doc/api/python build-docs

    echo ""
    echo "Available at: {{justfile_directory()}}/doc/api/python/target/html/"
