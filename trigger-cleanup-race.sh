#! /bin/bash

set -e

cd $(git rev-parse --show-toplevel)

NUM_JOBS=1
if [[ "$OSTYPE" == "linux-gnu"* ]] || [[ "$OSTYPE" == "cygwin" ]] || [[ "$OSTYPE" == "msys" ]]; then
    NUM_JOBS=$(nproc)
elif [[ "$OSTYPE" == "darwin"* ]]; then
    NUM_JOBS=$(sysctl -n hw.ncpu)
fi

# Build the C and C++ bindings
cargo build --package iceoryx2-ffi-c
cmake -S . -B target/ff/cc/build \
    -DRUST_BUILD_ARTIFACT_PATH="$(pwd)/target/debug" \
    -DCMAKE_BUILD_TYPE=Debug \
    -DBUILD_CXX=ON \
    -DBUILD_EXAMPLES=ON \
    -DBUILD_TESTING=ON
cmake --build target/ff/cc/build -j$NUM_JOBS

iterations=100
for i in $(seq 1 $iterations); do
    echo "#### Iteration: $i of $iterations"
    # examples/cross-language-end-to-end-tests/test_e2e_server_cxx_client_c.exp
    examples/cxx/request_response/test_e2e_request_response.exp
done
