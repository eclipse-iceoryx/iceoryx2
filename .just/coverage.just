# Copyright (c) 2026 Contributors to the Eclipse Foundation
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

# =============================================================================
# Coverage Recipes
# =============================================================================

COLOR_OFF    := "\\033[0m"
COLOR_GREEN  := "\\033[1;32m"
COLOR_YELLOW := "\\033[1;33m"

LLVM_PROFILE_PATH := "target/debug/llvm-profile-files"
RUST_COV_DIR      := "target/debug/rust_cov_build"
CMAKE_COV_DIR     := "target/ff/cc/cmake_cov_build"
COVERAGE_OUT_DIR  := "target/ff/coverage"

# Paths excluded from all coverage reports
COVERAGE_IGNORE := `echo \
    "_deps"           \
    "benchmarks"      \
    "component-tests" \
    "examples"        \
    "iceoryx2-cli"    \
    "iceoryx2-ffi"    \
    "tests"           \
    "tests-nostd"     \
    "testing"         \
    "build.rs"`

[private]
_parse-coverage-args *flags:
    #!/usr/bin/env bash
    set -euo pipefail

    HAS_HTML="false"
    HAS_LCOV="false"
    HAS_DETAILED="false"
    PACKAGES=()

    set -- {{flags}}
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --html)      HAS_HTML="true";        shift ;;
            --lcov)      HAS_LCOV="true";        shift ;;
            --detailed)  HAS_DETAILED="true";    shift ;;
            --package)   PACKAGES+=("$2");       shift 2 ;;
            *)           shift ;;
        esac
    done

    echo "HAS_HTML='$HAS_HTML'"
    echo "HAS_LCOV='$HAS_LCOV'"
    echo "HAS_DETAILED='$HAS_DETAILED'"
    echo "PACKAGES=(${PACKAGES[*]@Q})"

[private]
_coverage-dispatch action *flags:
    #!/usr/bin/env bash
    set -euo pipefail

    eval "$(just _parse-coverage-args {{flags}})"

    case "{{action}}" in
        "")
            echo "Usage: just coverage <command> [flags]"
            echo ""
            echo "Commands:"
            echo "  generate-profile                      generate coverage profile"
            echo "  generate-report --html|--lcov         generate rendered report"
            echo "  summarize [--detailed] [--package X]  show coverage summary"
            echo "  clean                                 remove all coverage data"
            exit 1
            ;;
        generate-profile)
            if [ "${#PACKAGES[@]}" -gt 0 ]; then
                echo "error: '--package' is not supported for 'just coverage generate-profile'" >&2; exit 1
            fi
            just _coverage-profile
            ;;
        generate-report)
            if [ "${#PACKAGES[@]}" -gt 0 ]; then
                echo "error: '--package' is not supported for 'just coverage generate-report'" >&2; exit 1
            fi
            if [ "$HAS_HTML" = "false" ] && [ "$HAS_LCOV" = "false" ]; then
                echo "error: 'just coverage report' requires --html, --lcov, or both" >&2
                exit 1
            fi
            if [ "$HAS_HTML" = "true" ]; then just _coverage-generate-report-html; fi
            if [ "$HAS_LCOV" = "true" ]; then just _coverage-generate-report-lcov; fi
            ;;
        summarize)
            just _coverage-summarize "$HAS_DETAILED" "${PACKAGES[@]}"
            ;;
        clean)
            just _coverage-clean
            ;;
        *)
            echo "error: unknown coverage action '{{action}}'" >&2; exit 1
            ;;
    esac

# --- profile -----------------------------------------------------------------

[private]
_coverage-profile:
    #!/usr/bin/env bash
    set -euo pipefail

    mkdir -p "{{CMAKE_COV_DIR}}"
    just _coverage-profile-cmake
    just _coverage-profile-rust

