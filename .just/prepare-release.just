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
# Prepare Release Recipes
# =============================================================================

VERSIONS_FILE      := justfile_directory() / "internal/VERSIONS"
PACKAGE_VERSION_RS := justfile_directory() / "iceoryx2-bb/elementary/src/package_version.rs"

[private]
_parse-prepare-release-args *flags:
    #!/usr/bin/env bash
    set -euo pipefail

    NEW_VERSION=""

    set -- {{flags}}
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --version)   NEW_VERSION="$2";              shift 2 ;;
            --version=*) NEW_VERSION="${1#--version=}"; shift ;;
            *)           shift ;;
        esac
    done

    echo "NEW_VERSION='$NEW_VERSION'"

[private]
_prepare-release-dispatch workspace action *flags:
    #!/usr/bin/env bash
    set -euo pipefail

    eval "$(just _parse-prepare-release-args {{flags}})"

    if [ -z "{{workspace}}" ] || [ -z "{{action}}" ]; then
        echo "Usage: just prepare-release <workspace> <action> [flags]"
        echo ""
        echo "Workspace:"
        echo "  all                  prepare release across all workspaces"
        echo ""
        echo "Action:"
        echo "  versions             update version strings across the repository"
        echo ""
        echo "Flags:"
        echo "  --version <X.Y.Z>    the new version to set"
        exit 1
    fi

    if [ "{{workspace}}" != "all" ]; then
        echo "Error: only 'all' is supported for prepare-release (got: {{workspace}})" >&2
        exit 1
    fi

    case "{{action}}" in
        versions) just _prepare-release-all-versions "$NEW_VERSION" ;;
        *)
            echo "Error: unknown action '{{action}}' (expected: versions)" >&2
            exit 1
            ;;
    esac

