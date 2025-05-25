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

filename=".miri_allowlist"
while IFS= read -r line; do
if [[ "$line" == \#* ]]; then
    continue
fi

if echo "$ALL_CHANGED_FILES" | grep -q "$line"; then
    cd "$line" || { echo "Failed to change directory to $line"; exit 1; }
    echo "Run cargo miri test under: $(pwd)"
    cargo miri test
    if [ $? -ne 0 ]; then
        echo "Error: cargo miri test failed."
        exit 1
    fi
    cd -
else
    echo "skip $line because the PR doesn't touch its files"
fi
done < "$filename"
