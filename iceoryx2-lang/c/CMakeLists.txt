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
project(iceoryx2_lang_c VERSION ${IOX2_VERSION_STRING})

set(PREFIX iceoryx2/v${CMAKE_PROJECT_VERSION})

#
########## Rust target ##########
#

set(RUST_BUILD_TYPE "release")
set(RUST_BUILD_TYPE_FLAG "--${RUST_BUILD_TYPE}")
if (CMAKE_BUILD_TYPE STREQUAL "Debug")
    set(RUST_BUILD_TYPE "debug")
    set(RUST_BUILD_TYPE_FLAG "")
endif()

set(RUST_ARCH_TRIPLET "")
set(RUST_ARCH_TRIPLET_FLAG "")
# if(${CMAKE_SYSTEM_PROCESSOR} STREQUAL "i486")
#     set(RUST_ARCH_TRIPLET "i686-unknown-linux-gnu")
#     set(RUST_ARCH_TRIPLET_FLAG "--target=${RUST_ARCH_TRIPLET}")
# elseif(${CMAKE_SYSTEM_PROCESSOR} STREQUAL "armv7l")
#     set(RUST_ARCH_TRIPLET "armv7-unknown-linux-gnueabihf")
#     set(RUST_ARCH_TRIPLET_FLAG "--target=${RUST_ARCH_TRIPLET}")
# elseif(${CMAKE_SYSTEM_PROCESSOR} STREQUAL "aarch64")
#     set(RUST_ARCH_TRIPLET "${CMAKE_SYSTEM_PROCESSOR}-unknown-linux-gnu")
#     set(RUST_ARCH_TRIPLET_FLAG "--target=${RUST_ARCH_TRIPLET}")
# endif()

set(RUST_TARGET_DIR ${CMAKE_BINARY_DIR}/rust)
set(RUST_ARCH_TARGET_DIR ${RUST_TARGET_DIR}/${RUST_ARCH_TRIPLET})

set(ICEORYX2_LANG_C_INCLUDE_DIR ${RUST_ARCH_TARGET_DIR}/${RUST_BUILD_TYPE}/iceoryx2_lang_c_cbindgen/include)

set(ICEORYX2_LANG_C_STATIC_LIB_NAME libiceoryx2_lang_c.a)
set(ICEORYX2_LANG_C_SHARED_LIB_NAME libiceoryx2_lang_c.so)
if(MSVC)
    set(ICEORYX2_LANG_C_STATIC_LIB_NAME iceoryx2_lang_c.lib)
    set(ICEORYX2_LANG_C_SHARED_LIB_NAME iceoryx2_lang_c.dll)
endif()

set(ICEORYX2_LANG_C_STATIC_LIB ${RUST_ARCH_TARGET_DIR}/${RUST_BUILD_TYPE}/${ICEORYX2_LANG_C_STATIC_LIB_NAME})
set(ICEORYX2_LANG_C_SHARED_LIB ${RUST_ARCH_TARGET_DIR}/${RUST_BUILD_TYPE}/${ICEORYX2_LANG_C_SHARED_LIB_NAME})

set(ICEORYX2_LANG_C_LIB_NAME ${ICEORYX2_LANG_C_STATIC_LIB_NAME})
set(ICEORYX2_LANG_C_LIB ${ICEORYX2_LANG_C_STATIC_LIB})
if(BUILD_SHARED_LIBS)
    set(ICEORYX2_LANG_C_LIB_NAME ${ICEORYX2_LANG_C_SHARED_LIB_NAME})
    set(ICEORYX2_LANG_C_LIB ${ICEORYX2_LANG_C_SHARED_LIB})
endif()

# run cargo
add_custom_target(
    iceoryx2_build_step ALL
    COMMAND cargo build ${RUST_BUILD_TYPE_FLAG} --target-dir=${RUST_TARGET_DIR} ${RUST_ARCH_TRIPLET_FLAG}
    BYPRODUCTS
        ${ICEORYX2_LANG_C_INCLUDE_DIR}/iox2/iceoryx2.h
        ${ICEORYX2_LANG_C_STATIC_LIB}
        ${ICEORYX2_LANG_C_SHARED_LIB}
    WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}
    VERBATIM
    USES_TERMINAL
)

#
########## C target ##########
#

add_library(${PROJECT_NAME} INTERFACE)
add_library(iceoryx2::${PROJECT_NAME} ALIAS ${PROJECT_NAME})

add_dependencies(${PROJECT_NAME} iceoryx2_build_step)

target_include_directories(${PROJECT_NAME}
    INTERFACE
    $<BUILD_INTERFACE:${ICEORYX2_LANG_C_INCLUDE_DIR}>
    $<INSTALL_INTERFACE:include/${PREFIX}>
)

# NOTE the solution with specifying a 'BUILD_INTERFACE' and a 'INSTALL_INTERFACE' was the only one that worked;
# alternatively an imported library could be created, e.g. 'iceoryx2_ffi' with a corresponding alias, e.g. 'iceoryx2::ffi';
# the imported lib needs to be present here and also in 'Config.cmake.in' but they need to be named differently, e.g.
# 'iceoryx2_ffi_imported' in 'Config.cmake.in' but with the same alias, e.g 'iceoryx2::ffi';
# the 'target_link_libraries' would then depend on 'iceoryx2::ffi'
include(GNUInstallDirs)
target_link_libraries(${PROJECT_NAME} INTERFACE
    $<BUILD_INTERFACE:${ICEORYX2_LANG_C_LIB}>
    $<INSTALL_INTERFACE:$<INSTALL_PREFIX>/${CMAKE_INSTALL_LIBDIR}/${ICEORYX2_LANG_C_LIB_NAME}>
    $<$<NOT:$<PLATFORM_ID:Windows>>:m>
    $<$<PLATFORM_ID:Windows>:ws2_32 wsock32 ntdll>
)

include(cmake/install.cmake)
