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

cd $(git rev-parse --show-toplevel)

RETVAL=0
FILES=$(find . -iwholename "*/tests/*.rs")
NUMBER_OF_FILES=0
FAILED_FILES=0
VIOLATIONS=0
CORRECT_USAGE=0

for f in $FILES
do
    HAS_ASSERT=$(grep "assert!\|assert_eq!\|assert_ne!" $f | wc -l)

    let VIOLATIONS=${VIOLATIONS}+${HAS_ASSERT}
    let CORRECT_USAGE=${CORRECT_USAGE}+$(grep "assert_that!" $f | wc -l)

    FAIL_FILE=0

    if [ ${HAS_ASSERT} != 0 ]
    then
        echo "Please use only the assert_that macro in tests. The file uses the rust default assert: ${f}"
        RETVAL=-1
        FAIL_FILE=1
    fi

    USE_ASSERT_THAT_INCORRECT=$(grep "assert_that!" $f | grep "is_ok()\|is_err()\|is_some()\|is_none()\|is_empty()" | wc -l )
    if [ ${USE_ASSERT_THAT_INCORRECT} != 0 ]
    then
        echo "Please use the modifier 'is_ok', 'is_err', 'is_some', 'is_none', 'is_empty', 'is_not_empty' instead of calling the methods directly and compare them with true/false: ${f}"
        RETVAL=-1
        FAIL_FILE=1
    fi

    USE_ASSERT_THAT_INCORRECT=$(grep "assert_that!.*==\|assert_that!.*!=\|assert_that!.*<=\|assert_that!.*<\|assert_that!.*>=\|assert_that!.*>" $f | wc -l )
    if [ ${USE_ASSERT_THAT_INCORRECT} != 0 ]
    then
        echo "Please use the modifier 'eq', 'ne', 'le', 'lt', 'ge', 'gt' directly instead of calling '==', '!=', '<=', '<', '>=', '>' and compare them with true/false: ${f}"
        RETVAL=-1
        FAIL_FILE=1
    fi

    USE_ASSERT_THAT_INCORRECT=$(grep "assert_that!" $f | grep "%" | wc -l )
    if [ ${USE_ASSERT_THAT_INCORRECT} != 0 ]
    then
        echo "Please use the modifier 'mod \$RHS, is \$RESULT' directly instead of calling '\$LHS % \$RHS == \$RESULT' and compare it with true/false: ${f}"
        RETVAL=-1
        FAIL_FILE=1
    fi

    USE_ASSERT_THAT_INCORRECT=$(grep "assert_that!" $f | grep "len()" | wc -l )
    if [ ${USE_ASSERT_THAT_INCORRECT} != 0 ]
    then
        echo "Please use the modifier '\$SUT, len \$EXPECTED_LEN': ${f}"
        RETVAL=-1
        FAIL_FILE=1
    fi





    if [ ${FAIL_FILE} == 1 ]
    then
        let FAILED_FILES=${FAILED_FILES}+1
    fi

    let NUMBER_OF_FILES=${NUMBER_OF_FILES}+1
done

let SUCCESS=${NUMBER_OF_FILES}-${FAILED_FILES}
echo
echo "Report"
echo "======"
echo "  Correct assert_that usage in ${SUCCESS}/${NUMBER_OF_FILES} files."
echo "  Number of violations: ${VIOLATIONS}"
echo "  Correct usage: ${CORRECT_USAGE}"
echo

exit $RETVAL
