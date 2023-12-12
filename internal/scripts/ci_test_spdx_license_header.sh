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

#!/bin/bash

cd $(git rev-parse --show-toplevel)

RET_VAL=0

check_license_header() {
    FILES=$(find . -type f -iwholename "${FILE_SUFFIX}" -not -path "./target/*" )

    for FILE in $FILES
    do
        HAS_CORRECT_HEADER=$(diff <(head -n 12 $FILE | tail -n 11 | sed "s/$COMMENT_SYMBOL\(.*\)/\1/") <(head -n 12 internal/scripts/copyright_header.template | tail -n 11) | wc -l)
        HAS_CORRECT_HEADER_YEAR_LINE=$(head -n 1 $FILE | grep -E "^$COMMENT_SYMBOL_GREP Copyright \(c\) 20[2-9][0-9] Contributors to the Eclipse Foundation\$" | wc -l)

        if [[ "$HAS_CORRECT_HEADER" != "0" ]] || [[ "$HAS_CORRECT_HEADER_YEAR_LINE" != "1" ]]
        then
            echo "The $FILE has a wrong license header."
            RET_VAL=1
        fi
    done
}

check_rust() {
    FILE_SUFFIX="*.rs"
    COMMENT_SYMBOL="\/\/"
    COMMENT_SYMBOL_GREP="//"
    check_license_header
}

check_shell() {
    FILE_SUFFIX="*.sh"
    COMMENT_SYMBOL="#"
    COMMENT_SYMBOL_GREP="#"
    check_license_header
}

check_toml() {
    FILE_SUFFIX="*.toml"
    COMMENT_SYMBOL="#"
    COMMENT_SYMBOL_GREP="#"
    check_license_header
}

check_rust
check_shell

# no toml check for now
# it is usually only some configuration files which can be used without copyright notice
# check_toml

exit $RET_VAL
