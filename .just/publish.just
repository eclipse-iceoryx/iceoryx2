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
# Publish Recipes
# =============================================================================

INTEGRATIONS_DIR := justfile_directory() / "integrations"

SDK_CRATES_TO_PUBLISH := `echo \
    iceoryx2-pal-configuration          \
    iceoryx2-pal-concurrency-sync       \
    iceoryx2-pal-posix                  \
    iceoryx2-pal-print                  \
    iceoryx2-pal-testing                \
    iceoryx2-pal-os-api                 \
    iceoryx2-log-types                  \
    iceoryx2-log                        \
    iceoryx2-bb-print                   \
    iceoryx2-bb-elementary-traits       \
    iceoryx2-bb-concurrency             \
    iceoryx2-bb-loggers                 \
    iceoryx2-bb-elementary              \
    iceoryx2-bb-derive-macros           \
    iceoryx2-bb-container               \
    iceoryx2-bb-system-types            \
    iceoryx2-bb-posix                   \
    iceoryx2-bb-linux                   \
    iceoryx2-bb-lock-free               \
    iceoryx2-bb-testing                 \
    iceoryx2-bb-testing-macros          \
    iceoryx2-bb-threadsafe              \
    iceoryx2-bb-memory                  \
    iceoryx2-cal                        \
    iceoryx2-cal-conformance-tests      \
    iceoryx2                            \
    iceoryx2-conformance-tests          \
    iceoryx2-services-common            \
    iceoryx2-services-discovery         \
    iceoryx2-services-tunnel-backend    \
    iceoryx2-services-tunnel            \
    iceoryx2-services-tunnel-conformance-tests \
    iceoryx2-userland-record-and-replay \
    iceoryx2-cli`

SDK_CRATES_TO_IGNORE := `echo \
    benchmark-event                        \
    benchmark-publish-subscribe            \
    benchmark-request-response             \
    benchmark-queue                        \
    component-tests_rust                   \
    example                                \
    iceoryx2-ffi-c                         \
    iceoryx2-ffi-macros                    \
    iceoryx2-ffi-python                    \
    iceoryx2-services-tunnel-end-to-end-tests \
    iceoryx2-bb-trait-tests                \
    iceoryx2-bb-elementary-tests-common    \
    iceoryx2-bb-elementary-tests-nostd     \
    iceoryx2-bb-threadsafe-tests-common    \
    iceoryx2-bb-threadsafe-tests-nostd     \
    iceoryx2-bb-concurrency-tests-common   \
    iceoryx2-bb-concurrency-tests-nostd    \
    iceoryx2-bb-linux-tests-common         \
    iceoryx2-bb-linux-tests-nostd          \
    iceoryx2-bb-lock-free-tests-common     \
    iceoryx2-bb-lock-free-tests-nostd      \
    iceoryx2-bb-memory-tests-common        \
    iceoryx2-bb-memory-tests-nostd         \
    iceoryx2-bb-container-tests-common     \
    iceoryx2-bb-container-tests-nostd      \
    iceoryx2-bb-derive-macros-tests-common \
    iceoryx2-bb-derive-macros-tests-nostd  \
    iceoryx2-bb-trait-tests-common         \
    iceoryx2-bb-trait-tests-nostd          \
    iceoryx2-bb-system-types-tests-common  \
    iceoryx2-bb-system-types-tests-nostd   \
    iceoryx2-bb-posix-tests-common         \
    iceoryx2-bb-posix-tests-nostd          \
    iceoryx2-cal-conformance-tests-common  \
    iceoryx2-cal-conformance-tests-nostd   \
    iceoryx2-cal-tests-common              \
    iceoryx2-cal-tests-nostd               \
    iceoryx2-conformance-tests-common      \
    iceoryx2-conformance-tests-nostd       \
    iceoryx2-tests-common                  \
    iceoryx2-tests-nostd`

INTEGRATIONS_CRATES_TO_PUBLISH := `echo \
    iceoryx2-integrations-zenoh-tunnel-backend \
    iceoryx2-integrations-zenoh-tunnel-cli`

INTEGRATIONS_CRATES_TO_IGNORE := `echo \
    iceoryx2                                \
    iceoryx2-bb-concurrency                 \
    iceoryx2-bb-posix                       \
    iceoryx2-bb-testing                     \
    iceoryx2-bb-testing-macros              \
    iceoryx2-log                            \
    iceoryx2-services-common                \
    iceoryx2-services-tunnel                \
    iceoryx2-services-tunnel-backend        \
    iceoryx2-services-tunnel-conformance-tests`

