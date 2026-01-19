#!/usr/bin/env bash
# Copyright (c) 2023 Contributors to the Eclipse Foundation
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

set -e

C_OFF='\033[0m'
C_BOLD='\033[1m'
C_RED='\033[1;31m'
C_GREEN='\033[1;32m'
C_YELLOW='\033[1;33m'
C_BLUE='\033[1;34m'

DO_DRY_RUN=false
ALLOW_DIRTY_FLAG=""
DO_LIST_CRATES_TO_PUBLISH=false
DO_PUBLISH=false
DO_SANITY_CHECKS=false

CRATES_TO_PUBLISH=(
    iceoryx2-pal-configuration
    iceoryx2-pal-testing
    iceoryx2-pal-concurrency-sync
    iceoryx2-pal-posix
    iceoryx2-pal-os-api
    iceoryx2-log-types
    iceoryx2-log
    iceoryx2-bb-print
    iceoryx2-bb-loggers
    iceoryx2-bb-conformance-test-macros
    iceoryx2-bb-elementary-traits
    iceoryx2-bb-testing
    iceoryx2-bb-concurrency
    iceoryx2-bb-elementary
    iceoryx2-bb-derive-macros
    iceoryx2-bb-container
    iceoryx2-bb-system-types
    iceoryx2-bb-posix
    iceoryx2-bb-linux
    iceoryx2-bb-lock-free
    iceoryx2-bb-threadsafe
    iceoryx2-bb-memory
    iceoryx2-cal
    iceoryx2-cal-conformance-tests
    iceoryx2-bb-trait-tests
    iceoryx2
    iceoryx2-conformance-tests
    iceoryx2-services-discovery
    iceoryx2-userland-record-and-replay
    iceoryx2-tunnel-backend
    iceoryx2-tunnel
    iceoryx2-tunnel-conformance-tests
    iceoryx2-tunnel-zenoh
    iceoryx2-cli
)
CRATES_TO_IGNORE=(
    benchmark-event
    benchmark-publish-subscribe
    benchmark-request-response
    benchmark-queue
    component-tests_rust
    example
    iceoryx2-ffi-c
    iceoryx2-ffi-macros
    iceoryx2-ffi-python
    iceoryx2-tunnel-end-to-end-tests
)

if [[ "$#" -eq 0 ]]; then
    echo -e "${C_RED}ERROR:${C_OFF} No arguments provided. Try 'help' for options."
    exit 1
fi

while (( "$#" )); do
    case "$1" in
        "dry-run")
            DO_DRY_RUN=true
            shift 1
            ;;
        "dry-run-allow-dirty")
            DO_DRY_RUN=true
            ALLOW_DIRTY_FLAG="--allow-dirty"
            shift 1
            ;;
        "list-crates-to-publish")
            DO_LIST_CRATES_TO_PUBLISH=true
            shift 1
            ;;
        "publish")
            DO_PUBLISH=true
            shift 1
            ;;
        "sanity-checks")
            DO_SANITY_CHECKS=true
            shift 1
            ;;
        "help")
            echo -e "Script to publish to crates.io"
            echo -e ""
            echo -e "Usage: ${C_GREEN}$(basename $0)${C_OFF} publish"
            echo -e "Options:"
            echo -e "    dry-run                 Simulate publishing to crates.io"
            echo -e "                            Only works with Rust >= 1.90"
            echo -e "    dry-run-allow-dirty     Same as 'dry-run' but with a dirty workspace"
            echo -e "    publish                 Publish to crates.io"
            echo -e "    list-crates-to-publish  List crates to publish to crates.io"
            echo -e "    sanity-checks           Sanity checks for cyclic dependencies and new crates"
            echo -e ""
            exit 0
            ;;
        *)
            echo -e "${C_RED}ERROR:${C_OFF} Invalid argument '$1'. Try 'help' for options."
            exit 1
            ;;
    esac
done

cd $(git rev-parse --show-toplevel)

sanity_check_completeness() {
    local ICEORYX2_CRATES=$(cargo tree --workspace --depth 1 --prefix none | grep -v '(\*)' | grep -E '^(iceoryx2|benchmark|example)' | awk '{print $1}' | sort | uniq)

    local HAS_ERROR=false
    for CRATE in ${ICEORYX2_CRATES}; do
        if [[ " ${CRATES_TO_IGNORE[@]} " =~ ${CRATE} ]]; then
            continue
        fi

        if [[ " ${CRATES_TO_PUBLISH[@]} " =~ ${CRATE} ]]; then
            continue
        fi

        echo -e "${C_RED}ERROR:${C_OFF} Crate '$CRATE' is neither in the publish nor ignore list."
        HAS_ERROR=true
    done

    if [[ ${HAS_ERROR} == true ]]; then
        exit 1
    fi
}

sanity_check_cyclic_dependencies() {
    declare -a ALLOWED_CRATE_DEPENDENCIES

    local HAS_ERROR=false
    for CRATE in "${CRATES_TO_PUBLISH[@]}"; do
        ALLOWED_CRATE_DEPENDENCIES+=("${CRATE}")

        local CRATE_DEPENDENCIES=$(cargo tree --package "${CRATE}" --depth 1 --prefix none | grep -v '(\*)' | grep -e '^iceoryx2' | awk '{print $1}' | sort | uniq)
        for DEP in ${CRATE_DEPENDENCIES}; do
            if [[ " ${ALLOWED_CRATE_DEPENDENCIES[@]} " =~ " ${DEP} " ]]; then
                continue
            else
                echo -e "${C_RED}ERROR:${C_OFF} ${C_YELLOW}${CRATE}${C_OFF} dependency ${C_BLUE}${DEP}${C_OFF} is out of order for publishing!"
                HAS_ERROR=true
            fi
        done
    done

    if [[ ${HAS_ERROR} == true ]]; then
        exit 1
    fi
}

sanity_checks() {
    sanity_check_completeness
    sanity_check_cyclic_dependencies
}

dry_run() {
    local EXCLUDE_ARGS=""
    for CRATE in "${CRATES_TO_IGNORE[@]}"; do
        EXCLUDE_ARGS+="--exclude $CRATE "
    done
    cargo publish --dry-run --workspace ${EXCLUDE_ARGS} ${ALLOW_DIRTY_FLAG}
}

list_crates_to_publish() {
    for CRATE in ${CRATES_TO_PUBLISH[@]}; do
        echo -e "${C_BOLD}${CRATE}${C_OFF}"
    done
}

publish() {
    local EXCLUDE_ARGS=""
    for CRATE in "${CRATES_TO_IGNORE[@]}"; do
        EXCLUDE_ARGS+="--exclude $CRATE "
    done
    cargo publish --workspace ${EXCLUDE_ARGS}
}

if [[ ${DO_SANITY_CHECKS} == true ]]; then
    sanity_checks
fi

if [[ ${DO_LIST_CRATES_TO_PUBLISH} == true ]]; then
    list_crates_to_publish
fi

if [[ ${DO_DRY_RUN} == true ]]; then
    echo -e "Performing Sanity Checks"
    sanity_checks

    echo -e "Publishing (dry-run)..."
    dry_run
    echo -e "${C_GREEN}...done${C_OFF}"
fi

if [[ ${DO_PUBLISH} == true ]]; then
    echo -e "Performing Sanity Checks"
    sanity_checks

    echo -e "Publishing..."
    publish
    echo -e "${C_GREEN}...done${C_OFF}"
fi
