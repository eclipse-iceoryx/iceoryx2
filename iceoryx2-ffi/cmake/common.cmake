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

if(NOT DEFINED IOX2_VERSION_STRING)
    set(IOX2_VERSION_STRING "0.6.1")
endif()

set(IOX2_RUST_FEATURES "")

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

macro(add_rust_feature)
    set(ONE_VALUE_ARGS NAME DESCRIPTION DEFAULT_VALUE RUST_FEATURE)
    cmake_parse_arguments(ADD_RUST_FEATURE "" "${ONE_VALUE_ARGS}" "" ${ARGN})
    option(${ADD_RUST_FEATURE_NAME} ${ADD_RUST_FEATURE_DESCRIPTION} ${ADD_RUST_FEATURE_DEFAULT_VALUE})
    message(STATUS "  ${ADD_RUST_FEATURE_NAME}: ${${ADD_RUST_FEATURE_NAME}} (Description: ${ADD_RUST_FEATURE_DESCRIPTION})")
    if(${ADD_RUST_FEATURE_NAME})
        list(APPEND IOX2_RUST_FEATURES ${ADD_RUST_FEATURE_RUST_FEATURE})
    endif()
endmacro()
