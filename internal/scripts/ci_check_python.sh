#!/usr/bin/env bash
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

COLOR_RESET='\033[0m'
COLOR_GREEN='\033[1;32m'
COLOR_YELLOW='\033[1;33m'
COLOR_BLUE='\033[1;34m'
FONT_BOLD='\033[1m'
COLOR_RED='\033[1;31m'
SUCCESS_CODE=0
GIT_ROOT=$(git rev-parse --show-toplevel)

cd $GIT_ROOT

configure_python_env() {
    if [[ -z "$VIRTUAL_ENV" ]]; then
        echo -e "${COLOR_BLUE}create python dev environment${COLOR_RESET}"
        rm -rf .env
        python -m venv .env

        echo -e "${COLOR_BLUE}enter python dev environment${COLOR_RESET}"
        source .env/bin/activate

        echo -e "${COLOR_BLUE}install dependencies${COLOR_RESET}"
        pip install pytest
        pip install prospector[with_mypy]
        pip install black
        pip install isort
        pip install bandit
    else
        echo -e "${COLOR_YELLOW}use existing python dev environment${COLOR_RESET}"
    fi

    if [[ -z "$PYTHONPATH" ]]; then
        echo -e "${COLOR_BLUE}define PYTHONPATH${COLOR_RESET}"
        export PYTHONPATH=$GIT_ROOT/iceoryx2-ffi/python/python-src
    else
        echo -e "${COLOR_YELLOW}use predefined PYTHONPATH=\"${PYTHONPATH}\"${COLOR_RESET}"
    fi
}

compile() {
    echo -e "${COLOR_BLUE}compile python bindings${COLOR_RESET}"
    cd iceoryx2-ffi/python/python-src
    rm python-src/iceoryx2/*.so
    maturin develop
}

lint() {
    cd $GIT_ROOT
    echo -e "${COLOR_BLUE}[prospector] lint python bindings: examples${COLOR_RESET}"
    prospector -m -D -T -s veryhigh -F --profile .prospector.yaml examples/python/
    if [[ $? != "0" ]]; then
        echo -e "${COLOR_RED}${FONT_BOLD}lint python bindings: examples - failed${COLOR_RESET}\n"
        SUCCESS_CODE=1;
    else 
        echo -e "${COLOR_GREEN}lint python bindings: examples - success${COLOR_RESET}\n"
    fi
    echo -e "${COLOR_BLUE}[mypy] lint python bindings: examples${COLOR_RESET}"
    mypy examples/python/
    if [[ $? != "0" ]]; then
        echo -e "${COLOR_RED}${FONT_BOLD}lint python bindings: examples - failed${COLOR_RESET}\n"
        SUCCESS_CODE=1;
    else 
        echo -e "${COLOR_GREEN}lint python bindings: examples - success${COLOR_RESET}\n"
    fi


    echo -e "${COLOR_BLUE}[prospector] lint python bindings: tests${COLOR_RESET}"
    prospector -m -D -T -s veryhigh -F --profile .prospector-tests.yaml iceoryx2-ffi/python/tests/
    if [[ $? != "0" ]]; then
        echo -e "${COLOR_RED}${FONT_BOLD}lint python bindings: tests - failed${COLOR_RESET}\n"
        SUCCESS_CODE=1;
    else
        echo -e "${COLOR_GREEN}lint python bindings: tests - success${COLOR_RESET}\n"
    fi
    echo -e "${COLOR_BLUE}[mypy] lint python bindings: tests${COLOR_RESET}"
    mypy iceoryx2-ffi/python/tests/
    if [[ $? != "0" ]]; then
        echo -e "${COLOR_RED}${FONT_BOLD}lint python bindings: tests - failed${COLOR_RESET}\n"
        SUCCESS_CODE=1;
    else 
        echo -e "${COLOR_GREEN}lint python bindings: tests - success${COLOR_RESET}\n"
    fi


    echo -e "${COLOR_BLUE}[black] code formatting python bindings: examples${COLOR_RESET}"
    black --line-length=80 --check examples/python/
    if [[ $? != "0" ]]; then
        echo -e "${COLOR_RED}${FONT_BOLD}code formatting python bindings: examples - failed${COLOR_RESET}\n"
        SUCCESS_CODE=1;
    else
        echo -e "${COLOR_GREEN}code formatting python bindings: examples - success${COLOR_RESET}\n"
    fi


    echo -e "${COLOR_BLUE}[black] code formatting python bindings: tests${COLOR_RESET}"
    black --line-length=80 --check iceoryx2-ffi/python/tests/
    if [[ $? != "0" ]]; then
        echo -e "${COLOR_RED}${FONT_BOLD}code formatting python bindings: tests - failed${COLOR_RESET}\n"
        SUCCESS_CODE=1;
    else
        echo -e "${COLOR_GREEN}code formatting python bindings: tests - success${COLOR_RESET}\n"
    fi


    echo -e "${COLOR_BLUE}[isort] import ordering python bindings: examples${COLOR_RESET}"
    isort --check-only examples/python/
    if [[ $? != "0" ]]; then
        echo -e "${COLOR_RED}${FONT_BOLD}import ordering python bindings: examples - failed${COLOR_RESET}\n"
        SUCCESS_CODE=1;
    else
        echo -e "${COLOR_GREEN}import ordering python bindings: examples - success${COLOR_RESET}\n"
    fi


    echo -e "${COLOR_BLUE}[isort] import ordering python bindings: tests${COLOR_RESET}"
    isort --check-only iceoryx2-ffi/python/tests/
    if [[ $? != "0" ]]; then
        echo -e "${COLOR_RED}${FONT_BOLD}import ordering python bindings: tests - failed${COLOR_RESET}\n"
        SUCCESS_CODE=1;
    else
        echo -e "${COLOR_GREEN}import ordering python bindings: tests - success${COLOR_RESET}\n"
    fi
}

execute_tests() {
    echo -e "${COLOR_BLUE}python binding tests${COLOR_RESET}"
    pytest iceoryx2-ffi/python/tests/*
}

configure_python_env
compile
lint
execute_tests

exit $SUCCESS_CODE
