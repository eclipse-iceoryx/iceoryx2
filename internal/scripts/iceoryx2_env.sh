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

CONTAINER_NAME_PREFIX="iceoryx2_env_"
CONTAINER_MEMORY_SIZE="8g"
CONTAINER_SHM_MEMORY_SIZE="4g"
DEFAULT_OS_VERSION="ubuntu:22.04"
ICEORYX2_PATH=$(git rev-parse --show-toplevel)

COLOR_RESET='\033[0m'
COLOR_GREEN='\033[1;32m'
COLOR_CYAN='\033[1;34m'
FONT_BOLD='\033[1m'
COLOR_RED='\033[1;31m'

setup_docker_image() {
    echo "Europe/Berlin" > /etc/timezone
    ln -sf /usr/share/zoneinfo/Europe/Berlin /etc/localtime

    # ubuntu/debian and derivatives
    if command -v apt &>/dev/null; then
        apt update
        apt -y install sudo git fish curl expect vim lsb-release software-properties-common gcc libacl1-dev libclang-dev zlib1g-dev clang libpython3-all-dev
    elif command -v pacman &>/dev/null; then
        pacman -Syu --noconfirm fish curl expect git vim clang python
    else
        echo Please install the following packages to have a working iceoryx2 environment:
        echo fish curl clang python
    fi

    useradd testuser1
    useradd testuser2
    groupadd testgroup1
    groupadd testgroup2

    git config --global --add safe.directory /iceoryx2
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    export PATH=$PATH:/root/.cargo/bin
    rustup toolchain install stable
    rustup toolchain install nightly
    cargo install cargo-nextest

    mkdir -p /root/.config/fish
    echo "set -gx PATH /root/.cargo/bin \$PATH" >> /root/.config/fish/config.fish
    exit
}

start_docker_session() {
    bash
    exit
}

help() {
    echo
    echo -e "${FONT_BOLD}iceoryx2 development environment${COLOR_RESET}"
    echo
    echo -e "  $0 ${COLOR_CYAN}[ACTION] ${COLOR_RESET}(optional)${COLOR_CYAN}[DOCKER_OS]"
    echo
    echo -e "${COLOR_CYAN}ACTION:${COLOR_RESET}"
    echo -e "  ${FONT_BOLD}start${COLOR_RESET}          - start a specific docker container"
    echo -e "  ${FONT_BOLD}stop${COLOR_RESET}           - stop a specific docker container"
    echo -e "  ${FONT_BOLD}stop_all${COLOR_RESET}       - stops all docker containers running an iceoryx2 environment"
    echo -e "  ${FONT_BOLD}enter${COLOR_RESET}          - enters (and starts if not running) the docker container"
    echo -e "  ${FONT_BOLD}remove${COLOR_RESET}         - removes a specific docker container"
    echo -e "  ${FONT_BOLD}remove_all${COLOR_RESET}     - removes all docker containers running an iceoryx2 environment"
    echo -e "  ${FONT_BOLD}list${COLOR_RESET}           - list all docker containers running an iceoryx2 environment"
    echo -e "  ${FONT_BOLD}list_running${COLOR_RESET}   - list all running docker containers running an iceoryx2 environment"
    echo
    echo -e "${COLOR_CYAN}DOCKER_OS:${COLOR_RESET}"
    echo "  Defines the operating system of the docker container."
    echo "  Some standard options:"
    echo "    archlinux"
    echo "    ubuntu:22.04"
    echo "    ros:rolling"
    echo
    echo -e "${COLOR_CYAN}Example:${COLOR_RESET}"
    echo "  $0 start archlinux     # starts an iceoryx2 docker container based on archlinux"
    echo "  $0 enter ubuntu:22.04  # enters (and starts if not running) an iceoryx2 docker container based ubuntu"
    echo
    exit
}

create_docker() {
    echo -e "  ${COLOR_CYAN}create docker container${COLOR_RESET} [${FONT_BOLD}$CONTAINER_NAME${COLOR_RESET}]"
    docker run --name $CONTAINER_NAME \
               --mount type=bind,source=${ICEORYX2_PATH},target=/iceoryx2 \
               --hostname ${OS_VERSION} \
               -dt --memory $CONTAINER_MEMORY_SIZE \
               --shm-size $CONTAINER_SHM_MEMORY_SIZE ${OS_VERSION}
    echo -e "  ${COLOR_CYAN}setting up iceoryx2 development environment${COLOR_RESET} [${FONT_BOLD}$CONTAINER_NAME${COLOR_RESET}]"

    docker exec -it $CONTAINER_NAME /iceoryx2/$(realpath $0 --relative-to=$ICEORYX2_PATH) setup $OS_VERSION
}

