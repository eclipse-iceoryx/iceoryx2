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

project(iceoryx2-c-tests VERSION ${IOX2_VERSION} LANGUAGES CXX)

find_package(iceoryx2-c REQUIRED)

find_package(iceoryx2-cmake-modules REQUIRED)

include(Iceoryx2TestOptionsAndParams)

include(Iceoryx2PlatformSettings)

include(Iceoryx2GTest)

file(GLOB TEST_FILES src/*.cpp)

add_executable(${PROJECT_NAME} ${TEST_FILES})

target_link_libraries(
    ${PROJECT_NAME}
    iceoryx2-c::static-lib
    GTest::gtest
    GTest::gmock
    ${ICEORYX2_SANITIZER_FLAGS}
)
target_compile_options(
    ${PROJECT_NAME}
    PRIVATE
    ${ICEORYX2_TEST_CXX_FLAGS}
    ${ICEORYX2_CXX_WARNINGS}
    ${ICEORYX2_SANITIZER_FLAGS}
)
target_compile_features(${PROJECT_NAME} PRIVATE ${ICEORYX2_CXX_STD})

set_target_properties(${PROJECT_NAME} PROPERTIES
    RUNTIME_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/tests"
    RUNTIME_OUTPUT_DIRECTORY_DEBUG "${CMAKE_BINARY_DIR}/tests"
    RUNTIME_OUTPUT_DIRECTORY_RELEASE "${CMAKE_BINARY_DIR}/tests"
    RUNTIME_OUTPUT_DIRECTORY_MINSIZEREL "${CMAKE_BINARY_DIR}/tests"
    RUNTIME_OUTPUT_DIRECTORY_RELWITHDEBINFO "${CMAKE_BINARY_DIR}/tests"
)
