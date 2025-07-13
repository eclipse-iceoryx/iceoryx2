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

RUST_FILE="iceoryx2-cli/lib/src/config_descriptions.rs"   # <-- update this path if needed
README_FILE="config/README.md"

missing=0

echo "🔍 Checking if each config key from Rust exists in README.md ..."

while IFS= read -r key; do
    if grep -qF "$key" "$README_FILE"; then
        echo "✅ Found: $key"
    else
        echo "❌ Missing: $key"
        ((missing++))
    fi
done < <(grep -oP 'key:\s*"\K[^"]+' "$RUST_FILE")

echo
if [ "$missing" -eq 0 ]; then
    echo "🎉 All config keys from Rust are documented in README.md!"
else
    echo "⚠️  $missing config keys are missing from README.md. See above."
fi
