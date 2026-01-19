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

cmake_minimum_required(VERSION 3.22)
project(component-tests_cxx LANGUAGES CXX)

find_package(iceoryx2-cmake-modules REQUIRED)

include(Iceoryx2PlatformSettings)

set(CMAKE_CXX_STANDARD ${ICEORYX2_CXX_STD_VALUE})
set(CMAKE_CXX_STANDARD_REQUIRED ON)

find_package(iceoryx2-cxx 0.8.999 REQUIRED)

add_executable(component-tests_cxx)
target_compile_options(component-tests_cxx
    PRIVATE
    ${ICEORYX2_CXX_FLAGS}
    ${ICEORYX2_CXX_WARNINGS}
    ${ICEORYX2_SANITIZER_FLAGS}
    ${ICEORYX2_COVERAGE_FLAGS}
)
target_sources(component-tests_cxx
    PRIVATE
    ${PROJECT_SOURCE_DIR}/src/main.cpp
    ${PROJECT_SOURCE_DIR}/src/test_containers.cpp
    ${PROJECT_SOURCE_DIR}/src/test_container_mutation.cpp
    PRIVATE
    ${PROJECT_SOURCE_DIR}/include/iox2/component-tests/common.hpp
)
target_include_directories(component-tests_cxx
    PRIVATE
    ${PROJECT_SOURCE_DIR}/include
)
target_link_libraries(component-tests_cxx
    PRIVATE
    iceoryx2-cxx::static-lib-cxx
    iceoryx2-bb-cxx::iceoryx2-bb-cxx
    ${ICEORYX2_SANITIZER_FLAGS}
    ${ICEORYX2_COVERAGE_FLAGS}
)
