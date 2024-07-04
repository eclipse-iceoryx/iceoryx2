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

RUST_TOOLCHAIN="stable"
RUST_BUILD_TYPE_FLAG=""
CMAKE_BUILD_TYPE_FLAG="-DCMAKE_BUILD_TYPE=Debug"

while (( "$#" )); do
  case "$1" in
    --mode)
        if [[ "$2" == "release" ]]; then
            RUST_BUILD_TYPE_FLAG="--release"
            CMAKE_BUILD_TYPE_FLAG="-DCMAKE_BUILD_TYPE=Release"
        fi
        shift 2
        ;;
    --toolchain)
        RUST_TOOLCHAIN="$2"
        shift 2
        ;;
    "help")
        echo "Build script for the 32-64 bit mixed mode PoC."
        echo ""
        echo "Options:"
        echo "    --mode                Specify the build type. Either 'release' or 'debug'"
        echo "    --mode                Specify the rust toolchain, e.g. 'stable' or 'beta'"
        echo "Args:"
        echo "    help                  Print this help"
        echo ""
        exit 0
        ;;
    *)
        echo "Invalid argument '$1'. Try 'help' for options."
        exit 1
        ;;
  esac
done


export PATH=$PATH:$HOME/.cargo/bin
export LIBCLANG_PATH=/usr/local/llvm15/lib/
rustup default $RUST_TOOLCHAIN
export RUSTFLAGS="-C debug-assertions"
cargo fmt --all -- --check
cargo clippy -- -D warnings

echo "###################"
echo "# Run cargo build #"
echo "###################"

cargo build --workspace --all-targets $RUST_BUILD_TYPE_FLAG

echo "######################"
echo "# Run cargo nextest #"
echo "#####################"

cargo nextest run --workspace --no-fail-fast $RUST_BUILD_TYPE_FLAG

echo "###########################"
echo "# Build language bindings #"
echo "###########################"

cmake -S . -B target/ffi/build -DCMAKE_INSTALL_PREFIX=target/ffi/install $CMAKE_BUILD_TYPE_FLAG -DBUILD_EXAMPLES=ON -DBUILD_TESTING=ON
cmake --build target/ffi/build
cmake --install target/ffi/build

echo "#############################"
echo "# Run language binding tests #"
echo "#############################"

target/ffi/build/tests/iceoryx2-c-tests

echo "################################################################"
echo "# Build language binding examples in out-of-tree configuration #"
echo "################################################################"

rm -rf target/ffi/build
cmake -S examples/c -B target/ffi/out-of-tree -DCMAKE_PREFIX_PATH=$( pwd )/target/ffi/install $CMAKE_BUILD_TYPE_FLAG
cmake --build target/ffi/out-of-tree
