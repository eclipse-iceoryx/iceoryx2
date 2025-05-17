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

#!/bin/bash

set -e

pacman -Syu --noconfirm clang cmake gcc git rustup
pacman -Scc --noconfirm
rustup toolchain add beta nightly stable 1.81.0
rustup component add clippy llvm-tools rustfmt
rustup default stable
