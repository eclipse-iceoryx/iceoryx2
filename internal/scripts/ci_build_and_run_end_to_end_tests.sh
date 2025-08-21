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

COLOR_OFF='\033[0m'
COLOR_RED='\033[1;31m'
COLOR_GREEN='\033[1;32m'
COLOR_YELLOW='\033[1;33m'

BUILD_END_TO_END_TESTS=true
RUN_END_TO_END_TESTS=true
PYTHON_END_TO_END_TESTS=true

while (( "$#" )); do
    case "$1" in
        no-build)
            BUILD_END_TO_END_TESTS=false
            shift 1
            ;;
        no-python)
            PYTHON_END_TO_END_TESTS=false
            shift 1
            ;;
        no-run)
            RUN_END_TO_END_TESTS=false
            shift 1
            ;;
        "help")
            echo "Script to run the end-to-end tests"
            echo ""
            echo "Args:"
            echo "    no-build              Skips the build step"
            echo "    no-python             Skips the Python tests"
            echo "    no-run                Skips the run step"
            echo ""
            exit 0
            ;;
        *)
            echo "Invalid argument '$1'. Try 'help' for options."
            exit 1
            ;;
    esac
done

WORKSPACE=$(git rev-parse --show-toplevel)
cd "${WORKSPACE}"

if [[ ${BUILD_END_TO_END_TESTS} == true ]]; then
    echo "##########################"
    echo "# Build end to end tests #"
    echo "##########################"

    cargo build --examples
    cargo build --bin iox2-service

    NUM_JOBS=1
    if [[ "$OSTYPE" == "linux-gnu"* ]] || [[ "$OSTYPE" == "cygwin" ]] || [[ "$OSTYPE" == "msys" ]]; then
        NUM_JOBS=$(nproc)
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        NUM_JOBS=$(sysctl -n hw.ncpu)
    fi

    # Clean build for iceoryx_hoofs
    rm -rf ${WORKSPACE}/target/ff/iceoryx
    ${WORKSPACE}/internal/scripts/ci_build_and_install_iceoryx_hoofs.sh

    # Build the C and C++ bindings
    cargo build --package iceoryx2-ffi-c
    cmake -S . -B target/ff/cc/build \
        -DCMAKE_PREFIX_PATH="$(pwd)/target/ff/iceoryx/install" \
        -DRUST_BUILD_ARTIFACT_PATH="$(pwd)/target/debug" \
        -DCMAKE_BUILD_TYPE=Debug \
        -DBUILD_CXX=ON \
        -DBUILD_EXAMPLES=ON \
        -DBUILD_TESTING=OFF
    cmake --build target/ff/cc/build -j$NUM_JOBS

    # Build the Python bindings
    if [[ ${PYTHON_END_TO_END_TESTS} == true ]]; then
        poetry --project iceoryx2-ffi/python install
        poetry --project iceoryx2-ffi/python build-into-venv
    fi
fi


if [[ ${RUN_END_TO_END_TESTS} == true ]]; then
    echo "##########################"
    echo "# Run end to end tests #"
    echo "##########################"

    # Search for all end-to-end test files
    cd "${WORKSPACE}"
    FILES=$(find ${WORKSPACE} -type f | grep -E "test_e2e_.*\.exp" | sort)
    FILES_ARRAY=(${FILES})
    NUMBER_OF_FILES=${#FILES_ARRAY[@]}

    if [[ ${NUMBER_OF_FILES} -eq 0 ]]; then
        echo -e "${COLOR_YELLOW}-> nothing to do${COLOR_OFF}"
        return 0
    fi

    echo -e "${COLOR_GREEN}Running tests ...${COLOR_OFF}"
    eval $(poetry --project iceoryx2-ffi/python env activate)
    FILE_COUNTER=1
    for FILE in $FILES; do
        echo -e "${COLOR_GREEN}[${FILE_COUNTER}/${NUMBER_OF_FILES}]${COLOR_OFF} RUN ${FILE}"
        FILE_COUNTER=$((FILE_COUNTER + 1))

        if test -f "$FILE"; then
            if [[ ${PYTHON_END_TO_END_TESTS} == false ]] && grep -q python "${FILE}"; then
                echo -e "${COLOR_YELLOW}Skipping Python end-to-end test!${COLOR_OFF}"
                continue
            fi
            bash -c ${FILE}
        else
            echo -e "${COLOR_RED}File does not exist! Aborting!${COLOR_OFF}"
            return 1
        fi
    done
fi


echo -e "${COLOR_GREEN}... done!${COLOR_OFF}"
