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

cd $(git rev-parse --show-toplevel)

cargo publish -p iceoryx2-pal-configuration
cargo publish -p iceoryx2-pal-testing
cargo publish -p iceoryx2-pal-concurrency-sync
cargo publish -p iceoryx2-pal-posix
cargo publish -p iceoryx2-bb-elementary-traits
cargo publish -p iceoryx2-bb-testing
cargo publish -p iceoryx2-bb-log
cargo publish -p iceoryx2-bb-elementary
cargo publish -p iceoryx2-bb-derive-macros
cargo publish -p iceoryx2-bb-container
cargo publish -p iceoryx2-bb-system-types
cargo publish -p iceoryx2-bb-posix
cargo publish -p iceoryx2-bb-lock-free
cargo publish -p iceoryx2-bb-threadsafe
cargo publish -p iceoryx2-bb-memory
cargo publish -p iceoryx2-cal
cargo publish -p iceoryx2-bb-trait-tests
cargo publish -p iceoryx2
cargo publish -p iceoryx2-services-discovery
cargo publish -p iceoryx2-userland-record-and-replay
cargo publish -p iceoryx2-tunnels-zenoh
cargo publish -p iceoryx2-cli