[private]
_coverage-profile-cmake:
    #!/usr/bin/env bash
    set -euo pipefail

    cmake . -B"{{CMAKE_COV_DIR}}" -DCOVERAGE=ON -DBUILD_TESTING=ON -DCMAKE_BUILD_TYPE=Debug
    cmake --build "{{CMAKE_COV_DIR}}" -j
    ctest --test-dir "{{CMAKE_COV_DIR}}" --output-on-failure

[private]
_coverage-profile-rust:
    #!/usr/bin/env bash
    set -euo pipefail

    export LLVM_PROFILE_FILE="{{LLVM_PROFILE_PATH}}/iceoryx2-%p-%m.profraw"
    export DEBUGINFOD_URLS=/dev/null

    if [[ "$(rustc --version | grep nightly | wc -l)" == "1" ]]; then
        echo -e "{{COLOR_GREEN}}rust nightly compiler found, activating MC/DC coverage check{{COLOR_OFF}}"
        export RUSTFLAGS="-C instrument-coverage -Z coverage-options=condition"
    else
        echo -e "{{COLOR_YELLOW}}no rust nightly compiler found, MC/DC coverage is not available only line coverage{{COLOR_OFF}}"
        export RUSTFLAGS="-C instrument-coverage"
    fi
    just test workspace

# --- generate-report ---------------------------------------------------------

[private]
_coverage-generate-report-html:
    #!/usr/bin/env bash
    set -euo pipefail

    mkdir -p "{{COVERAGE_OUT_DIR}}/html/cpp"

    just _coverage-generate-report-impl \
        "html" \
        "{{COVERAGE_OUT_DIR}}/html/rust" \
        "--html-details" \
        "{{COVERAGE_OUT_DIR}}/html/cpp/index.html"
    echo "Rust HTML report: {{COVERAGE_OUT_DIR}}/html/rust/index.html"
    echo "C++ HTML report:  {{COVERAGE_OUT_DIR}}/html/cpp/index.html"

[private]
_coverage-generate-report-lcov:
    #!/usr/bin/env bash
    set -euo pipefail

    mkdir -p "{{COVERAGE_OUT_DIR}}/lcov"

    just _coverage-generate-report-impl \
        "lcov" \
        "{{COVERAGE_OUT_DIR}}/lcov/iceoryx2_lcov_rust.info" \
        "--lcov" \
        "{{COVERAGE_OUT_DIR}}/lcov/iceoryx2_lcov_cpp.info"

[private]
_coverage-generate-report-impl output_type grcov_output gcovr_flag gcovr_output:
    #!/usr/bin/env bash
    set -euo pipefail

    if ! which grcov &>/dev/null; then
        echo "error: 'grcov' not found. Aborting!" >&2; exit 1
    fi
    if ! which gcovr &>/dev/null; then
        echo "error: 'gcovr' not found. Aborting!" >&2; exit 1
    fi

    LLVM_PATH=$(dirname "$(just _find-llvm-tool llvm-profdata)")

    echo "Generating Rust coverage report ..."

    GRCOV_IGNORE_FLAGS=(--ignore "*target*" --ignore "*.cargo*")
    for pattern in {{COVERAGE_IGNORE}}; do
        GRCOV_IGNORE_FLAGS+=(--ignore "*${pattern}*")
    done

    grcov \
        **/{{LLVM_PROFILE_PATH}} \
        **/**/{{LLVM_PROFILE_PATH}} \
        --binary-path target/debug \
        --source-dir . \
        --output-type {{output_type}} \
        --branch \
        --ignore-not-existing \
        "${GRCOV_IGNORE_FLAGS[@]}" \
        --llvm-path "$LLVM_PATH" \
        --output-path "{{grcov_output}}"

    echo "Generating C++ coverage report ..."

    GCOVR_IGNORE_FLAGS=(-e '/target/' -e '/\.cargo/')
    for pattern in {{COVERAGE_IGNORE}}; do
        GCOVR_IGNORE_FLAGS+=(-e "${pattern}")
    done

    # We use here https://github.com/gcovr/gcovr to handle the generated files by gcov
    # License: https://github.com/gcovr/gcovr/blob/main/LICENSE.txt
    gcovr "{{gcovr_flag}}" "{{gcovr_output}}" "{{CMAKE_COV_DIR}}" \
        "${GCOVR_IGNORE_FLAGS[@]}"

