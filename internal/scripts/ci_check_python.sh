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


#!/bin/bash
COLOR_RESET='\033[0m'
COLOR_GREEN='\033[1;32m'
COLOR_CYAN='\033[1;34m'
FONT_BOLD='\033[1m'
COLOR_RED='\033[1;31m'
SUCCESS_CODE=0
GIT_ROOT=$(git rev-parse --show-toplevel)

cd $GIT_ROOT

echo -e "${COLOR_CYAN}create python dev environment${COLOR_RESET}"
rm -rf .env
python -m venv .env

echo -e "${COLOR_CYAN}enter python dev environment${COLOR_RESET}"
source .env/bin/activate

echo -e "${COLOR_CYAN}install dependencies${COLOR_RESET}"
pip install pytest
pip install prospector[with_mypy]
pip install black
pip install isort

echo -e "${COLOR_CYAN}compile python bindings${COLOR_RESET}"
cd iceoryx2-ffi/python
maturin develop

cd $GIT_ROOT
echo -e "${COLOR_CYAN}lint python bindings: examples${COLOR_RESET}"
prospector -m -D -T -s veryhigh -F examples/python/
if [[ $? != "0" ]]; then
    echo -e "${COLOR_RED}${FONT_BOLD}lint python bindings: examples - failed${COLOR_RESET}"
    SUCCESS_CODE=1;
else 
    echo -e "${COLOR_GREEN}lint python bindings: examples - success${COLOR_RESET}"
fi
mypy examples/python/
if [[ $? != "0" ]]; then
    echo -e "${COLOR_RED}${FONT_BOLD}lint python bindings: examples - failed${COLOR_RESET}"
    SUCCESS_CODE=1;
else 
    echo -e "${COLOR_GREEN}lint python bindings: examples - success${COLOR_RESET}"
fi


echo -e "${COLOR_CYAN}lint python bindings: tests${COLOR_RESET}"
prospector -m -D -T -s veryhigh -F iceoryx2-ffi/python/tests/
if [[ $? != "0" ]]; then
    echo -e "${COLOR_RED}${FONT_BOLD}lint python bindings: tests - failed${COLOR_RESET}"
    SUCCESS_CODE=1;
else
    echo -e "${COLOR_GREEN}lint python bindings: tests - success${COLOR_RESET}"
fi
mypy iceoryx2-ffi/python/tests/
if [[ $? != "0" ]]; then
    echo -e "${COLOR_RED}${FONT_BOLD}lint python bindings: tests - failed${COLOR_RESET}"
    SUCCESS_CODE=1;
else 
    echo -e "${COLOR_GREEN}lint python bindings: tests - success${COLOR_RESET}"
fi


echo -e "${COLOR_CYAN}code formatting python bindings: examples${COLOR_RESET}"
black --check examples/python/
if [[ $? != "0" ]]; then
    echo -e "${COLOR_RED}${FONT_BOLD}code formatting python bindings: examples - failed${COLOR_RESET}"
    SUCCESS_CODE=1;
else
    echo -e "${COLOR_GREEN}code formatting python bindings: examples - success${COLOR_RESET}"
fi


echo -e "${COLOR_CYAN}code formatting python bindings: tests${COLOR_RESET}"
black --check iceoryx2-ffi/python/tests/
if [[ $? != "0" ]]; then
    echo -e "${COLOR_RED}${FONT_BOLD}code formatting python bindings: tests - failed${COLOR_RESET}"
    SUCCESS_CODE=1;
else
    echo -e "${COLOR_GREEN}code formatting python bindings: tests - success${COLOR_RESET}"
fi


echo -e "${COLOR_CYAN}import ordering python bindings: examples${COLOR_RESET}"
isort --check-only examples/python/
if [[ $? != "0" ]]; then
    echo -e "${COLOR_RED}${FONT_BOLD}import ordering python bindings: examples - failed${COLOR_RESET}"
    SUCCESS_CODE=1;
else
    echo -e "${COLOR_GREEN}import ordering python bindings: examples - success${COLOR_RESET}"
fi


echo -e "${COLOR_CYAN}import ordering python bindings: tests${COLOR_RESET}"
isort --check-only iceoryx2-ffi/python/tests/
if [[ $? != "0" ]]; then
    echo -e "${COLOR_RED}${FONT_BOLD}import ordering python bindings: tests - failed${COLOR_RESET}"
    SUCCESS_CODE=1;
else
    echo -e "${COLOR_GREEN}import ordering python bindings: tests - success${COLOR_RESET}"
fi

echo -e "${COLOR_CYAN}python binding tests${COLOR_RESET}"
pytest iceoryx2-ffi/python/tests/*

exit $SUCCESS_CODE
