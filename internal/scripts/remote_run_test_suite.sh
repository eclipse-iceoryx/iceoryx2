#!/bin/bash
# Copyright (c) 2025 Contributors to the Eclipse Foundation
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

SCRIPT_START_TIME=$(date +%s)

# Get git root directory
GIT_ROOT=$(git rev-parse --show-toplevel 2>/dev/null)
if [ $? -ne 0 ]; then
    echo -e "${RED}ERROR: This script must be run from within the iceoryx2 repository${NC}"
    exit 1
fi

SCRIPT_PATH="$GIT_ROOT/internal/scripts/$(basename "$0")"

# Parse Arguments
# ---------------
DRY_RUN=false
TEST_DEVICE_ADDR="172.31.1.11:12345"  # Default value
TARGET=""                             # Required
TOOLCHAIN=""                          # Required

while [[ $# -gt 0 ]]; do
    case $1 in
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --device-addr)
            if [[ -n $2 && $2 != --* ]]; then
                TEST_DEVICE_ADDR="$2"
                shift 2
            else
                echo -e "${RED}ERROR: --device-addr requires an address argument${NC}"
                exit 1
            fi
            ;;
        --target)
            if [[ -n $2 && $2 != --* ]]; then
                TARGET="$2"
                shift 2
            else
                echo -e "${RED}ERROR: --target requires a target argument${NC}"
                exit 1
            fi
            ;;
        --toolchain)
            if [[ -n $2 && $2 != --* ]]; then
                TOOLCHAIN="$2"
                shift 2
            else
                echo -e "${RED}ERROR: --toolchain requires a toolchain name argument${NC}"
                exit 1
            fi
            ;;
        --help|-h)
            echo "Usage: $SCRIPT_PATH [OPTIONS]"
            echo ""
            echo "Required Options:"
            echo "  --target TARGET        Set the build target (e.g., x86_64-pc-nto-qnx710, aarch64-unknown-linux-gnu)"
            echo "  --toolchain TOOLCHAIN  Set the Rust toolchain name (e.g., stable, nightly, custom-toolchain)"
            echo ""
            echo "Optional Options:"
            echo "  --dry-run              Show what tests would be run without executing them"
            echo "  --device-addr ADDR     Set the test device address (default: 172.31.1.11:12345)"
            echo "  --help, -h             Show this help message"
            echo ""
            echo "Examples:"
            echo "  $SCRIPT_PATH --target x86_64-pc-nto-qnx710 --toolchain qnx-custom"
            echo "  $SCRIPT_PATH --target aarch64-unknown-nto-qnx710 --toolchain qnx-custom --device-addr 192.168.1.100:8080"
            echo "  $SCRIPT_PATH --target x86_64-pc-nto-qnx710 --toolchain qnx-custom --dry-run"
            echo ""
            echo "This script runs tests remotely on the specified target using remote-test-client"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use $SCRIPT_PATH --help for usage information"
            exit 1
            ;;
    esac
done

# Validate required arguments
if [[ -z "$TARGET" ]]; then
    echo -e "${RED}ERROR: --target is required${NC}"
    echo "Use $SCRIPT_PATH --help for usage information"
    exit 1
fi

if [[ -z "$TOOLCHAIN" ]]; then
    echo -e "${RED}ERROR: --toolchain is required${NC}"
    echo "Use $SCRIPT_PATH --help for usage information"
    exit 1
fi

# Set the environment variable
export TEST_DEVICE_ADDR

# Configuration
# -------------

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

EXCLUDED_PACKAGES=(
    "example"
    "benchmark-*"
    "iceoryx2-cli"
    "iceoryx2-ffi"
    "iceoryx2-ffi-*"
    "iceoryx2-services-*"
    "iceoryx2-tunnels-*"
    "iceoryx2-userland-*"
)

# Validate Dependencies
# ---------------------

if ! rustup toolchain list | grep -q "^${TOOLCHAIN}"; then
    echo -e "${RED}ERROR: Toolchain '${TOOLCHAIN}' not found${NC}"
    echo -e "${RED}Available toolchains:${NC}"
    rustup toolchain list | sed 's/^/  /'
    exit 1
fi

if ! command -v remote-test-client &> /dev/null; then
    echo -e "${RED}ERROR: remote-test-client not found${NC}"
    echo -e "${RED}Make sure remote-test-client is installed and available in PATH${NC}"
    exit 1
fi

if ! command -v jq &> /dev/null; then
    echo -e "${RED}ERROR: jq is required but not installed${NC}"
    exit 1
fi

# Build Test Suite
# ----------------

EXCLUDE_FLAGS=""
for pkg in "${EXCLUDED_PACKAGES[@]}"; do
    EXCLUDE_FLAGS="$EXCLUDE_FLAGS --exclude $pkg"
done

