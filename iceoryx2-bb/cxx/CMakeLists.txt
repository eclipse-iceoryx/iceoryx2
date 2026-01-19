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

set(IOX2_VERSION 0.8.999)
project(iceoryx2-bb-cxx VERSION ${IOX2_VERSION} LANGUAGES CXX)

set(PREFIX iceoryx2/v${CMAKE_PROJECT_VERSION})
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

find_package(iceoryx2-cmake-modules REQUIRED)

include(Iceoryx2CommonOptionsAndParams)
include(cmake/options.cmake)
include(cmake/GdbPrettyPrinter.cmake)

include(Iceoryx2PlatformSettings)

if(IOX2_BB_CXX_CONFIG_USE_STD_EXPECTED AND IOX2_BB_CXX_CONFIG_USE_CUSTOM_VOCABULARY_TYPES)
    message(FATAL_ERROR "Expected cannot be overriden to use both custom and std variants at the same time.")
endif()
if(IOX2_BB_CXX_CONFIG_USE_STD_OPTIONAL AND IOX2_BB_CXX_CONFIG_USE_CUSTOM_VOCABULARY_TYPES)
    message(FATAL_ERROR "Optional cannot be overriden to use both custom and std variants at the same time.")
endif()

if(IOX2_BB_CXX_CONFIG_USE_CUSTOM_VOCABULARY_TYPES)
    find_package(${IOX2_BB_CXX_CONFIG_CUSTOM_VOCABULARY_TYPES_CMAKE_TARGET})
endif()

add_library(iceoryx2-bb-cxx INTERFACE)
add_library(iceoryx2-bb-cxx::iceoryx2-bb-cxx ALIAS iceoryx2-bb-cxx)

target_sources(iceoryx2-bb-cxx INTERFACE
    $<BUILD_INTERFACE:${PROJECT_SOURCE_DIR}/include/iox2/bb/detail/assertions.hpp>
    $<BUILD_INTERFACE:${PROJECT_SOURCE_DIR}/include/iox2/bb/detail/builder.hpp>
    $<BUILD_INTERFACE:${PROJECT_SOURCE_DIR}/include/iox2/bb/detail/path_and_file_verifier.hpp>
    $<BUILD_INTERFACE:${PROJECT_SOURCE_DIR}/include/iox2/bb/detail/raw_byte_storage.hpp>
    $<BUILD_INTERFACE:${PROJECT_SOURCE_DIR}/include/iox2/bb/detail/static_function.hpp>
    $<BUILD_INTERFACE:${PROJECT_SOURCE_DIR}/include/iox2/bb/detail/string_internal.hpp>
    $<BUILD_INTERFACE:${PROJECT_SOURCE_DIR}/include/iox2/bb/expected.hpp>
    $<BUILD_INTERFACE:${PROJECT_SOURCE_DIR}/include/iox2/bb/duration.hpp>
    $<BUILD_INTERFACE:${PROJECT_SOURCE_DIR}/include/iox2/bb/file_name.hpp>
    $<BUILD_INTERFACE:${PROJECT_SOURCE_DIR}/include/iox2/bb/file_path.hpp>
    $<BUILD_INTERFACE:${PROJECT_SOURCE_DIR}/include/iox2/bb/into.hpp>
    $<BUILD_INTERFACE:${PROJECT_SOURCE_DIR}/include/iox2/bb/layout.hpp>
    $<BUILD_INTERFACE:${PROJECT_SOURCE_DIR}/include/iox2/bb/optional.hpp>
    $<BUILD_INTERFACE:${PROJECT_SOURCE_DIR}/include/iox2/bb/path.hpp>
    $<BUILD_INTERFACE:${PROJECT_SOURCE_DIR}/include/iox2/bb/semantic_string.hpp>
    $<BUILD_INTERFACE:${PROJECT_SOURCE_DIR}/include/iox2/bb/slice.hpp>
    $<BUILD_INTERFACE:${PROJECT_SOURCE_DIR}/include/iox2/bb/static_function.hpp>
    $<BUILD_INTERFACE:${PROJECT_SOURCE_DIR}/include/iox2/bb/static_string.hpp>
    $<BUILD_INTERFACE:${PROJECT_SOURCE_DIR}/include/iox2/bb/static_vector.hpp>
    $<BUILD_INTERFACE:${PROJECT_SOURCE_DIR}/include/iox2/bb/std_chrono_support.hpp>
    $<BUILD_INTERFACE:${PROJECT_SOURCE_DIR}/include/iox2/bb/stl/expected.hpp>
    $<BUILD_INTERFACE:${PROJECT_SOURCE_DIR}/include/iox2/bb/stl/optional.hpp>
    $<BUILD_INTERFACE:$<$<CXX_COMPILER_ID:MSVC>:${PROJECT_SOURCE_DIR}/iceoryx2-bb-cxx.natvis>>
)