# --- summarize ---------------------------------------------------------------

[private]
_coverage-summarize detailed *packages:
    #!/usr/bin/env bash
    set -euo pipefail

    if [ "{{detailed}}" = "true" ] && ! which rustfilt &>/dev/null; then
        echo "error: 'rustfilt' not found. Aborting!" >&2; exit 1
    fi

    LLVM_COV=$(just _find-llvm-tool llvm-cov)

    just _coverage-merge-profiles

    FILES=$(find ./target/debug/deps/ -type f -executable)

    IGNORE_PATTERNS=('/\.cargo/registry' '/target/')
    for pattern in {{COVERAGE_IGNORE}}; do
        IGNORE_PATTERNS+=("$pattern")
    done
    IGNORE_REGEX=$(IFS='|'; echo "${IGNORE_PATTERNS[*]}")

    CMD="$LLVM_COV report --use-color --ignore-filename-regex='$IGNORE_REGEX' --instr-profile=./{{RUST_COV_DIR}}/json5format.profdata"
    for FILE in $FILES; do
        CMD="$CMD --object $FILE"
    done
    for pkg in {{packages}}; do
        PKG_SRC_DIR=$(just _find-crate-src "$pkg")
        CMD="$CMD --sources $PKG_SRC_DIR"
    done
    if [ "{{detailed}}" = "true" ]; then
        CMD="$CMD --show-instantiation-summary --Xdemangler=rustfilt | less -R"
    fi
    eval $CMD

[private]
_coverage-merge-profiles:
    #!/usr/bin/env bash
    set -euo pipefail

    if [[ ! -f "./{{RUST_COV_DIR}}/json5format.profdata" ]]; then
        LLVM_PROFDATA=$(just _find-llvm-tool llvm-profdata)
        mkdir -p "./{{RUST_COV_DIR}}/"
        FILES=$(find . -name "*profraw")
        "$LLVM_PROFDATA" merge --sparse $FILES -o "./{{RUST_COV_DIR}}/json5format.profdata"
    fi

# --- clean -------------------------------------------------------------------

[private]
_coverage-clean:
    find . -name "*profraw" -exec rm {} \;
    cargo clean


# --- helpers --------------------------------------------------------------

[private]
_find-llvm-tool tool:
    #!/usr/bin/env bash
    set -euo pipefail

    RUSTC_LLVM_VERSION=$(rustc --version --verbose | grep -oP 'LLVM version: \K[0-9]+')
    RUST_SYSROOT=$(rustc --print sysroot)
    HOST=$(rustc -vV | sed -n 's|host: ||p')

    # Check for llvm-tools in the rust sysroot
    if [ -f "${RUST_SYSROOT}/lib/rustlib/${HOST}/bin/{{tool}}" ]; then
        echo "${RUST_SYSROOT}/lib/rustlib/${HOST}/bin/{{tool}}"

    # Check for versioned system binary matching rustc llvm version
    elif which "{{tool}}-${RUSTC_LLVM_VERSION}" &>/dev/null; then
        which "{{tool}}-${RUSTC_LLVM_VERSION}"

    # Check for unversioned system binary, validate version matches before using
    elif which "{{tool}}" &>/dev/null; then
        LLVM_VERSION=$({{tool}} --version | grep -oP 'LLVM version \K[0-9]+')
        if [[ "$LLVM_VERSION" -ne "$RUSTC_LLVM_VERSION" ]]; then
            echo "{{tool}} LLVM version: $LLVM_VERSION" >&2
            echo "rustc LLVM version: $RUSTC_LLVM_VERSION" >&2
            echo "error: LLVM major versions do not match" >&2; exit 1
        fi
        which "{{tool}}"
    else
        echo "error: '{{tool}}' not found. Aborting!" >&2; exit 1
    fi