if [ "$DRY_RUN" = true ]; then
    echo -e "${YELLOW}DRY RUN MODE - No tests will be executed${NC}"
    echo ""
fi

echo -e "${BLUE}Configuration:${NC}"
echo -e "  Target: ${TARGET}"
echo -e "  Toolchain: ${TOOLCHAIN}"
echo -e "  Device Address: ${TEST_DEVICE_ADDR}"
echo ""

echo -e "${BLUE}Building tests for workspace...${NC}"
if ! cargo +${TOOLCHAIN} build --target $TARGET --workspace $EXCLUDE_FLAGS --tests; then
    echo -e "${RED}ERROR: Failed to build tests${NC}"
    exit 1
fi

# Run Test Suite
# --------------

# Gather all test executables
mapfile -t TEST_EXECUTABLES < <(cargo +${TOOLCHAIN} test --target $TARGET --no-run --workspace $EXCLUDE_FLAGS --message-format=json 2>/dev/null | \
jq -r 'select(.reason == "compiler-artifact" and .target.kind[] == "test") | .executable')

# Initialize counters
CURRENT=0
PASSED=0
FAILED=0
FAILED_TESTS=()

echo -e "${YELLOW}Found ${#TEST_EXECUTABLES[@]} test executables${NC}"

if [ "$DRY_RUN" = true ]; then
    echo ""
    echo -e "${BLUE}Tests that would be executed:${NC}"
    for test_executable in "${TEST_EXECUTABLES[@]}"; do
        ((CURRENT++))
        TEST_NAME=$(basename "$test_executable")
        echo -e "  ${BLUE}[$CURRENT]${NC} $TEST_NAME"
    done
    
    echo ""
    echo -e "${YELLOW}Dry run completed. $CURRENT tests would be executed.${NC}"
    exit 0
fi

set +e  # Don't exit on any error

# Loop all discovered test executables
for test_executable in "${TEST_EXECUTABLES[@]}"; do
    ((CURRENT++))
    TEST_NAME=$(basename "$test_executable")
    
    echo ""
    echo -e "${BLUE}========================================${NC}"
    echo -e "${BLUE}[$CURRENT/${#TEST_EXECUTABLES[@]}] Running: $TEST_NAME${NC}"
    echo -e "${BLUE}========================================${NC}"
    
    # Run the test and capture output
    TEST_START_TIME=$(date +%s)
    TEST_OUTPUT=$(remote-test-client run 0 "$test_executable" 2>&1)
    EXIT_CODE=$?
    TEST_END_TIME=$(date +%s)
    TEST_DURATION=$((TEST_END_TIME - TEST_START_TIME))
    
    # Print output
    echo "$TEST_OUTPUT"
    
    # Show result
    echo ""
    if [ $EXIT_CODE -eq 0 ]; then
        echo -e "${GREEN}✓ PASSED: $TEST_NAME (${TEST_DURATION}s)${NC}"
        ((PASSED++))
    else
        echo -e "${RED}✗ FAILED: $TEST_NAME (${TEST_DURATION}s, exit code: $EXIT_CODE)${NC}"
        if echo "$TEST_OUTPUT" | grep -q "TcpStream::connect.*failed"; then
            echo -e "${YELLOW}HINT"
            echo -e "${YELLOW}  * Ensure the target is reachable at the specified address: ${TEST_DEVICE_ADDR}${NC}"
            echo -e "${YELLOW}  * Ensure the server is running on the target: remote-test-server -v --bind 0.0.0.0:12345 --sequential${NC}"
            exit 1
        fi
        ((FAILED++))
        FAILED_TESTS+=("$TEST_NAME")
    fi

    echo ""
    PROGRESS_PERCENT=$((CURRENT * 100 / ${#TEST_EXECUTABLES[@]}))
    echo -e "${YELLOW}Progress: ${PROGRESS_PERCENT}%${NC}"
done

SCRIPT_END_TIME=$(date +%s)
TOTAL_DURATION=$((SCRIPT_END_TIME - SCRIPT_START_TIME))

# Print Summary
# -------------

echo ""
echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}SUMMARY${NC}"
echo -e "${BLUE}========================================${NC}"
echo -e "Target: $TARGET"
echo -e "Toolchain: $TOOLCHAIN"
echo -e "Total tests: $CURRENT"
echo -e "Total execution time: ${TOTAL_DURATION}s"
echo -e "${GREEN}Passed: $PASSED${NC}"
echo -e "${RED}Failed: $FAILED${NC}"

if [ $FAILED -gt 0 ]; then
    echo ""
    echo -e "${RED}FAILED TESTS:${NC}"
    for test in "${FAILED_TESTS[@]}"; do
        echo -e "  ${RED}✗${NC} $test"
    done
    exit 1
else
    echo -e "${GREEN}🎉 All tests passed!${NC}"
fi
