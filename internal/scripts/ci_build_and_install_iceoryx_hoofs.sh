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

#!/bin/sh

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

git clone --depth 1 --branch v2.95.4 https://github.com/eclipse-iceoryx/iceoryx.git target/iceoryx/src

cmake -S target/iceoryx/src/iceoryx_platform -B target/iceoryx/build/platform -DBUILD_SHARED_LIBS=OFF -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=target/iceoryx/install
cmake --build target/iceoryx/build/platform -j$NUM_JOBS
cmake --install target/iceoryx/build/platform

cmake -S target/iceoryx/src/iceoryx_hoofs -B target/iceoryx/build/hoofs -DBUILD_SHARED_LIBS=OFF -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=target/iceoryx/install -DCMAKE_PREFIX_PATH="$( pwd )/target/iceoryx/install"
cmake --build target/iceoryx/build/hoofs -j$NUM_JOBS
cmake --install target/iceoryx/build/hoofs