[private]
_publish-dispatch workspace *flags:
    #!/usr/bin/env bash
    set -euo pipefail

    DRY_RUN="false"
    ALLOW_DIRTY="false"
    SANITY_CHECKS="false"
    LIST="false"
    NO_VERIFY="false"
    REGISTRY=""

    set -- {{flags}}
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --dry-run)       DRY_RUN="true";       shift ;;
            --allow-dirty)   ALLOW_DIRTY="true";    shift ;;
            --sanity-checks) SANITY_CHECKS="true";  shift ;;
            --list)          LIST="true";            shift ;;
            --no-verify)     NO_VERIFY="true";       shift ;;
            --registry)      REGISTRY="$2";         shift 2 ;;
            *)               echo "Unknown flag: $1"; exit 1 ;;
        esac
    done

    if [ -z "{{workspace}}" ]; then
        echo "Usage: just publish <workspace> [flags]"
        echo ""
        echo "Workspace:"
        echo "  sdk             publish the main workspace crates"
        echo "  integrations    publish the integrations workspace crates"
        echo "  all             publish all workspaces in order"
        echo ""
        echo "Flags:"
        echo "  --dry-run          simulate publishing (requires Rust >= 1.90)"
        echo "  --allow-dirty      allow dirty working directory"
        echo "  --sanity-checks    run sanity checks only"
        echo "  --list             list crates to publish"
        echo "  --no-verify        skip build verification during publishing"
        echo "  --registry <name>  publish to a custom registry configured in ~/.cargo/config.toml"
        exit 1
    fi

    if [ "$LIST" = "true" ]; then
        just _publish-list "{{workspace}}"
    elif [ "$SANITY_CHECKS" = "true" ]; then
        just _publish-sanity-checks "{{workspace}}"
    else
        just _publish "{{workspace}}" "$DRY_RUN" "$ALLOW_DIRTY" "$NO_VERIFY" "$REGISTRY"
    fi

# -----------------------------------------------------------------------------
# Helpers
# -----------------------------------------------------------------------------

[private]
_publish-crates-to-publish workspace:
    #!/usr/bin/env bash
    case "{{workspace}}" in
        sdk)          echo "{{SDK_CRATES_TO_PUBLISH}}" ;;
        integrations) echo "{{INTEGRATIONS_CRATES_TO_PUBLISH}}" ;;
    esac

[private]
_publish-crates-to-ignore workspace:
    #!/usr/bin/env bash
    case "{{workspace}}" in
        sdk)          echo "{{SDK_CRATES_TO_IGNORE}}" ;;
        integrations) echo "{{INTEGRATIONS_CRATES_TO_IGNORE}}" ;;
    esac

# -----------------------------------------------------------------------------
# List
# -----------------------------------------------------------------------------

[private]
_publish-list workspace:
    #!/usr/bin/env bash
    set -euo pipefail

    WORKSPACES=$(just _parse-workspace "{{workspace}}")

    for WS in $WORKSPACES; do
        PUBLISH=$(just _publish-crates-to-publish "$WS")

        echo "Crates to publish ($WS):"
        for CRATE in $PUBLISH; do
            echo "  $CRATE"
        done
    done

# -----------------------------------------------------------------------------
# Sanity checks
# -----------------------------------------------------------------------------

[private]
_publish-sanity-checks workspace:
    #!/usr/bin/env bash
    set -euo pipefail

    WORKSPACES=$(just _parse-workspace "{{workspace}}")

    for WS in $WORKSPACES; do
        echo "Running sanity checks for '$WS'..."
        just _publish-check-completeness "$WS"
        just _publish-check-cyclic-deps "$WS"
        echo "All sanity checks passed."
    done

[private]
_publish-check-completeness workspace:
    #!/usr/bin/env bash
    set -euo pipefail

    MANIFEST_FLAG=$(just _workspace-manifest-arg "{{workspace}}")

    WORKSPACE_CRATES=$(cargo tree --workspace --depth 1 --prefix none $MANIFEST_FLAG \
        | grep -v '(\*)' \
        | grep -E '^(iceoryx2|benchmark|example)' \
        | awk '{print $1}' \
        | sort | uniq)

    PUBLISH=$(just _publish-crates-to-publish "{{workspace}}")
    IGNORE=$(just _publish-crates-to-ignore "{{workspace}}")

    HAS_ERROR=false
    for CRATE in $WORKSPACE_CRATES; do
        if [[ -n "$IGNORE" ]] && [[ " $IGNORE " =~ " ${CRATE} " ]]; then
            continue
        fi
        if [[ " $PUBLISH " =~ " ${CRATE} " ]]; then
            continue
        fi
        echo "ERROR: Crate '$CRATE' is neither in the publish nor ignore list for '{{workspace}}'."
        HAS_ERROR=true
    done

    if [ "$HAS_ERROR" = true ]; then
        exit 1
    fi