[private]
_prepare-release-all-versions new_version:
    #!/usr/bin/env bash
    set -euo pipefail

    if [ -z "{{new_version}}" ]; then
        echo "Error: --version <X.Y.Z> is required" >&2
        exit 1
    fi

    if ! [[ "{{new_version}}" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
        echo "Error: --version must be of the form X.Y.Z (got: {{new_version}})" >&2
        exit 1
    fi

    OLD_VERSION=$(just _release-read-current-version)
    NEW_VERSION="{{new_version}}"

    if [ "$OLD_VERSION" = "$NEW_VERSION" ]; then
        echo "Current version is already '$NEW_VERSION'; nothing to do."
        exit 0
    fi

    echo "Updating iceoryx2 version: $OLD_VERSION -> $NEW_VERSION"

    just _release-update-package-version-rs "$OLD_VERSION" "$NEW_VERSION"
    just _release-update-cargo-tomls       "$OLD_VERSION" "$NEW_VERSION"
    just _release-update-cmake             "$OLD_VERSION" "$NEW_VERSION"
    just _release-update-python            "$OLD_VERSION" "$NEW_VERSION"
    just _release-update-bazel             "$OLD_VERSION" "$NEW_VERSION"
    just _release-update-ros               "$OLD_VERSION" "$NEW_VERSION"
    just _release-update-docs              "$OLD_VERSION" "$NEW_VERSION"
    just _release-verify-no-stale-version  "$OLD_VERSION"
    just _release-write-current-version    "$OLD_VERSION" "$NEW_VERSION"

    echo ""
    echo "Successfully updated iceoryx2 version to '$NEW_VERSION'."
    echo "Note: rebuild with cargo, bazel, and the python bindings to refresh lock files."

# -----------------------------------------------------------------------------
# Helpers
# -----------------------------------------------------------------------------

[private]
_release-read-current-version:
    @grep '^CURRENT:' {{VERSIONS_FILE}} | sed 's/CURRENT: //'

[private]
_release-write-current-version old new:
    #!/usr/bin/env bash
    set -euo pipefail

    sed -i 's/^CURRENT: {{old}}$/CURRENT: {{new}}/g' {{VERSIONS_FILE}}

    if grep -q "^CURRENT: {{old}}$" {{VERSIONS_FILE}}; then
        echo "Error: failed to update 'CURRENT' in {{VERSIONS_FILE}}" >&2
        exit 1
    fi

[private]
_release-update-package-version-rs old new:
    #!/usr/bin/env bash
    set -euo pipefail

    IFS='.' read -r OLD_MAJOR OLD_MINOR OLD_PATCH <<< "{{old}}"
    IFS='.' read -r NEW_MAJOR NEW_MINOR NEW_PATCH <<< "{{new}}"

    sed -i "s/const MAJOR: u16 = ${OLD_MAJOR};/const MAJOR: u16 = ${NEW_MAJOR};/g" {{PACKAGE_VERSION_RS}}
    sed -i "s/const MINOR: u16 = ${OLD_MINOR};/const MINOR: u16 = ${NEW_MINOR};/g" {{PACKAGE_VERSION_RS}}
    sed -i "s/const PATCH: u16 = ${OLD_PATCH};/const PATCH: u16 = ${NEW_PATCH};/g" {{PACKAGE_VERSION_RS}}

[private]
_release-update-cargo-tomls old new:
    #!/usr/bin/env bash
    set -euo pipefail

    # Workspace-level `version = "..."` fields
    sed -i 's/^version = "{{old}}"/version = "{{new}}"/g' \
        Cargo.toml \
        integrations/Cargo.toml \
        examples/nostd/posix/rust/Cargo.toml

    # iceoryx2-* path-dep version pins. Matches lines such as:
    #   iceoryx2-bb-posix     = { version = "X", path = "iceoryx2-bb/posix/" }
    #   iceoryx2              = { version = "X", path = "../iceoryx2/" }
    #   iceoryx2-integrations-zenoh-tunnel-cli = { version = "X", path = "zenoh/tunnel-cli" }
    sed -i -E 's/^(iceoryx2[a-zA-Z0-9_-]* *= *\{ *version = "){{old}}"/\1{{new}}"/g' \
        $(git ls-files '*Cargo.toml')

[private]
_release-update-cmake old new:
    #!/usr/bin/env bash
    set -euo pipefail

    sed -i 's/set(IOX2_VERSION {{old}})/set(IOX2_VERSION {{new}})/g' \
        $(git ls-files '*CMakeLists.txt' '*.cmake')

    sed -i \
        -e 's/find_package(iceoryx2-c {{old}}/find_package(iceoryx2-c {{new}}/g' \
        -e 's/find_package(iceoryx2-cxx {{old}}/find_package(iceoryx2-cxx {{new}}/g' \
        $(git ls-files '*CMakeLists.txt')

[private]
_release-update-python old new:
    @sed -i 's/version = "{{old}}"/version = "{{new}}"/g' $(git ls-files '*pyproject.toml')

[private]
_release-update-bazel old new:
    @sed -i -z 's/name = "iceoryx2",\n    version = "{{old}}"/name = "iceoryx2",\n    version = "{{new}}"/g' MODULE.bazel

[private]
_release-update-ros old new:
    @sed -i 's|    <version>{{old}}|    <version>{{new}}|g' package.xml

[private]
_release-update-docs old new:
    #!/usr/bin/env bash
    set -euo pipefail

    sed -i 's/pip install iceoryx2=={{old}}/pip install iceoryx2=={{new}}/g' \
        examples/python/README.md
    sed -i 's/git checkout v{{old}}/git checkout v{{new}}/g' \
        examples/python/README.md

    sed -i 's|iceoryx2/v{{old}}|iceoryx2/v{{new}}|g' \
        doc/user-documentation/use-iceoryx2-with-zig.md

# -----------------------------------------------------------------------------
# Sanity check
# -----------------------------------------------------------------------------

[private]
_release-verify-no-stale-version old:
    #!/usr/bin/env bash
    set -euo pipefail

    if grep -rF \
        --exclude-dir=.env \
        --exclude-dir=.git \
        --exclude-dir=landing-page \
        --exclude-dir=plots \
        --exclude-dir=release-notes \
        --exclude-dir=target \
        --exclude=Cargo.lock \
        --exclude=Cargo.Bazel.lock \
        --exclude=MODULE.bazel.lock \
        --exclude=CHANGELOG.md \
        --exclude=header.html \
        --exclude=poetry.lock \
        --exclude=VERSIONS \
        "{{old}}" | grep iceoryx2; then

        echo "Error: found stale references to old version '{{old}}'." >&2
        echo "Update the prepare-release recipes to cover the new occurrences." >&2
        exit 1
    fi
