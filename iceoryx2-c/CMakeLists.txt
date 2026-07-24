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

set(IOX2_VERSION 0.9.999)
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
    $<$<NOT:$<PLATFORM_ID:Windows,QNX,Android>>:m pthread>
    $<$<PLATFORM_ID:Windows>:userenv ntdll psapi ws2_32 wsock32>
    $<$<AND:$<PLATFORM_ID:Windows>,$<OR:$<C_COMPILER_ID:MSVC>,$<CXX_COMPILER_ID:MSVC>>>:bcrypt synchronization>
    $<$<PLATFORM_ID:Darwin>:stdc++>
    $<$<PLATFORM_ID:FreeBSD>:rt util>
    $<$<PLATFORM_ID:Linux>:dl rt>
    $<$<PLATFORM_ID:QNX>:stdc++ socket>
)
target_link_libraries(shared-lib INTERFACE
    iceoryx2-c::includes-only
    $<BUILD_INTERFACE:${ICEORYX2_C_SHARED_LIB_LINK_FILE}>
    $<INSTALL_INTERFACE:$<INSTALL_PREFIX>/${CMAKE_INSTALL_LIBDIR}/${ICEORYX2_C_SHARED_LIB_LINK_NAME}>
)

include(${CMAKE_CURRENT_LIST_DIR}/cmake/install.cmake)

# package generation using CPack

set(CPACK_PACKAGE_NAME              "iceoryx2-c")
set(CPACK_PACKAGE_VERSION           ${IOX2_VERSION})
set(CPACK_PACKAGE_VENDOR            "Eclipse Foundation")
set(CPACK_PACKAGE_CONTACT           "iceoryx2")
set(CPACK_PACKAGE_DESCRIPTION_SUMMARY
    "C bindings for the iceoryx2 zero-copy IPC middleware")
set(CPACK_RESOURCE_FILE_LICENSE     "${PROJECT_SOURCE_DIR}/LICENSE-APACHE")

set(CPACK_GENERATOR                 "DEB;STGZ")
set(CPACK_DEB_COMPONENT_INSTALL     ON)
set(CPACK_DEBIAN_PACKAGE_SECTION    "libs")
set(CPACK_DEBIAN_PACKAGE_PRIORITY   "optional")
set(CPACK_DEBIAN_PACKAGE_HOMEPAGE   "https://github.com/eclipse-iceoryx/iceoryx2")

# Derive Rust-style target triplet for the package filename.
if(NOT DEFINED RUST_TARGET_TRIPLET OR "${RUST_TARGET_TRIPLET}" STREQUAL "")
    if(CMAKE_SYSTEM_PROCESSOR MATCHES "x86_64|AMD64")
        set(_IOX2_ARCH "x86_64")
    elseif(CMAKE_SYSTEM_PROCESSOR MATCHES "aarch64|ARM64")
        set(_IOX2_ARCH "aarch64")
    else()
        set(_IOX2_ARCH "${CMAKE_SYSTEM_PROCESSOR}")
    endif()

    if(CMAKE_SYSTEM_NAME STREQUAL "Linux")
        set(RUST_TARGET_TRIPLET "${_IOX2_ARCH}-unknown-linux-gnu")
    elseif(CMAKE_SYSTEM_NAME STREQUAL "Darwin")
        set(RUST_TARGET_TRIPLET "${_IOX2_ARCH}-apple-darwin")
    else()
        set(RUST_TARGET_TRIPLET "${_IOX2_ARCH}-unknown-${CMAKE_SYSTEM_NAME}")
    endif()
    unset(_IOX2_ARCH)
endif()

# Base filename used by all generators.
set(CPACK_PACKAGE_FILE_NAME
    "libiceoryx2-c-${IOX2_VERSION}-${RUST_TARGET_TRIPLET}")

# -- lib component: shared and static libraries (Rust FFI artifacts)
set(CPACK_DEBIAN_LIB_PACKAGE_NAME       "libiceoryx2-c")
set(CPACK_DEBIAN_LIB_PACKAGE_SECTION    "libs")
set(CPACK_DEBIAN_LIB_FILE_NAME
    "libiceoryx2-c-${IOX2_VERSION}-${RUST_TARGET_TRIPLET}.deb")

# -- dev component: headers and CMake config files
set(CPACK_DEBIAN_DEV_PACKAGE_NAME       "libiceoryx2-c-dev")
set(CPACK_DEBIAN_DEV_PACKAGE_SECTION    "libdevel")
set(CPACK_DEBIAN_DEV_DESCRIPTION
    "iceoryx2 C bindings - headers and CMake config")
set(CPACK_DEBIAN_DEV_PACKAGE_DEPENDS
    "libiceoryx2-c (= ${IOX2_VERSION})")
set(CPACK_DEBIAN_DEV_FILE_NAME
    "libiceoryx2-c-dev-${IOX2_VERSION}-${RUST_TARGET_TRIPLET}.deb")

# -- bin component: runtime executables (currently empty)
set(CPACK_DEBIAN_BIN_FILE_NAME
    "libiceoryx2-c-bin-${IOX2_VERSION}-${RUST_TARGET_TRIPLET}.deb")

include(CPack)

# Tests
if(${BUILD_TESTING})
    add_subdirectory(tests)
endif()