startup_docker() {
    echo -en "         start iceoryx2 development environment [${FONT_BOLD}$CONTAINER_NAME${COLOR_RESET}]"
    docker start $CONTAINER_NAME > /dev/null
    echo -e "\r  [${COLOR_GREEN}done${COLOR_RESET}]"
}

list_docker() {
    docker container ls -a | sed -n "s/.*\(iceoryx2_env_.*\)/  \1/p"
}

list_running_docker() {
    docker container ls | sed -n "s/.*\(iceoryx2_env_.*\)/  \1/p"
}

start_docker() {
    if [[ $(docker container inspect -f '{{.State.Running}}' $CONTAINER_NAME 2> /dev/null) == "true" ]]; then
        return
    fi

    if [[ $(list_docker | grep ${CONTAINER_NAME} | wc -l) == "0" ]]; then
        create_docker
    else
        startup_docker
    fi

    echo
    echo -e "  ${COLOR_CYAN}iceoryx2 development environment${COLOR_RESET}"
    echo -e "  #################################################"
    echo
    echo -e "    container name..........: ${FONT_BOLD}${CONTAINER_NAME}${COLOR_RESET}"
    echo -e "    OS-Version..............: ${FONT_BOLD}${OS_VERSION}${COLOR_RESET}"
    echo -e "    memory..................: ${FONT_BOLD}${CONTAINER_MEMORY_SIZE}${COLOR_RESET}"
    echo -e "    shared memory...........: ${FONT_BOLD}${CONTAINER_SHM_MEMORY_SIZE}${COLOR_RESET}"
    echo -e "    iceoryx2-path............: ${FONT_BOLD}${ICEORYX2_PATH}${COLOR_RESET}"
    echo
}

stop_docker() {
    if [[ $(docker container inspect -f '{{.State.Running}}' $CONTAINER_NAME) == "true" ]]; then
        echo -en "         stopping iceoryx2 development environment [${FONT_BOLD}${CONTAINER_NAME}${COLOR_RESET}] container"
        docker container stop $CONTAINER_NAME > /dev/null
        echo -e "\r  [${COLOR_GREEN}done${COLOR_RESET}]"
    fi
}

stop_all_docker() {
    echo -e "${COLOR_CYAN}stopping all iceoryx2 development environment docker containers${COLOR_RESET}"
    for DOCKER in $(list_running_docker); do
        CONTAINER_NAME=$DOCKER
        stop_docker
    done
}

drop_docker() {
    stop_docker
    echo -en "         removing iceoryx2 development environment [${FONT_BOLD}${CONTAINER_NAME}${COLOR_RESET}] container"
    docker rm $CONTAINER_NAME > /dev/null
    echo -e "\r  [${COLOR_GREEN}done${COLOR_RESET}]"
}

drop_all_docker() {
    echo -e "${COLOR_RED}removing all iceoryx2 environment docker containers${COLOR_RESET}"
    for DOCKER in $(list_docker); do
        CONTAINER_NAME=$DOCKER
        drop_docker
    done
}

enter_docker() {
    start_docker

    docker exec -it $CONTAINER_NAME fish -c "
    echo
    eval 'echo \"  rustup version...........: \"\\033\[1\;37m(rustup --version | head -1 )\\033\[0m'
    eval 'echo \"  rust version.............: \"\\033\[1\;37m(rustc --version )\\033\[0m'
    echo
    cd /iceoryx2
    fish"

    # we use eval here since we would like to evaluate the expression inside of the docker
    # container and not right away in this script
    if [[ $? -ne 0 ]]; then
        docker exec -it $CONTAINER_NAME bash -c "
        echo
        eval 'echo \"  rustup version...........: \"\\033\[1\;37m(rustup --version | head -1 )\\033\[0m'
        eval 'echo \"  rust version.............: \"\\033\[1\;37m(rustc --version )\\033\[0m'
        echo
        cd /iceoryx2
        bash
        "
    fi
}

ACTION=$1
OS_VERSION=$2

if [[ -z $OS_VERSION ]]; then
    OS_VERSION=$DEFAULT_OS_VERSION
fi

CONTAINER_NAME=${CONTAINER_NAME_PREFIX}$(echo ${OS_VERSION} | tr : . | tr \/ .)

if [[ $ACTION == "start" ]]; then
    start_docker
elif [[ $ACTION == "stop" ]]; then
    stop_docker
elif [[ $ACTION == "stop_all" ]]; then
    stop_all_docker
elif [[ $ACTION == "remove" ]]; then
    drop_docker
elif [[ $ACTION == "remove_all" ]]; then
    drop_all_docker
elif [[ $ACTION == "enter" ]]; then
    enter_docker
elif [[ $ACTION == "setup" ]]; then
    setup_docker_image
elif [[ $ACTION == "list" ]]; then
    list_docker
elif [[ $ACTION == "list_running" ]]; then
    list_running_docker
else
    help
fi
