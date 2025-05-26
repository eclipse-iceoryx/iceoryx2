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

RET_VAL=0
IFS=$'\n'
for LINE in $(git shortlog | sed -n "s/      \(.*\)/\1/p" | grep -v "Merge pull request" | grep -v "Merge branch" | grep -v "Merge remote-tracking branch" )
do
    if [[ $(echo $LINE | grep -Ev "\[#[0-9]*\]" | wc -l) != "0" ]]
    then
        echo "Every commit message must start with [#???] where ??? corresponds to the issue number."
        echo "\"$LINE\" violates the commit message format"
        RET_VAL=1
    fi

    if [[ $(echo $LINE | sed -n "s/\[#[0-9]*\]\ \(.*\)/\1/p") == "" ]]
    then
        echo "Empty commit messages are not allowed, this commit message has no content after the issue number prefix."
        echo "\"$LINE\" violates the commit message format"
        RET_VAL=1
    fi
done

exit $RET_VAL
