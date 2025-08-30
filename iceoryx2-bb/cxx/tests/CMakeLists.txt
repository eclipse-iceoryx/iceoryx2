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

project(iceoryx2-bb-cxx-tests VERSION ${IOX2_VERSION} LANGUAGES CXX)

find_package(iceoryx2-bb-cxx REQUIRED)

find_package(iceoryx2-cmake-modules REQUIRED)

include(Iceoryx2TestOptionsAndParams)

include(Iceoryx2PlatformSettings)

include(Iceoryx2GTest)

add_executable(iceoryx2-bb-containers-tests ${TEST_FILES})
target_sources(iceoryx2-bb-containers-tests
    PRIVATE
    ${PROJECT_SOURCE_DIR}/src/testing/observable.hpp
    ${PROJECT_SOURCE_DIR}/src/testing/test_utils.hpp
    PRIVATE
    ${PROJECT_SOURCE_DIR}/src/testing/observable.cpp
    ${PROJECT_SOURCE_DIR}/src/testing/test_utils.cpp
    PRIVATE
    ${PROJECT_SOURCE_DIR}/src/main.cpp
    ${PROJECT_SOURCE_DIR}/src/optional_tests.cpp
)
target_include_directories(iceoryx2-bb-containers-tests PRIVATE ${PROJECT_SOURCE_DIR}/src)
target_link_libraries(
    iceoryx2-bb-containers-tests
    PRIVATE
    iceoryx2-bb-cxx::iceoryx2-bb-containers-cxx
    GTest::gtest GTest::gmock
    ${ICEORYX2_SANITIZER_FLAGS}
)
target_compile_options(
    iceoryx2-bb-containers-tests
    PRIVATE
    ${ICEORYX2_TEST_CXX_FLAGS}
    ${ICEORYX2_CXX_WARNINGS}
    ${ICEORYX2_SANITIZER_FLAGS}
)
target_compile_features(iceoryx2-bb-containers-tests PRIVATE ${ICEORYX2_CXX_STD})

set_target_properties(iceoryx2-bb-containers-tests PROPERTIES
    RUNTIME_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/tests"
    RUNTIME_OUTPUT_DIRECTORY_DEBUG "${CMAKE_BINARY_DIR}/tests"
    RUNTIME_OUTPUT_DIRECTORY_RELEASE "${CMAKE_BINARY_DIR}/tests"
    RUNTIME_OUTPUT_DIRECTORY_MINSIZEREL "${CMAKE_BINARY_DIR}/tests"
    RUNTIME_OUTPUT_DIRECTORY_RELWITHDEBINFO "${CMAKE_BINARY_DIR}/tests"
)

add_test(NAME "iceoryx2 C++ BB Container Tests" COMMAND iceoryx2-bb-containers-tests)
