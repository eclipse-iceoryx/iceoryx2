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
project(iceoryx2-workspace VERSION ${IOX2_VERSION})

set(CMAKE_EXPORT_COMPILE_COMMANDS ON) # "Create compile_commands.json file"

add_subdirectory(iceoryx2-cmake-modules)

find_package(iceoryx2-cmake-modules REQUIRED)
include(Iceoryx2OptionAndParamMacros)

# BEGIN Helper macro to define Rust feature flags when iceoryx2 is build via cmake

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

# END Helper macro to define Rust feature flags when iceoryx2 is build via cmake


# BEGIN Options, Params and Feature Flags

message(STATUS "[i] iceoryx2 dev setup options and params:")

add_option(
    NAME BUILD_CXX
    DESCRIPTION "Build C++ components, e.g. container and bindings"
    DEFAULT_VALUE ON
)

add_option(
    NAME BUILD_EXAMPLES
    DESCRIPTION "Build examples"
    DEFAULT_VALUE OFF
)

add_param(
    NAME RUST_TARGET_TRIPLET
    DESCRIPTION "The target triplet for cross compilation when 'RUST_BUILD_ARTIFACT_PATH' is not set, e.g. 'aarch64-unknown-linux-gnu'"
    DEFAULT_VALUE ""
)

message(STATUS "[i] iceoryx2 dev setup Rust feature flags (only used when 'RUST_BUILD_ARTIFACT_PATH' is not set):")

add_rust_feature(
    NAME IOX2_FEATURE_STD
    DESCRIPTION "Build iceoryx2 with std library"
    DEFAULT_VALUE ON
    RUST_FEATURE "iceoryx2-ffi-c/std"
)

add_rust_feature(
    NAME IOX2_FEATURE_DEV_PERMISSIONS
    DESCRIPTION "The permissions of all resources will be set to read, write, execute for everyone"
    DEFAULT_VALUE OFF
    RUST_FEATURE "iceoryx2/dev_permissions"
)

add_rust_feature(
    NAME IOX2_FEATURE_LIBC_PLATFORM
    DESCRIPTION "A platform abstraction based on the libc crate, eliminating the need for bindgen. Only available on Linux"
    DEFAULT_VALUE OFF
    RUST_FEATURE "iceoryx2/libc_platform"
)

add_rust_feature(
    NAME IOX2_FEATURE_LOGGER_STD
    DESCRIPTION "Use the std module in the logger backend"
    DEFAULT_VALUE ON
    RUST_FEATURE "iceoryx2-bb-loggers/std"
)

add_rust_feature(
    NAME IOX2_FEATURE_LOGGER_POSIX
    DESCRIPTION "Use the POSIX abstraction in the logger backend"
    DEFAULT_VALUE OFF
    RUST_FEATURE "iceoryx2-bb-loggers/posix"
)

add_rust_feature(
    NAME IOX2_FEATURE_LOGGER_BARE_METAL
    DESCRIPTION "Use bare metal abstractions in the logger backend"
    DEFAULT_VALUE OFF
    RUST_FEATURE "iceoryx2-bb-loggers/bare_metal"
)

add_rust_feature(
    NAME IOX2_FEATURE_LOGGER_BUFFER
    DESCRIPTION "Sets the buffer logger as the default logger backend"
    DEFAULT_VALUE OFF
    RUST_FEATURE "iceoryx2-bb-loggers/buffer"
)

add_rust_feature(
    NAME IOX2_FEATURE_LOGGER_CONSOLE
    DESCRIPTION "Sets the console logger as the default logger backend"
    DEFAULT_VALUE ON
    RUST_FEATURE "iceoryx2-bb-loggers/console"
)

add_rust_feature(
    NAME IOX2_FEATURE_LOGGER_FILE
    DESCRIPTION "Sets the file logger as the default logger backend"
    DEFAULT_VALUE OFF
    RUST_FEATURE "iceoryx2-bb-loggers/file"
)

add_rust_feature(
    NAME IOX2_FEATURE_LOGGER_LOG
    DESCRIPTION "Enables https://crates.io/crates/log as default logger"
    DEFAULT_VALUE OFF
    RUST_FEATURE "iceoryx2-bb-loggers/log"
)

