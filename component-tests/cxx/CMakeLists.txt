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

string(JOIN " " ICEORYX2_C_FLAGS_STRING ${ICEORYX2_C_FLAGS})
string(JOIN " " ICEORYX2_C_WARNINGS_STRING ${ICEORYX2_C_WARNINGS})
string(JOIN " " ICEORYX2_CXX_FLAGS_STRING ${ICEORYX2_CXX_FLAGS})
string(JOIN " " ICEORYX2_CXX_WARNINGS_STRING ${ICEORYX2_CXX_WARNINGS})
string(JOIN " " ICEORYX2_SANITIZER_FLAGS_STRING ${ICEORYX2_SANITIZER_FLAGS})

set(CMAKE_C_FLAGS "\
    ${CMAKE_C_FLAGS} \
    ${ICEORYX2_C_FLAGS_STRING} \
    ${ICEORYX2_C_WARNINGS_STRING}\
")
set(CMAKE_CXX_FLAGS "\
    ${CMAKE_CXX_FLAGS} \
    ${ICEORYX2_CXX_FLAGS_STRING} \
    ${ICEORYX2_CXX_WARNINGS_STRING} \
    ${ICEORYX2_SANITIZER_FLAGS_STRING} \
")

find_package(iceoryx2-cxx 0.7.0 REQUIRED)

add_executable(component-tests_cxx)
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
    iceoryx2-bb-cxx::iceoryx2-bb-containers-cxx
)
