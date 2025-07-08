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

COLOR_RESET='\033[0m'
COLOR_GREEN='\033[1;32m'
COLOR_YELLOW='\033[1;33m'

cd $(git rev-parse --show-toplevel)

RET_VAL=0

check_license_header() {
    FILES=$(find . -type f -iwholename "${FILE_SUFFIX}" -not -path "./target/*" -not -path "./.env/*" )
    let HEAD_LEN=$START_POS+12
    let YEAR_LINE=$START_POS+1

    for FILE in $FILES
    do
        HAS_CORRECT_HEADER=$(diff <(head -n $HEAD_LEN $FILE | tail -n 11 | sed "s/$COMMENT_SYMBOL\(.*\)/\1/") <(head -n 12 internal/scripts/copyright_header.template | tail -n 11) | wc -l)
        HAS_CORRECT_HEADER_YEAR_LINE=$(head -n $YEAR_LINE $FILE | grep -E "^$COMMENT_SYMBOL_GREP Copyright \(c\) 20[2-9][0-9] Contributors to the Eclipse Foundation\$" | wc -l)
        if [[ "$HAS_CORRECT_HEADER_YEAR_LINE" == "0" ]]
        then
            HAS_CORRECT_HEADER_YEAR_LINE=$(head -n 1 $FILE | grep -E "^$COMMENT_SYMBOL_GREP Copyright \(c\) 20[2-9][0-9] - 20[2-9][0-9] Contributors to the Eclipse Foundation\$" | wc -l)
        fi

        if [[ "$HAS_CORRECT_HEADER" != "0" ]] || [[ "$HAS_CORRECT_HEADER_YEAR_LINE" != "1" ]]
        then
            echo "The file '$FILE' has a wrong license header."
            RET_VAL=1
        fi
    done
}

check_rust() {
    START_POS=0
    FILE_SUFFIX="*.rs"
    COMMENT_SYMBOL="\/\/"
    COMMENT_SYMBOL_GREP="//"
    check_license_header
}

check_shell() {
    START_POS=1 # first line is #!/bin/bash
    FILE_SUFFIX="*.sh"
    COMMENT_SYMBOL="#"
    COMMENT_SYMBOL_GREP="#"
    check_license_header
}

check_expect() {
    START_POS=1 # first line is #!/bin/bash
    FILE_SUFFIX="*.exp"
    COMMENT_SYMBOL="#"
    COMMENT_SYMBOL_GREP="#"
    check_license_header
}

check_toml() {
    START_POS=0
    FILE_SUFFIX="*.toml"
    COMMENT_SYMBOL="#"
    COMMENT_SYMBOL_GREP="#"
    check_license_header
}

check_python() {
    FILE_SUFFIX="*.py"
    COMMENT_SYMBOL="#"
    COMMENT_SYMBOL_GREP="#"
    check_license_header
}

check_c_cpp() {
    START_POS=0
    FILE_SUFFIX="*.h"
    COMMENT_SYMBOL="\/\/"
    COMMENT_SYMBOL_GREP="//"
    check_license_header
    FILE_SUFFIX="*.h.in"
    check_license_header
    FILE_SUFFIX="*.c"
    check_license_header
    FILE_SUFFIX="*.hpp"
    check_license_header
    FILE_SUFFIX="*.hpp.in"
    check_license_header
    FILE_SUFFIX="*.inl"
    check_license_header
    FILE_SUFFIX="*.cpp"
    check_license_header
}

check_cmake() {
    START_POS=0
    FILE_SUFFIX="*.cmake"
    COMMENT_SYMBOL="#"
    COMMENT_SYMBOL_GREP="#"
    check_license_header
    FILE_SUFFIX="*.cmake.in"
    check_license_header
    FILE_SUFFIX="*CMakeLists.txt"
    check_license_header
}

check_bazel() {
    START_POS=0
    FILE_SUFFIX="*.bazel"
    COMMENT_SYMBOL="#"
    COMMENT_SYMBOL_GREP="#"
    check_license_header
}

check_rust
check_shell
check_expect
check_c_cpp
check_cmake
check_bazel
check_python

# no toml check for now
# it is usually only some configuration files which can be used without copyright notice
# check_toml

if [[ "$RET_VAL" == "0" ]]
then
    echo -e "${COLOR_GREEN}All checked files have a valid license header${COLOR_RESET}"
else
    echo -e "${COLOR_YELLOW}The listed files don't have a valid license header${COLOR_RESET}"
fi

exit $RET_VAL