add_rust_feature(
    NAME IOX2_FEATURE_LOGGER_TRACING
    DESCRIPTION "Enables https://crates.io/crates/tracing as default logger"
    DEFAULT_VALUE OFF
    RUST_FEATURE "iceoryx2-bb-loggers/tracing"
)

# END Options, Params and Feature Flags


include(Iceoryx2CommonOptionsAndParams)
include(iceoryx2-bb/cxx/cmake/options.cmake)
include(iceoryx2-c/cmake/options.cmake)
include(iceoryx2-cxx/cmake/options.cmake)
include(Iceoryx2TestOptionsAndParams)

if(BUILD_TESTING)
    enable_testing()
endif()


# BEGIN C binding

if(RUST_BUILD_ARTIFACT_PATH AND NOT "${RUST_BUILD_ARTIFACT_PATH}" STREQUAL "" )
    add_subdirectory(iceoryx2-c)
else()
    message(WARNING
        "Using cargo to build the Rust part of iceoryx2. "
        "This is fine for development but for production, it is "
        "recommended to use an existing installation with\n"
        "'-DRUST_BUILD_ARTIFACT_PATH=/full/path/to/iceoryx2/target/release'!"
    )

    set(RUST_BUILD_TYPE "release")
    set(RUST_BUILD_TYPE_FLAG "--${RUST_BUILD_TYPE}")
    if (CMAKE_BUILD_TYPE STREQUAL "Debug")
        set(RUST_BUILD_TYPE "debug")
        set(RUST_BUILD_TYPE_FLAG "")
    endif()

    if("${RUST_TARGET_TRIPLET}" STREQUAL "")
        set(RUST_TARGET_DIR ${CMAKE_BINARY_DIR}/rust/native)
        set(RUST_BUILD_ARTIFACT_PATH ${CMAKE_BINARY_DIR}/rust/native/${RUST_BUILD_TYPE})
        set(RUST_TARGET_TRIPLET_FLAG "")
    else()
        set(RUST_TARGET_DIR ${CMAKE_BINARY_DIR}/rust)
        set(RUST_BUILD_ARTIFACT_PATH ${CMAKE_BINARY_DIR}/rust/${RUST_TARGET_TRIPLET}/${RUST_BUILD_TYPE})
        set(RUST_TARGET_TRIPLET_FLAG "--target=${RUST_TARGET_TRIPLET}")
    endif()

    set(RUST_FEATURE_FLAGS "")
    list(LENGTH IOX2_RUST_FEATURES IOX2_RUST_FEATURES_COUNT)
    if(IOX2_RUST_FEATURES_COUNT GREATER 0)
        list(JOIN IOX2_RUST_FEATURES "," RUST_FEATURE_FLAGS_STRING)
        set(RUST_FEATURE_FLAGS "--features=${RUST_FEATURE_FLAGS_STRING}")
    endif()

    include(iceoryx2-c/cmake/rust-ffi-c-byproduct-definitions.cmake)

    # run cargo
    add_custom_target(
        iceoryx2-ffi-c-build-step ALL
        COMMAND cargo build ${RUST_BUILD_TYPE_FLAG} --no-default-features ${RUST_FEATURE_FLAGS} --package iceoryx2-ffi-c --target-dir=${RUST_TARGET_DIR} ${RUST_TARGET_TRIPLET_FLAG}
        BYPRODUCTS
            ${ICEORYX2_C_INCLUDE_DIR}/iox2/iceoryx2.h
            ${ICEORYX2_C_STATIC_LIB_LINK_FILE}
            ${ICEORYX2_C_SHARED_LIB_LINK_FILE}
            ${ICEORYX2_C_SHARED_LIB_DLL_FILE}
        WORKING_DIRECTORY ${CMAKE_CURRENT_LIST_DIR}
        VERBATIM
        USES_TERMINAL
    )

    add_subdirectory(iceoryx2-c)

    add_dependencies(includes-only iceoryx2-ffi-c-build-step)
endif()

# END C binding

if(BUILD_EXAMPLES)
    add_subdirectory(examples/c)
endif()


if(BUILD_CXX)
    # C++ containers and building blocks
    add_subdirectory(iceoryx2-bb/cxx)

    # C++ binding
    add_subdirectory(iceoryx2-cxx)

    if(BUILD_EXAMPLES)
        add_subdirectory(examples/cxx)
    endif()
    if(BUILD_TESTING)
        add_subdirectory(component-tests/cxx)
    endif()
endif()