[private]
_publish-check-cyclic-deps workspace:
    #!/usr/bin/env bash
    set -euo pipefail

    MANIFEST_FLAG=$(just _workspace-manifest-arg "{{workspace}}")
    PUBLISH=$(just _publish-crates-to-publish "{{workspace}}")

    ALLOWED=""
    HAS_ERROR=false

    for CRATE in $PUBLISH; do
        ALLOWED="$ALLOWED $CRATE"

        DEPS=$(cargo tree --package "$CRATE" --depth 1 --prefix none --edges normal $MANIFEST_FLAG \
            | grep -v '(\*)' \
            | grep -e '^iceoryx2' \
            | awk '{print $1}' \
            | sort | uniq)

        for DEP in $DEPS; do
            if [[ ! " $PUBLISH " =~ " ${DEP} " ]]; then
                # Dependency is not in this workspace's publish list (e.g. SDK crate)
                continue
            fi
            if [[ " $ALLOWED " =~ " ${DEP} " ]]; then
                continue
            fi
            echo "ERROR: '$CRATE' dependency '$DEP' is out of order for publishing!"
            HAS_ERROR=true
        done
    done

    if [ "$HAS_ERROR" = true ]; then
        exit 1
    fi

# -----------------------------------------------------------------------------
# Publish
# -----------------------------------------------------------------------------

[private]
_publish workspace dry_run allow_dirty no_verify registry:
    #!/usr/bin/env bash
    set -euo pipefail

    WORKSPACES=$(just _parse-workspace "{{workspace}}")

    for WS in $WORKSPACES; do
        if [ "{{workspace}}" = "all" ]; then
            echo "=== $WS ==="
        fi

        just _publish-sanity-checks "$WS"

        if [ "{{dry_run}}" = "true" ]; then
            just _publish-dry-run "$WS" "{{allow_dirty}}" "{{no_verify}}" "{{registry}}"
        else
            just _publish-run "$WS" "{{allow_dirty}}" "{{no_verify}}" "{{registry}}"
        fi

        if [ "{{workspace}}" = "all" ]; then
            echo ""
        fi
    done

[private]
_publish-exclude-args workspace:
    #!/usr/bin/env bash
    set -euo pipefail

    MANIFEST_FLAG=$(just _workspace-manifest-arg "{{workspace}}")
    IGNORE=$(just _publish-crates-to-ignore "{{workspace}}")

    MEMBERS=$(cargo metadata --format-version=1 --no-deps $MANIFEST_FLAG \
        | jq -r '.packages[].name')

    EXCLUDE_ARGS=""
    for CRATE in $IGNORE; do
        if echo "$MEMBERS" | grep -qx "$CRATE"; then
            EXCLUDE_ARGS+="--exclude $CRATE "
        fi
    done
    echo "$EXCLUDE_ARGS"

[private]
_publish-dry-run workspace allow_dirty no_verify registry:
    #!/usr/bin/env bash
    set -euo pipefail

    MANIFEST_FLAG=$(just _workspace-manifest-arg "{{workspace}}")
    EXCLUDE_ARGS=$(just _publish-exclude-args "{{workspace}}")

    DIRTY_FLAG=""
    if [ "{{allow_dirty}}" = "true" ]; then
        DIRTY_FLAG="--allow-dirty"
    fi

    NO_VERIFY_FLAG=""
    if [ "{{no_verify}}" = "true" ]; then
        NO_VERIFY_FLAG="--no-verify"
    fi

    REGISTRY_FLAG=""
    if [ -n "{{registry}}" ]; then
        REGISTRY_FLAG="--registry {{registry}}"
    fi

    echo "Publishing '{{workspace}}' (dry-run)..."
    cargo publish --dry-run --workspace \
        $MANIFEST_FLAG $REGISTRY_FLAG $EXCLUDE_ARGS $DIRTY_FLAG $NO_VERIFY_FLAG
    echo "Done."

[private]
_publish-run workspace allow_dirty no_verify registry:
    #!/usr/bin/env bash
    set -euo pipefail

    MANIFEST_FLAG=$(just _workspace-manifest-arg "{{workspace}}")
    EXCLUDE_ARGS=$(just _publish-exclude-args "{{workspace}}")

    DIRTY_FLAG=""
    if [ "{{allow_dirty}}" = "true" ]; then
        DIRTY_FLAG="--allow-dirty"
    fi

    NO_VERIFY_FLAG=""
    if [ "{{no_verify}}" = "true" ]; then
        NO_VERIFY_FLAG="--no-verify"
    fi

    REGISTRY_FLAG=""
    if [ -n "{{registry}}" ]; then
        REGISTRY_FLAG="--registry {{registry}}"
    fi

    echo "Publishing '{{workspace}}'..."
    cargo publish --workspace $MANIFEST_FLAG $REGISTRY_FLAG $EXCLUDE_ARGS $DIRTY_FLAG $NO_VERIFY_FLAG
    echo "Done."