target_include_directories(iceoryx2-bb-cxx
    INTERFACE
    $<BUILD_INTERFACE:${PROJECT_SOURCE_DIR}/include>
    $<INSTALL_INTERFACE:include/${PREFIX}>
)

if(IOX2_BB_CXX_CONFIG_USE_CUSTOM_VOCABULARY_TYPES)
    target_link_libraries(iceoryx2-bb-cxx
        INTERFACE
        ${IOX2_BB_CXX_CONFIG_CUSTOM_VOCABULARY_TYPES_CMAKE_TARGET}
    )
    set(IOX2_VOCABULARY_TYPES_DEPENDENCY_CONFIG_CMAKE_IN "find_dependency(${IOX2_BB_CXX_CONFIG_CUSTOM_VOCABULARY_TYPES_CMAKE_TARGET})")
else()
    if(IOX2_BB_CXX_CONFIG_USE_STD_EXPECTED)
        set(IOX2_VARIATION_EXPECTED_INCLUDE "expected-std-include")
    else()
        set(IOX2_VARIATION_EXPECTED_INCLUDE "expected-iox2-include")
    endif()

    if(IOX2_BB_CXX_CONFIG_USE_STD_OPTIONAL)
        set(IOX2_VARIATION_OPTIONAL_INCLUDE "optional-std-include")
    else()
        set(IOX2_VARIATION_OPTIONAL_INCLUDE "optional-iox2-include")
    endif()

    target_sources(iceoryx2-bb-cxx
        INTERFACE
        $<BUILD_INTERFACE:${PROJECT_SOURCE_DIR}/variation/${IOX2_VARIATION_EXPECTED_INCLUDE}/iox2/bb/variation/expected_adaption.hpp>
        $<BUILD_INTERFACE:${PROJECT_SOURCE_DIR}/variation/${IOX2_VARIATION_OPTIONAL_INCLUDE}/iox2/bb/variation/optional_adaption.hpp>
    )

    target_include_directories(iceoryx2-bb-cxx
        INTERFACE
        $<BUILD_INTERFACE:${PROJECT_SOURCE_DIR}/variation/${IOX2_VARIATION_EXPECTED_INCLUDE}>
        $<BUILD_INTERFACE:${PROJECT_SOURCE_DIR}/variation/${IOX2_VARIATION_OPTIONAL_INCLUDE}>
    )

    if(IOX2_BB_CXX_CONFIG_USE_STD_EXPECTED AND ${ICEORYX2_CXX_STD_VALUE} VERSION_LESS 23)
        set(ICEORYX2_CXX_STD cxx_std_23)
        message(STATUS "[i]  The std::expected requires at least C++23! Setting cxx_std_23 for iceoryx2-bb-cxx!")
    elseif(IOX2_BB_CXX_CONFIG_USE_STD_OPTIONAL AND ${ICEORYX2_CXX_STD_VALUE} LESS 17)
        set(ICEORYX2_CXX_STD cxx_std_17)
        message(STATUS "[i]  The std::optional requires at least C++17! Setting cxx_std_17 for iceoryx2-bb-cxx!")
    endif()
endif()

target_compile_features(iceoryx2-bb-cxx INTERFACE ${ICEORYX2_CXX_STD})

# NOTE: position independent code is required due to the usage of thread-local-storage
set_target_properties(iceoryx2-bb-cxx PROPERTIES POSITION_INDEPENDENT_CODE ON)

target_gdb_pretty_printer(iceoryx2-bb-cxx
    INTERFACE
    ${PROJECT_SOURCE_DIR}/iceoryx2-bb-cxx-pretty-printer.py
)

# include install setup
# find_package from source tree
set(${PROJECT_NAME}_DIR ${PROJECT_SOURCE_DIR}/cmake
    CACHE FILEPATH
    "${PROJECT_NAME}Config.cmake to make find_package(${PROJECT_NAME}) work in source tree!"
    FORCE
)

# include install setup

include(${CMAKE_CURRENT_LIST_DIR}/cmake/install.cmake)

# add tests

if(${BUILD_TESTING})
    enable_testing()
    add_subdirectory(tests)
endif()
