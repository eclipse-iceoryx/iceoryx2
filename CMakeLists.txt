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

cmake_minimum_required(VERSION 3.28)
set(IOX2_VERSION_STRING "0.3.0")
project(iceoryx2 VERSION ${IOX2_VERSION_STRING})

#TODO how to handle feature flags

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

message(STATUS "iceoryx2 options:")

add_option(
    NAME EXAMPLES
    DESCRIPTION "Build examples"
    DEFAULT_VALUE OFF
)

add_param(
    NAME RUST_TARGET_TRIPLET
    DESCRIPTION "The target triplet for cross compilation, e.g. 'aarch64-unknown-linux-gnu'"
    DEFAULT_VALUE ""
)

add_subdirectory(iceoryx2-ffi/c)

if(EXAMPLES)
    add_subdirectory(examples/c)
endif()
