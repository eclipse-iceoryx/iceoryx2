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

set(IOX2_VERSION 0.7.0)
project(iceoryx2-c VERSION ${IOX2_VERSION} LANGUAGES C)

set(PREFIX iceoryx2/v${CMAKE_PROJECT_VERSION})
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

find_package(iceoryx2-cmake-modules REQUIRED)
include(Iceoryx2OptionAndParamMacros)

include(Iceoryx2CommonOptionsAndParams)
include(cmake/options.cmake)

include(Iceoryx2PlatformSettings)

#
########## C target ##########
#

include(cmake/rust-ffi-c-byproduct-definitions.cmake)

# includes lib

add_library(includes-only INTERFACE)
add_library(iceoryx2-c::includes-only ALIAS includes-only)

target_include_directories(includes-only
    INTERFACE
    $<BUILD_INTERFACE:${ICEORYX2_C_INCLUDE_DIR}>
    $<INSTALL_INTERFACE:include/${PREFIX}>
)

# static lib

add_library(static-lib INTERFACE)
add_library(iceoryx2-c::static-lib ALIAS static-lib)

# shared lib

add_library(shared-lib INTERFACE)
add_library(iceoryx2-c::shared-lib ALIAS shared-lib)

# NOTE the solution with specifying a 'BUILD_INTERFACE' and a 'INSTALL_INTERFACE' was the only one that worked;
# alternatively an imported library could be created, e.g. 'iceoryx2_ffi' with a corresponding alias, e.g. 'iceoryx2::ffi';
# the imported lib needs to be present here and also in 'Config.cmake.in' but they need to be named differently, e.g.
# 'iceoryx2_ffi_imported' in 'Config.cmake.in' but with the same alias, e.g 'iceoryx2::ffi';
# the 'target_link_libraries' would then depend on 'iceoryx2::ffi'
include(GNUInstallDirs)
target_link_libraries(static-lib INTERFACE
    iceoryx2-c::includes-only
    $<BUILD_INTERFACE:${ICEORYX2_C_STATIC_LIB_LINK_FILE}>
    $<INSTALL_INTERFACE:$<INSTALL_PREFIX>/${CMAKE_INSTALL_LIBDIR}/${ICEORYX2_C_STATIC_LIB_LINK_NAME}>
    $<$<NOT:$<PLATFORM_ID:Windows>>:m pthread>
    $<$<PLATFORM_ID:Windows>:userenv ntdll psapi ws2_32 wsock32>
    $<$<AND:$<PLATFORM_ID:Windows>,$<OR:$<C_COMPILER_ID:MSVC>,$<CXX_COMPILER_ID:MSVC>>>:bcrypt synchronization>
    $<$<PLATFORM_ID:Darwin>:stdc++>
    $<$<PLATFORM_ID:FreeBSD>:rt util>
    $<$<PLATFORM_ID:Linux>:dl rt>
)
target_link_libraries(shared-lib INTERFACE
    iceoryx2-c::includes-only
    $<BUILD_INTERFACE:${ICEORYX2_C_SHARED_LIB_LINK_FILE}>
    $<INSTALL_INTERFACE:$<INSTALL_PREFIX>/${CMAKE_INSTALL_LIBDIR}/${ICEORYX2_C_SHARED_LIB_LINK_NAME}>
)

include(${CMAKE_CURRENT_LIST_DIR}/cmake/install.cmake)

# Tests
if(${BUILD_TESTING})
    add_subdirectory(tests)
endif()
