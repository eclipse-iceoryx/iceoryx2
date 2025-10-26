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

FILE_FILTER="(\.(rs|h|h.in|hh|hh.in|hpp|hpp.in|hxx|hxx.in|inl|c|cc|cpp|cxx|cmake|cmake.in|bazel|py|sh|exp)|CMakeLists.txt)$"

COLOR_RESET='\033[0m'
COLOR_GREEN='\033[1;32m'
COLOR_YELLOW='\033[1;33m'

cd $(git rev-parse --show-toplevel)

RET_VAL=0

FILES=$(find . -type f -not -path "./git/*" -not -path "./target/*" -not -path "./.env/*" | grep -E ${FILE_FILTER})

for FILE in $FILES
do
    # check that SPDX license identifier is used only once
    SPDX_LICENSE_IDENTIFIER_COUNT=$(echo $(grep "SPDX-License-Identifier: Apache-2.0 OR MIT" --count $FILE))
    if [[ "$SPDX_LICENSE_IDENTIFIER_COUNT" == "0" ]]; then
        echo "The file '$FILE' has no valid SPDX license identifier with 'Aoache-2.0 OR MIT'."
        RET_VAL=1
    elif [[ "$SPDX_LICENSE_IDENTIFIER_COUNT" != "1" ]]; then
        if [[ "$FILE" != "./internal/scripts/ci_test_spdx_license_header.sh" \
            && "$FILE" != "./internal/scripts/set_license_header.sh" ]]; then
            echo "The file '$FILE' has to many SPDX license identifier."
            RET_VAL=1
        fi
    else
        # check that Copyright is set
        LICENSE_HEADER=$(grep "SPDX-License-Identifier: Apache-2.0 OR MIT" -B 100 $FILE)
        if [[ $(echo $( echo $LICENSE_HEADER | grep "Copyright" --count)) == "0" ]]; then
            echo "The file '$FILE' has no 'Copyright' notice."
            RET_VAL=1
        fi
    fi
done

if [[ "$RET_VAL" == "0" ]]
then
    echo -e "${COLOR_GREEN}All checked files have a valid license header${COLOR_RESET}"
else
    echo -e "${COLOR_YELLOW}The listed files don't have a valid license header${COLOR_RESET}"
fi

exit $RET_VAL
