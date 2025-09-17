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

COLOR_OFF='\033[0m'
COLOR_RED='\033[1;31m'
COLOR_GREEN='\033[1;32m'
COLOR_YELLOW='\033[1;33m'

cd $(git rev-parse --show-toplevel)

LLVM_PATH=$(dirname $(which llvm-profdata))
LLVM_PROFILE_PATH="target/debug/llvm-profile-files"
export LLVM_PROFILE_FILE="${LLVM_PROFILE_PATH}/iceoryx2-%p-%m.profraw"
export DEBUGINFOD_URLS=/dev/null

set_rustc_flags() {
    if [[ "$(rustc --version | grep nightly | wc -l)" == "1" ]]
    then
        echo -e "${COLOR_GREEN}rust nightly compiler found, activating MC/DC coverage check${COLOR_OFF}"
        export RUSTFLAGS="-C instrument-coverage -Z coverage-options=condition" # Todo (iox2-#1052) use mcdc here
    else
        echo -e "${COLOR_YELLOW}no rust nightly compiler found, MC/DC coverage is not available only line coverage${COLOR_OFF}"
        export RUSTFLAGS="-C instrument-coverage"
    fi
}

RUST_COV_DIR="target/debug/coverage"
CMAKE_COV_DIR="target/ff/cc/coverage"
COVERAGE_OUT_DIR="target/coverage"
COMMIT_SHA=$(git rev-parse HEAD)

CLEAN=0
GENERATE=0
REPORT=0
OVERVIEW=0
HTML=0
LCOV=0

dependency_check() {
    which $1 1> /dev/null || { echo -e "${COLOR_RED}'${1}' not found. Aborting!${COLOR_OFF}"; exit 1; }
}

cleanup() {
    find . -name "*profraw" -exec rm {} \;
    cargo clean
}

generate_full_profile() {
    mkdir -p ${CMAKE_COV_DIR}
    generate_cmake_profile
    generate_rust_profile
}

generate_rust_profile() {
    set_rustc_flags
    cargo test --workspace --all-targets -- --test-threads=1 --skip "zenoh_tunnel_events" --skip "zenoh_tunnel_publish_subscribe"
}

generate_cmake_profile() {
    # Build with Coverage to generate .gcno files
    cmake . -B${CMAKE_COV_DIR} -DCOVERAGE=ON -DBUILD_TESTING=ON
    cmake --build ${CMAKE_COV_DIR} -j
    # Execute all tests to generate .gcda files
    ctest --test-dir ${CMAKE_COV_DIR} --output-on-failure
}

merge_report() {
    dependency_check llvm-profdata

    if [[ ! -f "./${RUST_COV_DIR}/json5format.profdata" ]]; then
        # get LLVM versions of llvm-profdata and rustc
        LLVM_PROFDATA_VERSION_OUTPUT=$( llvm-profdata merge --version )
        LLVM_VERSION=$(echo "$LLVM_PROFDATA_VERSION_OUTPUT" | grep -oP 'LLVM version \K[0-9]+')

        RUSTC_VERSION_OUTPUT=$( rustc --version --verbose )
        RUSTC_LLVM_VERSION=$(echo "$RUSTC_VERSION_OUTPUT" | grep -oP 'LLVM version: \K[0-9]+')

        # check LLVM versions for compatibility
        if [[ "$LLVM_VERSION" -ne "$RUSTC_LLVM_VERSION" ]]; then
            echo -e "llvm-profdata LLVM version: $LLVM_VERSION"
            echo -e "rustc LLVM version: $RUSTC_LLVM_VERSION"
            echo -e "${COLOR_RED}error: LLVM major versions do not match${COLOR_OFF}"
            exit 1
        fi

        # create report
        mkdir -p ./${RUST_COV_DIR}/
        local FILES=$(find . -name "*profraw")
        llvm-profdata merge --sparse $FILES -o ./${RUST_COV_DIR}/json5format.profdata
    fi
}

generate() {
    cleanup
    generate_rust_profile
}

show_overview() {
    dependency_check llvm-cov

    merge_report

    local FILES=$(find ./target/debug/deps/ -type f -executable)
    CMD="llvm-cov report --use-color --ignore-filename-regex='/.cargo/registry' --instr-profile=./${RUST_COV_DIR}/json5format.profdata"

    for FILE in $FILES
    do
        CMD="$CMD --object $FILE"
    done

    eval $CMD
}

