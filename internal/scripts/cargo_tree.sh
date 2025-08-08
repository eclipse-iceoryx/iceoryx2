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
iceoryx2-ffi
"

PACKAGE_LIST_CORE_MACROS="
iceoryx2-bb-derive-macros
iceoryx2-ffi-macros
"

PACKAGE_LIST_BINDINGS="
iceoryx2-ffi
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
        echo "Script to run 'cargo tree' on multiple packages"
        echo ""
        echo "The first argument selects the package list and"
        echo "all other arguments are forwarded to 'cargo tree'"
        echo ""
        echo ""
        echo "Args:"
        echo "    core              All workspace packages"
        echo "                      iceoryx2 depends on and the"
        echo "                      iceoryx2-ffi package"
        echo "    core-macros       The macros from core"
        echo "    bindings          All bindings"
        echo "    binding-macros    The macros from bindings"
        echo "    services          All services"
        echo "    tunnels           All tunnels"
        echo "    userland          All of userland"
        echo "    cli               All CLI tools"
        echo "    examples          All examples"
        echo "    benchmarks        All benchmarks"
        echo ""
        echo ""
        echo "Example usage:"
        echo ""
        echo "cargo_tree.sh core --depth 1 --edges all"
        echo ""
        echo "To get only selected dependencies, use 'normal', 'build' or 'dev' for '--edges'"
        echo "To get a plain list of the dependencies, use'--prefix none'"
        echo ""
        echo "To filter out all duplicates, add: \"| grep -v '(\*)'\""
        echo "To filter out iceoryx2 packages, add: \"| grep -v ' iceoryx2'\""
        echo ""
        echo "This command shows the minimal 'normal' dependencies of 'core' without duplicates"
        echo "cargo_tree.sh core --depth 1 --edges normal --prefix none --no-default-features | grep -v '(\*)' | grep -v iceoryx2 | sort | uniq"
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


