#!/usr/bin/env bash
# Copyright (c) 2025 Contributors to the Eclipse Foundation
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

COLOR_OFF='\033[0m'
COLOR_RED='\033[1;31m'
COLOR_GREEN='\033[1;32m'
COLOR_YELLOW='\033[1;33m'
COLOR_BLUE='\033[1;34m'

PACKAGE_LIST=""

PACKAGE_LIST_CORE="
iceoryx2
iceoryx2-cal
iceoryx2-bb-lock-free
iceoryx2-bb-threadsafe
iceoryx2-bb-container
iceoryx2-bb-elementary
iceoryx2-bb-elementary-traits
iceoryx2-bb-log
iceoryx2-bb-memory
iceoryx2-bb-posix
iceoryx2-bb-system-types
iceoryx2-bb-testing
iceoryx2-bb-trait-tests
iceoryx2-pal-concurrency-sync
iceoryx2-pal-posix
iceoryx2-pal-configuration
iceoryx2-pal-testing
iceoryx2-ffi-c
"

PACKAGE_LIST_CORE_MACROS="
iceoryx2-bb-derive-macros
iceoryx2-ffi-macros
"

PACKAGE_LIST_BINDINGS="
iceoryx2-ffi-c
iceoryx2-ffi-python
"

PACKAGE_LIST_BINDING_MACROS="
iceoryx2-ffi-macros
"
PACKAGE_LIST_SERVICES="
iceoryx2-services-discovery
"

PACKAGE_LIST_TUNNEL="
iceoryx2-tunnels-end-to-end-testing
iceoryx2-tunnels-zenoh
"

PACKAGE_LIST_USERLAND="
iceoryx2-userland-record-and-replay
"

PACKAGE_LIST_CLI="
iceoryx2-cli
"

PACKAGE_LIST_EXAMPLES="
examples
"

PACKAGE_LIST_BENCHMARKS="
benchmarks-request-response
benchmarks-publish-subscribe
benchmarks-event
benchmarks-queue
"

print_help() {
    echo -e "Script to run ${COLOR_GREEN}cargo tree${COLOR_OFF} on multiple packages"
    echo -e ""
    echo -e "Usage: ${COLOR_GREEN}$(basename $0)${COLOR_OFF} ${COLOR_BLUE}SCRIPT-OPTION${COLOR_OFF} [${COLOR_BLUE}CARGO-TREE-OPTIONS${COLOR_OFF}]"
    echo -e ""
    echo -e "The first argument selects the package list and"
    echo -e "all other arguments are forwarded to ${COLOR_GREEN}cargo tree${COLOR_OFF}"
    echo -e ""
    echo -e ""
    echo -e "Options:"
    echo -e "    ${COLOR_BLUE}core${COLOR_OFF}              All workspace packages"
    echo -e "                      iceoryx2 depends on and the"
    echo -e "                      iceoryx2-ffi-c package"
    echo -e "    ${COLOR_BLUE}core-macros${COLOR_OFF}       The macros from core"
    echo -e "    ${COLOR_BLUE}bindings${COLOR_OFF}          All bindings"
    echo -e "    ${COLOR_BLUE}binding-macros${COLOR_OFF}    The macros from bindings"
    echo -e "    ${COLOR_BLUE}services${COLOR_OFF}          All services"
    echo -e "    ${COLOR_BLUE}tunnels${COLOR_OFF}           All tunnels"
    echo -e "    ${COLOR_BLUE}userland${COLOR_OFF}          All of userland"
    echo -e "    ${COLOR_BLUE}cli${COLOR_OFF}               All CLI tools"
    echo -e "    ${COLOR_BLUE}examples${COLOR_OFF}          All examples"
    echo -e "    ${COLOR_BLUE}benchmarks${COLOR_OFF}        All benchmarks"
    echo -e ""
    echo -e ""
    echo -e "Example usage:"
    echo -e ""
    echo -e "${COLOR_GREEN}cargo_tree.sh core --depth 1 --edges all${COLOR_OFF}"
    echo -e ""
    echo -e "To get only selected dependencies, use ${COLOR_BLUE}normal${COLOR_OFF}, ${COLOR_BLUE}build${COLOR_OFF} or ${COLOR_BLUE}dev${COLOR_OFF} for ${COLOR_BLUE}--edges${COLOR_OFF}"
    echo -e "To get a plain list of the dependencies, use ${COLOR_BLUE}--prefix none${COLOR_OFF}"
    echo -e ""
    echo -e "To filter out all duplicates, add: ${COLOR_GREEN}| grep -v '(\*)'${COLOR_OFF}"
    echo -e "To filter out iceoryx2 packages, add: ${COLOR_GREEN}| grep -v ' iceoryx2'${COLOR_OFF}"
    echo -e ""
    echo -e "This command shows the minimal 'normal' dependencies of 'core' without duplicates"
    echo -e "${COLOR_GREEN}cargo_tree.sh core --depth 1 --edges normal --prefix none --no-default-features | grep -v '(\*)' | grep -v iceoryx2 | sort | uniq${COLOR_OFF}"
}

if [ $# -eq 0 ]; then
    print_help
    exit 0
fi

case "$1" in
    core)
        PACKAGE_LIST=${PACKAGE_LIST_CORE}
        shift 1
        ;;
    core-macros)
        PACKAGE_LIST=${PACKAGE_LIST_CORE_MACROS}
        shift 1
        ;;
    bindings)
        PACKAGE_LIST=${PACKAGE_LIST_BINDINGS}
        shift 1
        ;;
    binding-macross)
        PACKAGE_LIST=${PACKAGE_LIST_BINDING_MACROS}
        shift 1
        ;;
    services)
        PACKAGE_LIST=${PACKAGE_LIST_SERVICES}
        shift 1
        ;;
    tunnel)
        PACKAGE_LIST=${PACKAGE_LIST_TUNNEL}
        shift 1
        ;;
    userland)
        PACKAGE_LIST=${PACKAGE_LIST_USERLAND}
        shift 1
        ;;
    cli)
        PACKAGE_LIST=${PACKAGE_LIST_CLI}
        shift 1
        ;;
    examples)
        PACKAGE_LIST=${PACKAGE_LIST_EXAMPLES}
        shift 1
        ;;
    benchmarks)
        PACKAGE_LIST=${PACKAGE_LIST_BENCHMARKS}
        shift 1
        ;;
    "help")
        print_help
        exit 0
        ;;
    *)
        echo "Invalid argument '$1'. Try 'help' for options."
        exit 1
        ;;
esac

WORKSPACE=$(git rev-parse --show-toplevel)
cd "${WORKSPACE}"

# NOTE using 'cargo tree --package foo --package bar --edges normal' did not print the
# dependencies of some packages, therefore run 'cargo tree' on each package separately
for PACKAGE in ${PACKAGE_LIST}; do
    cargo tree --package ${PACKAGE} $@
done


