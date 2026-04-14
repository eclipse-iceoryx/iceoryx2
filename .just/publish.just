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

ICEORYX2_CRATES_TO_PUBLISH := `echo \
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
    iceoryx2-services-discovery         \
    iceoryx2-services-tunnel-backend    \
    iceoryx2-services-tunnel            \
    iceoryx2-services-tunnel-conformance-tests \
    iceoryx2-userland-record-and-replay \
    iceoryx2-cli`

ICEORYX2_CRATES_TO_IGNORE := `echo \
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

[private]
_publish-dispatch what *flags:
    #!/usr/bin/env bash
    set -euo pipefail

    DRY_RUN="false"
    ALLOW_DIRTY="false"
    SANITY_CHECKS="false"
    LIST="false"
    REGISTRY=""

    set -- {{flags}}
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --dry-run)       DRY_RUN="true";       shift ;;
            --allow-dirty)   ALLOW_DIRTY="true";    shift ;;
            --sanity-checks) SANITY_CHECKS="true";  shift ;;
            --list)          LIST="true";            shift ;;
            --registry)      REGISTRY="$2";         shift 2 ;;
            *)               echo "Unknown flag: $1"; exit 1 ;;
        esac
    done

    case "{{what}}" in
        "")
            echo "Usage: just publish <what> [flags]"
            echo ""
            echo "  sdk             publish the main workspace crates"
            echo "  integrations    publish the integrations workspace crates"
            echo "  all             publish all workspaces in order"
            echo ""
            echo "Flags:"
            echo "  --dry-run          simulate publishing (requires Rust >= 1.90)"
            echo "  --allow-dirty      allow dirty workspace during dry-run"
            echo "  --sanity-checks    run sanity checks only"
            echo "  --list             list crates to publish"
            echo "  --registry <url>   publish to a custom registry"
            exit 1
            ;;
        sdk)
            just _publish-action "$DRY_RUN" "$ALLOW_DIRTY" "$SANITY_CHECKS" "$LIST" "$REGISTRY"
            ;;
        integrations)
            echo "Publishing for 'integrations' is not yet implemented."
            exit 0
            ;;
        all)
            echo "=== sdk ==="
            just _publish-action "$DRY_RUN" "$ALLOW_DIRTY" "$SANITY_CHECKS" "$LIST" "$REGISTRY"
            echo ""
            echo "=== integrations ==="
            echo "Publishing for 'integrations' is not yet implemented."
            ;;
        *)
            echo "Unknown publish target: {{what}}"
            exit 1
            ;;
    esac

# -----------------------------------------------------------------------------
# Action dispatch
# -----------------------------------------------------------------------------

[private]
_publish-action dry_run allow_dirty sanity_checks list registry:
    #!/usr/bin/env bash
    set -euo pipefail

    if [ "{{list}}" = "true" ]; then
        just _publish-list
        exit 0
    fi

    if [ "{{sanity_checks}}" = "true" ]; then
        just _publish-sanity-checks
        exit 0
    fi

    if [ "{{dry_run}}" = "true" ]; then
        just _publish-sanity-checks
        just _publish-dry-run "{{allow_dirty}}" "{{registry}}"
    else
        just _publish-sanity-checks
        just _publish-run "{{registry}}"
    fi

# -----------------------------------------------------------------------------
# List
# -----------------------------------------------------------------------------

[private]
_publish-list:
    #!/usr/bin/env bash
    set -euo pipefail

    echo "Crates to publish:"
    PUBLISH="{{ICEORYX2_CRATES_TO_PUBLISH}}"
    for CRATE in $PUBLISH; do
        echo "  $CRATE"
    done

# -----------------------------------------------------------------------------
# Sanity checks
# -----------------------------------------------------------------------------

[private]
_publish-sanity-checks:
    #!/usr/bin/env bash
    set -euo pipefail

    echo "Running sanity checks..."
    just _publish-check-completeness
    just _publish-check-cyclic-deps
    echo "All sanity checks passed."

[private]
_publish-check-completeness:
    #!/usr/bin/env bash
    set -euo pipefail

    WORKSPACE_CRATES=$(cargo tree --workspace --depth 1 --prefix none \
        | grep -v '(\*)' \
        | grep -E '^(iceoryx2|benchmark|example)' \
        | awk '{print $1}' \
        | sort | uniq)

    PUBLISH="{{ICEORYX2_CRATES_TO_PUBLISH}}"
    IGNORE="{{ICEORYX2_CRATES_TO_IGNORE}}"

    HAS_ERROR=false
    for CRATE in $WORKSPACE_CRATES; do
        if [[ " $IGNORE " =~ " ${CRATE} " ]]; then
            continue
        fi
        if [[ " $PUBLISH " =~ " ${CRATE} " ]]; then
            continue
        fi
        echo "ERROR: Crate '$CRATE' is neither in the publish nor ignore list."
        HAS_ERROR=true
    done

    if [ "$HAS_ERROR" = true ]; then
        exit 1
    fi

[private]
_publish-check-cyclic-deps:
    #!/usr/bin/env bash
    set -euo pipefail

    PUBLISH="{{ICEORYX2_CRATES_TO_PUBLISH}}"

    ALLOWED=""
    HAS_ERROR=false

    for CRATE in $PUBLISH; do
        ALLOWED="$ALLOWED $CRATE"

        DEPS=$(cargo tree --package "$CRATE" --depth 1 --prefix none --edges normal \
            | grep -v '(\*)' \
            | grep -e '^iceoryx2' \
            | awk '{print $1}' \
            | sort | uniq)

        for DEP in $DEPS; do
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
_publish-dry-run allow_dirty registry:
    #!/usr/bin/env bash
    set -euo pipefail

    IGNORE="{{ICEORYX2_CRATES_TO_IGNORE}}"

    EXCLUDE_ARGS=""
    for CRATE in $IGNORE; do
        EXCLUDE_ARGS+="--exclude $CRATE "
    done

    DIRTY_FLAG=""
    if [ "{{allow_dirty}}" = "true" ]; then
        DIRTY_FLAG="--allow-dirty"
    fi

    REGISTRY_FLAG=""
    if [ -n "{{registry}}" ]; then
        REGISTRY_FLAG="--registry {{registry}}"
    fi

    echo "Publishing (dry-run)..."
    cargo publish --dry-run --workspace $EXCLUDE_ARGS $DIRTY_FLAG $REGISTRY_FLAG
    echo "Done."

[private]
_publish-run registry:
    #!/usr/bin/env bash
    set -euo pipefail

    IGNORE="{{ICEORYX2_CRATES_TO_IGNORE}}"

    EXCLUDE_ARGS=""
    for CRATE in $IGNORE; do
        EXCLUDE_ARGS+="--exclude $CRATE "
    done

    REGISTRY_FLAG=""
    if [ -n "{{registry}}" ]; then
        REGISTRY_FLAG="--registry {{registry}}"
    fi

    echo "Publishing..."
    cargo publish --workspace $EXCLUDE_ARGS $REGISTRY_FLAG
    echo "Done."
