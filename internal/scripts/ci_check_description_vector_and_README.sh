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

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
RESET='\033[0m'

echo -e "${CYAN}${BOLD}=== Checking if each config key from Rust exists in README.md ===${RESET}"

while IFS= read -r key; do
    if grep -qF "$key" "$README_FILE"; then
        echo -e "${GREEN}[ OK ]${RESET} Found: $key"
    else
        echo -e "${RED}[FAIL]${RESET} Missing: $key"
        ((missing++))
    fi
done < <(grep -oP 'key:\s*"\K[^"]+' "$RUST_FILE")

echo
if [ "$missing" -eq 0 ]; then
    echo -e "${GREEN}==============================================="
    echo -e "[SUCCESS] All config keys are documented!"
    echo -e "===============================================${RESET}"
else
    echo -e "${YELLOW}==============================================="
    echo -e "[WARNING] $missing config key(s) missing in README.md"
    echo -e "See above for details."
    echo -e "===============================================${RESET}"
    exit "$missing"
fi
