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
_parse-doc-args *flags:
    #!/usr/bin/env bash
    set -euo pipefail

    LANGUAGE="rust"
    LANG_COUNT=0

    set -- {{flags}}
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --rust)   LANGUAGE="rust";   LANG_COUNT=$((LANG_COUNT + 1)); shift ;;
            --c)      LANGUAGE="c";      LANG_COUNT=$((LANG_COUNT + 1)); shift ;;
            --cxx)    LANGUAGE="cxx";    LANG_COUNT=$((LANG_COUNT + 1)); shift ;;
            --python) LANGUAGE="python"; LANG_COUNT=$((LANG_COUNT + 1)); shift ;;
            *)        shift ;;
        esac
    done

    if [ "$LANG_COUNT" -gt 1 ]; then
        echo "Error: only one language flag may be specified" >&2
        exit 1
    fi

    echo "LANGUAGE='$LANGUAGE'"

[private]
_doc-dispatch workspace *flags:
    #!/usr/bin/env bash
    set -euo pipefail

    eval $(just _parse-doc-args {{flags}})

    case "{{workspace}}" in
        "")
            echo "Usage: just doc <workspace> [flags]"
            echo ""
            echo "  sdk             build docs for the main workspace"
            echo "  integrations    build docs for the integrations workspace"
            echo "  all             build docs for all workspaces"
            echo ""
            echo "Flags:"
            echo "  --rust          build Rust API docs (default)"
            echo "  --c             build C API docs"
            echo "  --cxx           build C++ API docs"
            echo "  --python        build Python API docs"
            exit 1
            ;;
        sdk|integrations|all) ;;
        *)
            echo "Error: unknown workspace '{{workspace}}'" >&2
            exit 1
            ;;
    esac

    if [ "{{workspace}}" = "integrations" ] && [ "$LANGUAGE" != "rust" ]; then
        echo "Error: the integrations workspace only has Rust docs (given: --$LANGUAGE)" >&2
        exit 1
    fi

    case "$LANGUAGE" in
        rust)   just _doc-rust "{{workspace}}" ;;
        c)      just _doc-c ;;
        cxx)    just _doc-cxx ;;
        python) just _doc-python ;;
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
_doc-c:
    #!/usr/bin/env bash
    set -euo pipefail

    cargo build --release --package iceoryx2-ffi-c
    make -C {{justfile_directory()}}/doc/api/c html

    echo ""
    echo "Available at: {{justfile_directory()}}/doc/api/c/target/html/"

# -----------------------------------------------------------------------------
# C++
# -----------------------------------------------------------------------------

[private]
_doc-cxx:
    #!/usr/bin/env bash
    set -euo pipefail

    make -C {{justfile_directory()}}/doc/api/cxx html

    echo ""
    echo "Available at: {{justfile_directory()}}/doc/api/cxx/target/html/"

# -----------------------------------------------------------------------------
# Python
# -----------------------------------------------------------------------------

[private]
_doc-python:
    #!/usr/bin/env bash
    set -euo pipefail

    poetry --project {{justfile_directory()}}/iceoryx2-ffi/python install
    poetry --project {{justfile_directory()}}/iceoryx2-ffi/python build-into-venv
    poetry --project {{justfile_directory()}}/doc/api/python install
    poetry --project {{justfile_directory()}}/doc/api/python build-docs

    echo ""
    echo "Available at: {{justfile_directory()}}/doc/api/python/target/html/"
