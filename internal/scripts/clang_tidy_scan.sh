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

# This script checks code files with clang-tidy
# Example usage: ./tools/scripts/clang_tidy_check.sh full|hook|ci_pull_request

set -e

COLOR_OFF='\033[0m'
COLOR_RED='\033[1;31m'
COLOR_GREEN='\033[1;32m'
COLOR_YELLOW='\033[1;33m'

DIRECTORIES_TO_SCAN="iceoryx2* examples benchmarks"
FILE_FILTER="\.(h|hh|hpp|hxx|inl|c|cc|cpp|cxx)$"
FILES_TO_SCAN=""
WARN_MODE_PARAM=""

DIRECTORIES_MODE=false
FILES_MODE=false
DIFF_TO_MAIN_MODE=false
MAIN_ORIGIN="origin"
FULL_MODE=false
CACHED_COMMIT_MODE=false
MODIFIED_MODE=false

NUM_JOBS=1
if [[ "$OSTYPE" == "linux-gnu"* ]] || [[ "$OSTYPE" == "cygwin" ]] || [[ "$OSTYPE" == "msys" ]]; then
    NUM_JOBS=$(nproc)
elif [[ "$OSTYPE" == "darwin"* ]]; then
    NUM_JOBS=$(sysctl -n hw.ncpu)
fi

while (( "$#" )); do
    case "$1" in
        --directories)
            DIRECTORIES_TO_SCAN=$2
            DIRECTORIES_MODE=true
            shift 2
            ;;
        --files)
            FILES_TO_SCAN=$2
            FILES_MODE=true
            shift 2
            ;;
        --main-origin)
            MAIN_ORIGIN="$2"
            shift 2
            ;;
        cached-commit)
            CACHED_COMMIT_MODE=true
            shift 1
            ;;
        diff-to-main)
            DIFF_TO_MAIN_MODE=true
            shift 1
            ;;
        full)
            FULL_MODE=true
            shift 1
            ;;
        modified)
            MODIFIED_MODE=true
            shift 1
            ;;
        warning-as-error)
            WARN_MODE_PARAM="--warnings-as-errors=*"
            shift 1
            ;;
        "help")
            echo "Script to run clang-tidy with all available cores"
            echo ""
            echo "Options:"
            echo "    --directories         Scan all files from the specified directories"
            echo "                          Multiple directories must be enclosed in quotes"
            echo "                          e.g. --directories \"dir1 dir2 dir3\""
            echo "    --files               Scan all specified files"
            echo "                          Multiple files must be enclosed in quotes"
            echo "                          e.g. --files \"file1 file2 file3\""
            echo "    --main-origin         The origin of main. By default \"origin\""
            echo "Args:"
            echo "    cached-commit         Scan all modified and added files which are cached for a commit"
            echo "    diff-to-main          Scan all modified towards HEAD of main branch"
            echo "    full                  Scan all versioned files from [$DIRECTORIES_TO_SCAN]"
            echo "    help                  Print this help"
            echo "    modified              Scan all modified, added and untracked files from the git repo"
            echo "    warning-as-error      Treat warnings as errors"
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

