#!/usr/bin/env bash
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

echo "#######################"
echo "# Build iceoryx_hoofs #"
echo "#######################"

cd $(git rev-parse --show-toplevel)

NUM_JOBS=1
if [[ "$OSTYPE" == "linux-gnu"* ]] || [[ "$OSTYPE" == "cygwin" ]] || [[ "$OSTYPE" == "msys" ]]; then
    NUM_JOBS=$(nproc)
elif [[ "$OSTYPE" == "darwin"* ]]; then
    NUM_JOBS=$(sysctl -n hw.ncpu)
fi

git clone --depth 1 --branch v2.95.6 https://github.com/eclipse-iceoryx/iceoryx.git target/ff/iceoryx/src

cmake -S target/ff/iceoryx/src/iceoryx_platform -B target/ff/iceoryx/build/platform -DBUILD_SHARED_LIBS=OFF -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=target/ff/iceoryx/install
cmake --build target/ff/iceoryx/build/platform -j$NUM_JOBS
cmake --install target/ff/iceoryx/build/platform

cmake -S target/ff/iceoryx/src/iceoryx_hoofs -B target/ff/iceoryx/build/hoofs -DBUILD_SHARED_LIBS=OFF -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=target/ff/iceoryx/install -DCMAKE_PREFIX_PATH="$( pwd )/target/ff/iceoryx/install" -DIOX_USE_HOOFS_SUBSET_ONLY=ON
cmake --build target/ff/iceoryx/build/hoofs -j$NUM_JOBS
cmake --install target/ff/iceoryx/build/hoofs
