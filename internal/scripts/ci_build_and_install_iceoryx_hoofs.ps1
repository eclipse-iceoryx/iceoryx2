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

param(
    [Parameter()]
    [String]$mode = "release",
    [Parameter()]
    [String]$toolchain = "stable"
)

$ErrorActionPreference = "Stop"

$NUM_JOBS = (Get-WmiObject Win32_processor).NumberOfLogicalProcessors

git clone --depth 1 --branch v2.95.7 https://github.com/eclipse-iceoryx/iceoryx.git target/ff/iceoryx/src

switch ($mode) {
    "release" {
        $CMAKE_BUILD_TYPE="-DCMAKE_BUILD_TYPE=Release"
        $CMAKE_BUILD_CONFIG="--config Release"
    }
    "debug" {
        $CMAKE_BUILD_TYPE="-DCMAKE_BUILD_TYPE=Debug"
        $CMAKE_BUILD_CONFIG="--config Debug"
    }
}

switch ($toolchain) {
    "stable-gnu" {
        if ($?) { Write-Host "## Using the MinGW toolchain" }
        if ($?) { cmake -S target/ff/iceoryx/src/iceoryx_platform -B target/ff/iceoryx/build/platform -DBUILD_SHARED_LIBS=OFF $CMAKE_BUILD_TYPE -DCMAKE_INSTALL_PREFIX=target/ff/iceoryx/install -G "MinGW Makefiles" }
    }
    default {
        if ($?) { Write-Host "## Using the MSVC toolchain" }
        if ($?) { cmake -S target/ff/iceoryx/src/iceoryx_platform -B target/ff/iceoryx/build/platform -DBUILD_SHARED_LIBS=OFF $CMAKE_BUILD_TYPE -DCMAKE_INSTALL_PREFIX=target/ff/iceoryx/install -DCMAKE_CXX_FLAGS="/MP" }
    }
}

if ($?) { Write-Host "## Building and installing iceoryx_platform with $NUM_JOBS cores" }
if ($?) { cmake --build target/ff/iceoryx/build/platform $CMAKE_BUILD_CONFIG -j $NUM_JOBS }
if ($?) { cmake --install target/ff/iceoryx/build/platform $CMAKE_BUILD_CONFIG }

switch ($toolchain) {
    "stable-gnu" {
        if ($?) { cmake -S target/ff/iceoryx/src/iceoryx_hoofs -B target/ff/iceoryx/build/hoofs -DBUILD_SHARED_LIBS=OFF $CMAKE_BUILD_TYPE -DCMAKE_INSTALL_PREFIX=target/ff/iceoryx/install -DCMAKE_PREFIX_PATH="$pwd/target/ff/iceoryx/install" -G "MinGW Makefiles" -DIOX_USE_HOOFS_SUBSET_ONLY=ON }
    }
    default {
        if ($?) { cmake -S target/ff/iceoryx/src/iceoryx_hoofs -B target/ff/iceoryx/build/hoofs -DBUILD_SHARED_LIBS=OFF $CMAKE_BUILD_TYPE -DCMAKE_INSTALL_PREFIX=target/ff/iceoryx/install -DCMAKE_PREFIX_PATH="$pwd/target/ff/iceoryx/install" -DCMAKE_CXX_FLAGS="/MP" -DIOX_USE_HOOFS_SUBSET_ONLY=ON }
    }
}

if ($?) { Write-Host "## Building and installing iceoryx_hoofs with $NUM_JOBS cores" }
if ($?) { cmake --build target/ff/iceoryx/build/hoofs $CMAKE_BUILD_CONFIG -j $NUM_JOBS }
if ($?) { cmake --install target/ff/iceoryx/build/hoofs $CMAKE_BUILD_CONFIG }
