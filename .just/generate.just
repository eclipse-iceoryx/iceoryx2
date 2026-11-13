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
# Generate Recipes
# =============================================================================

[private]
[no-exit-message]
_parse-generate-args *flags:
    #!/usr/bin/env bash
    set -euo pipefail

    MODE="write"

    set -- {{flags}}
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --check) MODE="check"; shift ;;
            *)       echo "Invalid argument '$1'! Try 'just generate --help' for options." >&2; exit 1 ;;
        esac
    done

    echo "MODE='$MODE'"

[private]
[no-exit-message]
_generate-dispatch what *flags:
    #!/usr/bin/env bash
    set -euo pipefail

    show_help() {
        echo "Usage: just generate <what> [flags]"
        echo ""
        echo "What:"
        echo "  flatbuffers         every flatc output in the tree, with the flatc of the"
        echo "                      flatbuffers crate version pinned in Cargo.toml"
        echo ""
        echo "Flags:"
        echo "  --check             generate aside and fail on any difference to the tree"
    }

    PARSED_ARGS=$(just _parse-generate-args {{flags}}) || exit $?
    eval "$PARSED_ARGS"

    case "{{what}}" in
        flatbuffers)
            just _generate-flatbuffers "$MODE"
            ;;
        ""|--help|-h)
            show_help
            exit 0
            ;;
        *)
            echo "Invalid argument '{{what}}'!"
            echo ""
            show_help
            exit 1
            ;;
    esac

# -----------------------------------------------------------------------------
# Flatbuffers
# -----------------------------------------------------------------------------

# Generates every flatc output into a temporary tree, then copies it over the
# repository or, in check mode, compares it with what is already in the repository.
[private]
[no-exit-message]
_generate-flatbuffers mode:
    #!/usr/bin/env bash
    set -euo pipefail

    VERSION=$(just _flatbuffers-version)
    FLATC=$(just _flatc "$VERSION")
    export FLATC

    OUT=$(mktemp -d)
    trap 'rm -rf "$OUT"' EXIT

    just _generate-flatbuffers-schemas "$OUT"
    just _generate-flatbuffers-bindings "$OUT"
    just _generate-flatbuffers-python "$OUT"

    if [ "{{mode}}" = "check" ]; then
        just _compare-flatbuffers-generated "$VERSION" "$OUT"
    else
        cp -R "$OUT"/. "{{justfile_directory()}}/"
        echo "Flatbuffers output regenerated with flatc $VERSION."
    fi

# The version of the flatbuffers crate pinned in the workspace manifest.
[private]
_flatbuffers-version:
    #!/usr/bin/env bash
    set -euo pipefail

    sed -n 's/^flatbuffers = { version = "\([^"]*\)".*/\1/p' "{{justfile_directory()}}/Cargo.toml"

# The flatc release binary of flatbuffers `version`. The specified version is
# downloaded if not already available.
[private]
[no-exit-message]
_flatc version:
    #!/usr/bin/env bash
    set -euo pipefail

    FLATC="{{FLATC_CACHE_DIR}}/{{version}}/flatc"
    if [ ! -x "$FLATC" ]; then
        just _download-flatc "{{version}}" "$(dirname "$FLATC")" >&2
    fi
    echo "$FLATC"

# Generates every tracked `.bfbs` from its `.fbs` from within the
# schema's directory.
[private]
[no-exit-message]
_generate-flatbuffers-schemas out:
    #!/usr/bin/env bash
    set -euo pipefail

    cd "{{justfile_directory()}}"
    git ls-files -- '*.bfbs' | while read -r BFBS; do
        DIRECTORY=$(dirname "$BFBS")
        NAME=$(basename "$BFBS" .bfbs)
        if [ ! -f "$DIRECTORY/$NAME.fbs" ]; then
            echo "Error: no schema $DIRECTORY/$NAME.fbs for the tracked $BFBS" >&2
            exit 1
        fi
        (cd "$DIRECTORY" && "$FLATC" -o "{{out}}/$DIRECTORY" --schema --binary "$NAME.fbs")
    done

# Generates every tracked `*_generated.rs` binding from its `.fbs`, beside it
# or in the generated crate's `schema/` for those in its `binding/`.
[private]
[no-exit-message]
_generate-flatbuffers-bindings out:
    #!/usr/bin/env bash
    set -euo pipefail

    cd "{{justfile_directory()}}"
    GENERATED_DIR=$(just _path-in-tree "{{ICEORYX2_BB_FLATBUFFERS_GENERATED_DIR}}")
    git ls-files -- '*_generated.rs' | while read -r BINDINGS; do
        DIRECTORY=$(dirname "$BINDINGS")
        NAME=$(basename "$BINDINGS" _generated.rs)
        SCHEMA_DIRECTORY="$DIRECTORY"
        if [ "$DIRECTORY" = "$GENERATED_DIR/binding" ]; then
            SCHEMA_DIRECTORY="$GENERATED_DIR/schema"
        fi
        if [ ! -f "$SCHEMA_DIRECTORY/$NAME.fbs" ]; then
            echo "Error: no schema $SCHEMA_DIRECTORY/$NAME.fbs for the tracked $BINDINGS" >&2
            exit 1
        fi
        (cd "$SCHEMA_DIRECTORY" && "$FLATC" -o "{{out}}/$DIRECTORY" --rust "$NAME.fbs")
    done

# Generates the python classes of the `Testing` schemas.
[private]
[no-exit-message]
_generate-flatbuffers-python out:
    #!/usr/bin/env bash
    set -euo pipefail

    # Takes the licence header to prepend since it is checked in CI.
    HEADER=$(head -n 11 "{{ICEORYX2_FFI_PYTHON_FLATBUFFER_TYPES_DIR}}/__init__.py")
    PACKAGE_DIR="{{out}}/$(just _path-in-tree "{{ICEORYX2_FFI_PYTHON_FLATBUFFER_TYPES_DIR}}")"

    cd "{{ICEORYX2_BB_FLATBUFFERS_GENERATED_DIR}}/schema"
    SCHEMAS=$(grep -l '^namespace Testing;' ./*.fbs)
    "$FLATC" -o "$PACKAGE_DIR" --python $SCHEMAS

    # Add linter overrides.
    for FILE in "$PACKAGE_DIR"/Testing/*.py; do
        {
            printf '%s\n\n# pylint: skip-file\n# mypy: ignore-errors\n\n' "$HEADER"
            cat "$FILE"
        } > "$FILE.tmp"
        mv "$FILE.tmp" "$FILE"
    done

# Fails when a file generated below `out` with flatc `version` differs from
# the tree or is missing.
[private]
[no-exit-message]
_compare-flatbuffers-generated version out:
    #!/usr/bin/env bash
    set -euo pipefail

    cd "{{justfile_directory()}}"
    STATUS=0
    while IFS= read -r -d '' FILE; do
        TRACKED="${FILE#{{out}}/}"
        if ! cmp -s "$FILE" "$TRACKED"; then
            echo "Out of date: $TRACKED"
            case "$TRACKED" in
                *.bfbs) ;;
                *) diff -u "$TRACKED" "$FILE" || true ;;
            esac
            STATUS=1
        fi
    done < <(find "{{out}}" -type f -print0 | sort -z)

    if [ "$STATUS" -ne 0 ]; then
        echo "Error: flatbuffers output is out of date for flatc {{version}}, run 'just generate flatbuffers'" >&2
    else
        echo "Flatbuffers output is up to date for flatc {{version}}."
    fi
    exit "$STATUS"
