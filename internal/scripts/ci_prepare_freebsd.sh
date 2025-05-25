#!/bin/sh
# Copyright (c) 2024 Contributors to the Eclipse Foundation
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

export ASSUME_ALWAYS_YES=yes

pkg install python3 bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --no-modify-path
export PATH=$PATH:$HOME/.cargo/bin
rustup toolchain add beta nightly 1.81.0 stable
rustup component add clippy rustfmt
rustup default stable
cargo install cargo-nextest --locked
pw useradd testuser1
pw useradd testuser2
pw groupadd testgroup1
pw groupadd testgroup2
kldload mqueuefs
mkdir -p /mnt/mqueue/
mount -t mqueuefs null /mnt/mqueue/
