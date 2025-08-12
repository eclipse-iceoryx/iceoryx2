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

cmake_minimum_required(VERSION 3.22)

project(iceoryx2-bb VERSION ${IOX2_VERSION_STRING} LANGUAGES CXX)

set(PREFIX iceoryx2/v${CMAKE_PROJECT_VERSION})

# include only lib -> includes are installed only once despite being used in static lib as well as shared lib

add_library(iceoryx2-bb-containers INTERFACE)
add_library(iceoryx2-cxx::iceoryx2-bb-containers ALIAS iceoryx2-bb-containers)

target_include_directories(iceoryx2-bb-containers
    INTERFACE
    $<BUILD_INTERFACE:${PROJECT_SOURCE_DIR}/include>
    $<INSTALL_INTERFACE:include/${PREFIX}>
)

target_compile_features(iceoryx2-bb-containers INTERFACE cxx_std_14)

# include install setup
# @todo

# add tests

if(${BUILD_TESTING})
    enable_testing()
    add_subdirectory(tests)
endif()
