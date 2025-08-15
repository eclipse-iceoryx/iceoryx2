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

CMAKE_C_FLAGS=""
CMAKE_CXX_FLAGS=""

while (( "$#" )); do
  case "$1" in
    "32-bit-x86")
        echo " [i] Build as 32 bit x86 library"
        # NOTE: "-malign-double" would only be needed 32-64-bit cross architecture communication,
        #       but at least for the cross compilation setup, there are issues with the C++ tests
        #       when the "-malign-double" flag is not set
        CMAKE_C_FLAGS="${CMAKE_C_FLAGS} -m32 -malign-double"
        CMAKE_CXX_FLAGS="${CMAKE_CXX_FLAGS} -m32 -malign-double"
        shift 1
        ;;
    "32-bit-arm")
        echo " [i] Build as 32 bit ARM library"
        # NOTE: there is no '-m32' flag on ARM; the architecture is selected via the externally defined toolchain
        CMAKE_C_FLAGS="${CMAKE_C_FLAGS} -malign-double"
        CMAKE_CXX_FLAGS="${CMAKE_CXX_FLAGS} -malign-double"
        shift 1
        ;;
    "help")
        echo "Build script for iceoryx hoofs."
        echo ""
        echo "Usage:"
        echo "    ci_build_and_install_iceoryx_hoofs.sh [<args>]"
        echo "Args:"
        echo "    32-bit-x86            Build as 32 bit library for x64"
        echo "    32-bit-arm            Build as 32 bit library for arm"
        exit 0
        ;;
    *)
        echo "Invalid argument '$1'. Try 'help' for options."
        exit 1
        ;;
  esac
done

NUM_JOBS=1
if [[ "$OSTYPE" == "linux-gnu"* ]] || [[ "$OSTYPE" == "cygwin" ]] || [[ "$OSTYPE" == "msys" ]]; then
    NUM_JOBS=$(nproc)
elif [[ "$OSTYPE" == "darwin"* ]]; then
    NUM_JOBS=$(sysctl -n hw.ncpu)
fi

git clone --depth 1 --branch v2.95.7 https://github.com/eclipse-iceoryx/iceoryx.git target/ff/iceoryx/src

cmake -S target/ff/iceoryx/src/iceoryx_platform \
      -B target/ff/iceoryx/build/platform \
      -DBUILD_SHARED_LIBS=OFF \
      -DCMAKE_BUILD_TYPE=Release \
      -DCMAKE_INSTALL_PREFIX=target/ff/iceoryx/install \
      -DIOX_PLATFORM_FEATURE_ACL=OFF \
      -DCMAKE_C_FLAGS="$CMAKE_C_FLAGS" \
      -DCMAKE_CXX_FLAGS="$CMAKE_CXX_FLAGS"
cmake --build target/ff/iceoryx/build/platform -j$NUM_JOBS
cmake --install target/ff/iceoryx/build/platform

cmake -S target/ff/iceoryx/src/iceoryx_hoofs \
      -B target/ff/iceoryx/build/hoofs \
      -DBUILD_SHARED_LIBS=OFF \
      -DCMAKE_BUILD_TYPE=Release \
      -DCMAKE_INSTALL_PREFIX=target/ff/iceoryx/install \
      -DCMAKE_PREFIX_PATH="$( pwd )/target/ff/iceoryx/install" \
      -DIOX_USE_HOOFS_SUBSET_ONLY=ON \
      -DCMAKE_C_FLAGS="$CMAKE_C_FLAGS" \
      -DCMAKE_CXX_FLAGS="$CMAKE_CXX_FLAGS"
cmake --build target/ff/iceoryx/build/hoofs -j$NUM_JOBS
cmake --install target/ff/iceoryx/build/hoofs
