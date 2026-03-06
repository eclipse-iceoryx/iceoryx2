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

set -e

arch="${1:-x86_64}"
packages=(
    binutils-dev
    build-essential
    libclang-dev
    clang
    cmake
    curl
    doxygen
    expect
    flex
    gcc
    g++
    git
    libc6-dev
    libpython3-all-dev
    libdwarf-dev
    libelf-dev
    libunwind-dev
    qemu-system-arm
)

echo "Detected arch:$arch"
if [[ "$arch" == "i686" ]]; then
    dpkg --add-architecture i386
    packages+=(
        gcc-multilib
        g++-multilib
        libc6-dev-i386
        libc6-dev-i386-cross
        libstdc++6-i386-cross
    )
fi

apt-get update
apt-get install -y "${packages[@]}"