show_report() {
    dependency_check llvm-cov
    dependency_check rustfilt

    merge_report

    local FILES=$(find ./target/debug/deps/ -type f -executable)
    CMD="llvm-cov report --use-color --ignore-filename-regex='/.cargo/registry' --instr-profile=./${RUST_COV_DIR}/json5format.profdata"

    for FILE in $FILES
    do
        CMD="$CMD --object $FILE"
    done
    CMD="$CMD --show-instantiation-summary --Xdemangler=rustfilt | less -R"

    eval $CMD
}


generate_report() {
    dependency_check grcov

    mkdir -p ./${COVERAGE_OUT_DIR}/

    grcov \
          ${GRCOV_ARG} \
          --binary-path ${COV_BINARY_DIR} \
          --source-dir . \
          --output-type ${OUTPUT_TYPE} \
          --branch \
          --ignore-not-existing \
          --ignore "*iceoryx2-cli*" \
          --ignore "*iceoryx2-ffi*" \
          --ignore "*build.rs" \
          --ignore "*tests*" \
          --ignore "*testing*" \
          --ignore "*examples*" \
          --ignore "*benchmarks*" \
          --ignore "*target*" \
          --ignore "*.cargo*" \
          --llvm-path ${LLVM_PATH} \
          --output-path ${COVERAGE_OUT}
}

show_help() {
    echo "$0 [OPTIONS]"
    echo
    echo "-c|--clean                -   cleanup all reports"
    echo "-g|--generate             -   generate coverage report"
    echo "-o|--overview             -   show overview of coverage report"
    echo "-r|--report               -   show detailed report"
    echo "-l|--lcov                 -   creates lcov report"
    echo "-t|--html                 -   creates html report"
    echo "-f|--full                 -   generate coverage report and create html and lcov"
    echo
    exit 1
}

if [[ $# == 0 ]]; then
    show_help
fi

while [[ $# -gt 0 ]]; do
    case $1 in
        -c|--clean)
            CLEAN=1
            shift
            ;;
        -g|--generate)
            GENERATE=1
            shift
            ;;
        -o|--overview)
            OVERVIEW=1
            shift
            ;;
        -r|--report)
            REPORT=1
            shift
            ;;
        -f|--full)
            GENERATE=1
            LCOV=1
            HTML=1
            shift
            ;;
        -l|--lcov)
            LCOV=1
            shift
            ;;
        -t|--html)
            HTML=1
            shift
            ;;
        *)
            show_help
            ;;
    esac
done

if [[ $CLEAN == "1" ]]; then
    cleanup
fi

if [[ $GENERATE == "1" ]]; then
    generate_full_profile
fi

if [[ $OVERVIEW == "1" ]]; then
    show_overview
fi

if [[ $REPORT == "1" ]]; then
    show_report
fi

if [[ $LCOV == "1" ]]; then
    OUTPUT_TYPE=lcov
    COV_BINARY_DIR=target/debug
    mkdir -p ${COVERAGE_OUT_DIR}/lcov
    COVERAGE_OUT=${COVERAGE_OUT_DIR}/lcov/lcov_rust.info
    GRCOV_ARG="**/${LLVM_PROFILE_PATH} **/**/${LLVM_PROFILE_PATH}"
    generate_report
    COV_BINARY_DIR=${CMAKE_COV_DIR}
    COVERAGE_OUT=${COVERAGE_OUT_DIR}/lcov/lcov_cpp.info
    GRCOV_ARG=${CMAKE_COV_DIR}
    generate_report
fi

if [[ $HTML == "1" ]]; then
    OUTPUT_TYPE=html
    COV_BINARY_DIR=target/debug
    mkdir -p ${COVERAGE_OUT_DIR}/html
    COVERAGE_OUT=${COVERAGE_OUT_DIR}/html/rust
    GRCOV_ARG="**/${LLVM_PROFILE_PATH} **/**/${LLVM_PROFILE_PATH}"
    generate_report
    COVERAGE_OUT=${COVERAGE_OUT_DIR}/html/cpp
    COV_BINARY_DIR=${CMAKE_COV_DIR}
    GRCOV_ARG=${CMAKE_COV_DIR}
    generate_report
    echo "The Report for Rust Code in iceoryx2 is located at: ${COVERAGE_OUT_DIR}/html/rust/index.html"
    echo "The Report for C++ Code in iceoryx2 is located at: ${COVERAGE_OUT_DIR}/html/cpp/index.html"
fi
