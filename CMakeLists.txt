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
set(IOX2_VERSION_STRING "0.6.0")
project(iceoryx2 VERSION ${IOX2_VERSION_STRING})

#TODO how to handle feature flags

set(CMAKE_EXPORT_COMPILE_COMMANDS ON) # "Create compile_commands.json file"

macro(add_option)
    set(ONE_VALUE_ARGS NAME DESCRIPTION DEFAULT_VALUE)
    cmake_parse_arguments(ADD_OPTION "" "${ONE_VALUE_ARGS}" "" ${ARGN})

    option(${ADD_OPTION_NAME} ${ADD_OPTION_DESCRIPTION} ${ADD_OPTION_DEFAULT_VALUE})
    message(STATUS "  ${ADD_OPTION_NAME}: ${${ADD_OPTION_NAME}} (Description: ${ADD_OPTION_DESCRIPTION})")
endmacro()

macro(add_param)
    set(ONE_VALUE_ARGS NAME DESCRIPTION DEFAULT_VALUE)
    cmake_parse_arguments(ADD_PARAM "" "${ONE_VALUE_ARGS}" "" ${ARGN})

    if(NOT ${ADD_PARAM_NAME})
        set(${ADD_PARAM_NAME} ${ADD_PARAM_DEFAULT_VALUE})
    endif()
    message(STATUS "  ${ADD_PARAM_NAME}: ${${ADD_PARAM_NAME}} (Description: ${ADD_PARAM_DESCRIPTION})")
endmacro()

set(IOX2_RUST_FEATURES "")
macro(add_rust_feature)
    set(ONE_VALUE_ARGS NAME DESCRIPTION DEFAULT_VALUE RUST_FEATURE)
    cmake_parse_arguments(ADD_RUST_FEATURE "" "${ONE_VALUE_ARGS}" "" ${ARGN})

    option(${ADD_RUST_FEATURE_NAME} ${ADD_RUST_FEATURE_DESCRIPTION} ${ADD_RUST_FEATURE_DEFAULT_VALUE})
    message(STATUS "  ${ADD_RUST_FEATURE_NAME}: ${${ADD_RUST_FEATURE_NAME}} (Description: ${ADD_RUST_FEATURE_DESCRIPTION})")

    if(${ADD_RUST_FEATURE_NAME})
        list(APPEND IOX2_RUST_FEATURES ${ADD_RUST_FEATURE_RUST_FEATURE})
    endif()
endmacro()

message(STATUS "iceoryx2 options:")

add_option(
    NAME BUILD_CXX_BINDING
    DESCRIPTION "Build C++ binding"
    DEFAULT_VALUE ON
)

add_option(
    NAME BUILD_EXAMPLES
    DESCRIPTION "Build examples"
    DEFAULT_VALUE OFF
)

add_option(
    NAME BUILD_TESTING
    DESCRIPTION "Build tests"
    DEFAULT_VALUE OFF
)

add_option(
    NAME SANITIZERS
    DESCRIPTION "Build with undefined-behavior- and address-sanitizer"
    DEFAULT_VALUE OFF
)

add_option(
    NAME WARNING_AS_ERROR
    DESCRIPTION "Fails if the compiler emits a warning"
    DEFAULT_VALUE OFF
)

add_param(
    NAME RUST_BUILD_ARTIFACT_PATH
    DESCRIPTION "The path to the folder with the Rust build artifacts, e.g. '/full/path/to/iceoryx2/target/release'"
    DEFAULT_VALUE ""
)

add_param(
    NAME RUST_TARGET_TRIPLET
    DESCRIPTION "The target triplet for cross compilation when 'RUST_BUILD_ARTIFACT_PATH' is not set, e.g. 'aarch64-unknown-linux-gnu'"
    DEFAULT_VALUE ""
)

message(STATUS "iceoryx2 Rust feature flags (only used when 'RUST_BUILD_ARTIFACT_PATH' is not set):")

add_rust_feature(
    NAME IOX2_FEATURE_DEV_PERMISSIONS
    DESCRIPTION "The permissions of all resources will be set to read, write, execute for everyone."
    DEFAULT_VALUE OFF
    RUST_FEATURE "iceoryx2/dev_permissions"
)

add_rust_feature(
    NAME IOX2_FEATURE_LIBC_PLATFORM
    DESCRIPTION "A platform abstraction based on the libc crate, eliminating the need for bindgen. Only available on Linux."
    DEFAULT_VALUE OFF
    RUST_FEATURE "iceoryx2/libc_platform"
)

add_rust_feature(
    NAME IOX2_FEATURE_LOGGER_LOG
    DESCRIPTION "Enables https://crates.io/crates/log as default logger"
    DEFAULT_VALUE OFF
    RUST_FEATURE "iceoryx2/logger_log"
)

add_rust_feature(
    NAME IOX2_FEATURE_LOGGER_TRACING
    DESCRIPTION "Enables https://crates.io/crates/tracing as default logger"
    DEFAULT_VALUE OFF
    RUST_FEATURE "iceoryx2/logger_tracing"
)

if(WARNING_AS_ERROR)
    if(WIN32)
        set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS}")
        set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS}")
    else()
        set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -Werror -Wall -Wextra -Wpedantic")
        set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -Werror -Wall -Wextra -Wpedantic")
    endif()
endif()

if(SANITIZERS)
    set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -fsanitize=address -fsanitize=undefined")
    set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -fsanitize=address -fsanitize=undefined")
endif()

# C binding
add_subdirectory(iceoryx2-ffi/c)

if(BUILD_EXAMPLES)
    add_subdirectory(examples/c)
endif()

# C++ binding
if(BUILD_CXX_BINDING)
    add_subdirectory(iceoryx2-ffi/cxx)

    if(BUILD_EXAMPLES)
        add_subdirectory(examples/cxx)
    endif()
endif()