# exit no relevant files need to be scanned
MODIFIED_FILES=""
ADDED_FILES=""
FILE_LIST=""
if [[ $FILES_MODE == true || $DIFF_TO_MAIN_MODE == true ]]; then
    if [[ $DIFF_TO_MAIN_MODE == true ]]; then
        FILES_TO_SCAN=$(git diff --name-only --diff-filter=AM ${MAIN_ORIGIN}/main HEAD)
    fi

    SEPARATOR=''
    for FILE in ${FILES_TO_SCAN}; do
        if [[ $FILE =~ $FILE_FILTER ]]; then
            FILE_LIST+="${SEPARATOR}${FILE}"
            SEPARATOR=$'\n'
        fi
    done

    FILE_LIST_ARRAY=(${FILE_LIST})
    NUMBER_OF_FILES=${#FILE_LIST_ARRAY[@]}

    if [[ ${NUMBER_OF_FILES} -eq 0 ]]; then
        echo -e "${COLOR_YELLOW}-> nothing to do${COLOR_OFF}"
        exit 0
    fi
elif [[ $CACHED_COMMIT_MODE == true ]]; then
    MODIFIED_FILES=$(git diff --cached --name-only --diff-filter=CMRT | grep -E "$FILE_FILTER" | cat)
    MODIFIED_FILES_ARRAY=(${MODIFIED_FILES})
    NUMBER_OF_MODIFIED_FILES=${#MODIFIED_FILES_ARRAY[@]}

    ADDED_FILES=$(git diff --cached --name-only --diff-filter=A | grep -E "$FILE_FILTER" | cat)
    ADDED_FILES_ARRAY=(${ADDED_FILES})
    NUMBER_OF_ADDED_FILES=${#ADDED_FILES_ARRAY[@]}

    if [[ ${NUMBER_OF_MODIFIED_FILES} -eq 0 && ${NUMBER_OF_ADDED_FILES} -eq 0 ]]; then
        echo -e "${COLOR_YELLOW}-> nothing to do${COLOR_OFF}"
        exit 0
    fi
elif [[ $MODIFIED_MODE == true ]]; then
    MODIFIED_FILES=$(git status --porcelain | grep '^[ AM?]'| grep -E "$FILE_FILTER" | sed 's/^.\{2\} //')
    MODIFIED_FILES_ARRAY=(${MODIFIED_FILES})
    NUMBER_OF_MODIFIED_FILES=${#MODIFIED_FILES_ARRAY[@]}

    if [[ ${NUMBER_OF_MODIFIED_FILES} -eq 0 ]]; then
        echo -e "${COLOR_YELLOW}-> nothing to do${COLOR_OFF}"
        exit 0
    fi
fi

# we have to ensure that everything is build otherwise clang-tidy may not find some header
echo -e "${COLOR_YELLOW}Building iceoryx-ffi and C/C++ bindings as preparation for clang-tidy${COLOR_OFF}"
export CXX=clang++
export CC=clang

rm -rf target/ff/iceoryx
${WORKSPACE}/internal/scripts/ci_build_and_install_iceoryx_hoofs.sh
cargo build --package iceoryx2-ffi-c
cmake -S . -B target/clang-tidy-scan \
    -DCMAKE_PREFIX_PATH="$(pwd)/target/ff/iceoryx/install" \
    -DRUST_BUILD_ARTIFACT_PATH="$(pwd)/target/debug" \
    -DCMAKE_BUILD_TYPE=Debug \
    -DBUILD_CXX=ON \
    -DBUILD_EXAMPLES=ON \
    -DBUILD_TESTING=ON
cmake --build target/clang-tidy-scan -j$NUM_JOBS

echo "Using clang-tidy version: $(clang-tidy --version | sed -n "s/.*version \([0-9.]*\)/\1/p" )"

noSpaceInSuppressions=$(git ls-files | grep -E "$FILE_FILTER" | xargs -I {} grep -h '// NOLINTNEXTLINE (' {} || true)
if [[ -n "$noSpaceInSuppressions" ]]; then
    echo -e "${COLOR_RED}Remove space between NOLINTNEXTLINE and '('!${COLOR_OFF}"
    echo "$noSpaceInSuppressions"
    exit 1
fi

# Function to clean up background processes
cleanup() {
    echo "Cleaning up..."
    pkill -9 clang-tidy
    exit
}

# Trap EXIT signal to call cleanup
trap cleanup EXIT

function scan() {
    FILES=$1
    FILES_ARRAY=(${FILES})
    NUMBER_OF_FILES=${#FILES_ARRAY[@]}

    if [[ ${NUMBER_OF_FILES} -eq 0 ]]; then
        echo -e "${COLOR_YELLOW}-> nothing to do${COLOR_OFF}"
        return 0
    fi

    echo -e "${COLOR_GREEN}Processing files ...${COLOR_OFF}"
    MAX_CONCURRENT_EXECUTIONS=$NUM_JOBS
    CURRENT_CONCURRENT_EXECUTIONS=0
    echo "Concurrency set to '${MAX_CONCURRENT_EXECUTIONS}'"
    FILE_COUNTER=1
    for FILE in $FILES; do
        # run multiple clang-tidy instances concurrently
        if [[ ${CURRENT_CONCURRENT_EXECUTIONS} -ge ${MAX_CONCURRENT_EXECUTIONS} ]]; then
            wait -n # wait for one of the background processes to finish
            CURRENT_CONCURRENT_EXECUTIONS=$((CURRENT_CONCURRENT_EXECUTIONS - 1))
        fi

        echo -e "${COLOR_GREEN}[${FILE_COUNTER}/${NUMBER_OF_FILES}]${COLOR_OFF} ${FILE}"
        FILE_COUNTER=$((FILE_COUNTER + 1))

        if test -f "$FILE"; then
            EXTRA_ARG=""
            if [[ "$FILE" == iceoryx2-pal/posix/* ]]; then
                EXTRA_ARG="--extra-arg=-xc"
            fi
            SECONDS_START=${SECONDS}
            $(clang-tidy ${WARN_MODE_PARAM} --quiet -p target/clang-tidy-scan ${FILE} ${EXTRA_ARG} >&2 \
            || exit $? \
            && echo echo -e "${COLOR_YELLOW} $((${SECONDS}-${SECONDS_START}))s${COLOR_OFF} to scan '${FILE}'") &
            CURRENT_CONCURRENT_EXECUTIONS=$((CURRENT_CONCURRENT_EXECUTIONS + 1))
        else
            echo -e "${COLOR_RED}File does not exist! Aborting!${COLOR_OFF}"
            return 1
        fi
    done
    # wait on each background process individually to abort script when a process exits with an error
    while [[ ${CURRENT_CONCURRENT_EXECUTIONS} -ne 0 ]]; do
        wait -n # wait for one of the background processes to finish
        CURRENT_CONCURRENT_EXECUTIONS=$((CURRENT_CONCURRENT_EXECUTIONS - 1))
    done

    echo -e "${COLOR_GREEN}... done!${COLOR_OFF}"
}

if [[ $FULL_MODE == true || $DIRECTORIES_MODE == true ]]; then
    FILES=$(find ${DIRECTORIES_TO_SCAN} -type f | grep -E ${FILE_FILTER} | sort | uniq)
    echo ""
    echo "Checking files in [${DIRECTORIES_TO_SCAN}]"
    scan "${FILES}"
elif [[ $FILES_MODE == true || $DIFF_TO_MAIN_MODE == true ]]; then
    echo ""
    echo "Checking files from provided list"
    scan "$FILE_LIST"
elif [[ $CACHED_COMMIT_MODE == true || $MODIFIED_MODE == true ]]; then
    echo ""
    echo "Checking modified files"
    scan "${MODIFIED_FILES}"

    # List only added files
    echo ""
    echo "Checking added files"
    scan "${ADDED_FILES}"
fi

trap - EXIT
