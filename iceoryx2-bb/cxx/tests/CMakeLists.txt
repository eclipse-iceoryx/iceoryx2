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

project(iceoryx2-bb-cxx-tests VERSION ${IOX2_VERSION_STRING} LANGUAGES CXX)

include(cmake/googletest.cmake)

find_package(iceoryx2-bb-cxx REQUIRED)

add_executable(iceoryx2-bb-containers-tests ${TEST_FILES})
target_sources(iceoryx2-bb-containers-tests
    PRIVATE
    ${PROJECT_SOURCE_DIR}/src/main.cpp
)

target_link_libraries(iceoryx2-bb-containers-tests iceoryx2-bb-cxx::iceoryx2-bb-containers-cxx GTest::gtest GTest::gmock)
target_compile_options(iceoryx2-bb-containers-tests PRIVATE $<$<CXX_COMPILER_ID:MSVC>:/bigobj>)
target_compile_features(iceoryx2-bb-containers-tests PRIVATE cxx_std_14)

set_target_properties(iceoryx2-bb-containers-tests PROPERTIES
    RUNTIME_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/tests"
    RUNTIME_OUTPUT_DIRECTORY_DEBUG "${CMAKE_BINARY_DIR}/tests"
    RUNTIME_OUTPUT_DIRECTORY_RELEASE "${CMAKE_BINARY_DIR}/tests"
    RUNTIME_OUTPUT_DIRECTORY_MINSIZEREL "${CMAKE_BINARY_DIR}/tests"
    RUNTIME_OUTPUT_DIRECTORY_RELWITHDEBINFO "${CMAKE_BINARY_DIR}/tests"
)

add_test(NAME "iceoryx2 C++ BB Container Tests" COMMAND iceoryx2-bb-containers-tests)
