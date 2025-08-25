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

set(IOX2_VERSION 0.6.1)
project(iceoryx2-bb-cxx VERSION ${IOX2_VERSION} LANGUAGES CXX)

set(PREFIX iceoryx2/v${CMAKE_PROJECT_VERSION})
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

find_package(iceoryx2-cmake-modules REQUIRED)

include(Iceoryx2CommonOptionsAndParams)
include(cmake/options.cmake)

include(Iceoryx2PlatformSettings)

add_library(iceoryx2-bb-containers-cxx INTERFACE)
add_library(iceoryx2-bb-cxx::iceoryx2-bb-containers-cxx ALIAS iceoryx2-bb-containers-cxx)

target_sources(iceoryx2-bb-containers-cxx INTERFACE
    ${PROJECT_SOURCE_DIR}/include/iox2/container/config.hpp
    ${PROJECT_SOURCE_DIR}/include/iox2/container/optional.hpp
    $<$<CXX_COMPILER_ID:MSVC>:${PROJECT_SOURCE_DIR}/iceoryx2-bb-containers-cxx.natvis>
)

target_include_directories(iceoryx2-bb-containers-cxx
    INTERFACE
    $<BUILD_INTERFACE:${PROJECT_SOURCE_DIR}/include>
    $<INSTALL_INTERFACE:include/${PREFIX}>
)

target_compile_features(iceoryx2-bb-containers-cxx INTERFACE cxx_std_14)
# TODO check if this settings will be propagated to the user; maybe this should only be used for the tests
target_compile_options(iceoryx2-bb-containers-cxx INTERFACE ${ICEORYX2_CXX_FLAGS} ${ICEORYX2_CXX_WARNINGS})

# include install setup
# find_package from source tree
set(${PROJECT_NAME}_DIR ${PROJECT_SOURCE_DIR}/cmake
    CACHE FILEPATH
    "${PROJECT_NAME}Config.cmake to make find_package(${PROJECT_NAME}) work in source tree!"
    FORCE
)
# @todo - install, export and packaging

# add tests

if(${BUILD_TESTING})
    enable_testing()
    add_subdirectory(tests)
endif()
