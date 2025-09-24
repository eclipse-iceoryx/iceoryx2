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
DO_PUBLISH=false
DO_SANITY_CHECKS=false

CRATES_TO_PUBLISH=(
    iceoryx2-pal-configuration
    iceoryx2-pal-testing
    iceoryx2-pal-concurrency-sync
    iceoryx2-pal-posix
    iceoryx2-pal-os-api
    iceoryx2-bb-elementary-traits
    iceoryx2-bb-testing
    iceoryx2-bb-log
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
    iceoryx2-bb-trait-tests
    iceoryx2
    iceoryx2-services-discovery
    iceoryx2-userland-record-and-replay
    iceoryx2-tunnels-zenoh
    iceoryx2-cli
)
CRATES_TO_IGNORE=(
    benchmark-event
    benchmark-publish-subscribe
    benchmark-request-response
    benchmark-queue
    example
    iceoryx2-ffi-c
    iceoryx2-ffi-macros
    iceoryx2-ffi-python
    iceoryx2-tunnels-end-to-end-testing
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
            echo -e "    dry-run            Simulate publishing to crates.io"
            echo -e "                       Only works with Rust >= 1.90"
            echo -e "    publish            Publish to crates.io"
            echo -e "    sanity-checks      Sanity checks for cyclic dependencies and new crates"
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
    ICEORYX2_CRATES=$(grep -E '^iceoryx2' Cargo.toml 2>/dev/null | awk '{print $1}' | sort -u)

    for CRATE in ${ICEORYX2_CRATES}; do
        if [[ " ${CRATES_TO_IGNORE[@]} " =~ " ${CRATE} " ]]; then
            continue
        fi

        if [[ " ${CRATES_TO_PUBLISH[@]} " =~ " ${CRATE} " ]]; then
            continue
        fi

        echo -c "${C_RED}ERROR:${C_OFF} Crate '$CRATE' is neither in the publish nor ignore list."
        exit 1
    done
}

sanity_check_cyclic_dependencies() {
    declare -a ALLOWED_CRATE_DEPENDENCIES

    HAS_ERROR=false
    for CRATE in "${CRATES_TO_PUBLISH[@]}"; do
        ALLOWED_CRATE_DEPENDENCIES+=(${CRATE})

        CRATE_DEPENDENCIES=$(cargo tree --package "${CRATE}" --depth 1 --prefix none | grep -v '(\*)' | grep '^iceoryx2' | awk '{print $1}' | sort | uniq)
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
    EXCLUDE_ARGS=""
    for CRATE in "${CRATES_TO_IGNORE[@]}"; do
        EXCLUDE_ARGS+="--exclude $CRATE "
    done
    cargo publish --dry-run --workspace ${EXCLUDE_ARGS}
}

publish() {
    for CRATE in ${CRATES_TO_PUBLISH[@]}; do
        echo -e "${C_BLUE}${CRATE}${C_OFF}"
        cargo publish -p ${CRATE}
    done
}

if [[ ${DO_SANITY_CHECKS} == true ]]; then
    sanity_checks
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
