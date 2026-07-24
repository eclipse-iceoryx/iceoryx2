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
project(iceoryx2-cxx VERSION ${IOX2_VERSION} LANGUAGES CXX)

set(PREFIX iceoryx2/v${CMAKE_PROJECT_VERSION})
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

find_package(iceoryx2-c REQUIRED)
find_package(iceoryx2-bb-cxx REQUIRED)

find_package(iceoryx2-cmake-modules REQUIRED)
include(Iceoryx2OptionAndParamMacros)

include(Iceoryx2CommonOptionsAndParams)
include(cmake/options.cmake)

include(Iceoryx2PlatformSettings)

# include only lib -> includes are installed only once despite being used in static lib as well as shared lib

add_library(includes-only-cxx INTERFACE)
add_library(iceoryx2-cxx::includes-only-cxx ALIAS includes-only-cxx)

target_include_directories(includes-only-cxx
    INTERFACE
    $<BUILD_INTERFACE:${PROJECT_SOURCE_DIR}/include>
    $<INSTALL_INTERFACE:include/${PREFIX}>
)

# object lib

add_library(iceoryx2-cxx-object-lib OBJECT
    src/attribute.cpp
    src/attribute_set.cpp
    src/attribute_specifier.cpp
    src/attribute_verifier.cpp
    src/client_details.cpp
    src/config.cpp
    src/dynamic_config_blackboard.cpp
    src/dynamic_config_event.cpp
    src/dynamic_config_publish_subscribe.cpp
    src/dynamic_config_request_response.cpp
    src/event_activation.cpp
    src/event_id.cpp
    src/file_descriptor.cpp
    src/header_publish_subscribe.cpp
    src/header_request_response.cpp
    src/listener_details.cpp
    src/log.cpp
    src/message_type_details.cpp
    src/messaging_pattern.cpp
    src/node.cpp
    src/node_details.cpp
    src/node_name.cpp
    src/node_state.cpp
    src/notifier.cpp
    src/notifier_details.cpp
    src/port_factory_event.cpp
    src/port_factory_notifier.cpp
    src/port_name.cpp
    src/publisher_details.cpp
    src/reader_details.cpp
    src/server_details.cpp
    src/service.cpp
    src/service_builder_event.cpp
    src/service_hash.cpp
    src/service_name.cpp
    src/static_config.cpp
    src/static_config_blackboard.cpp
    src/static_config_event.cpp
    src/static_config_publish_subscribe.cpp
    src/static_config_request_response.cpp
    src/subscriber_details.cpp
    src/type_variant.cpp
    src/unique_node_id.cpp
    src/unique_port_id.cpp
    src/waitset.cpp
    src/writer_details.cpp
)

target_compile_options(
    iceoryx2-cxx-object-lib
    PRIVATE
    ${ICEORYX2_CXX_FLAGS}
    ${ICEORYX2_CXX_WARNINGS}
    ${ICEORYX2_SANITIZER_FLAGS}
    ${ICEORYX2_COVERAGE_FLAGS}
)
target_compile_features(iceoryx2-cxx-object-lib PRIVATE ${ICEORYX2_CXX_STD})

set_target_properties(iceoryx2-cxx-object-lib
    PROPERTIES
    POSITION_INDEPENDENT_CODE ON
)

target_link_libraries(iceoryx2-cxx-object-lib
    PUBLIC
    iceoryx2-bb-cxx::iceoryx2-bb-cxx
    iceoryx2-c::includes-only
    iceoryx2-cxx::includes-only-cxx
    ${ICEORYX2_SANITIZER_FLAGS}
    ${ICEORYX2_COVERAGE_FLAGS}
)

# static lib

add_library(static-lib-cxx STATIC $<TARGET_OBJECTS:iceoryx2-cxx-object-lib>)
add_library(iceoryx2-cxx::static-lib-cxx ALIAS static-lib-cxx)

target_compile_features(static-lib-cxx PUBLIC ${ICEORYX2_CXX_STD})
set_target_properties(static-lib-cxx
    PROPERTIES
    POSITION_INDEPENDENT_CODE ON
    OUTPUT_NAME "iceoryx2_cxx"
)

target_link_libraries(static-lib-cxx
    PUBLIC
    iceoryx2-bb-cxx::iceoryx2-bb-cxx
    iceoryx2-c::static-lib
    iceoryx2-cxx::includes-only-cxx
)

# shared lib

add_library(shared-lib-cxx SHARED $<TARGET_OBJECTS:iceoryx2-cxx-object-lib>)
add_library(iceoryx2-cxx::shared-lib-cxx ALIAS shared-lib-cxx)

target_compile_features(shared-lib-cxx PUBLIC ${ICEORYX2_CXX_STD})
set_target_properties(shared-lib-cxx
    PROPERTIES
    POSITION_INDEPENDENT_CODE ON
    OUTPUT_NAME "iceoryx2_cxx"
)

target_link_libraries(shared-lib-cxx
    PUBLIC
    iceoryx2-bb-cxx::iceoryx2-bb-cxx
    iceoryx2-c::shared-lib
    iceoryx2-cxx::includes-only-cxx
)

# include install setup

include(${CMAKE_CURRENT_LIST_DIR}/cmake/install.cmake)

# package generation using CPack

set(CPACK_PACKAGE_NAME              "iceoryx2-cxx")
set(CPACK_PACKAGE_VERSION           ${IOX2_VERSION})
set(CPACK_PACKAGE_VENDOR            "Eclipse Foundation")
set(CPACK_PACKAGE_CONTACT           "iceoryx2")
set(CPACK_PACKAGE_DESCRIPTION_SUMMARY
    "C++ bindings for the iceoryx2 zero-copy IPC middleware")
set(CPACK_RESOURCE_FILE_LICENSE     "${PROJECT_SOURCE_DIR}/LICENSE-APACHE")

set(CPACK_GENERATOR                 "DEB;STGZ")
set(CPACK_DEB_COMPONENT_INSTALL     ON)
set(CPACK_DEBIAN_PACKAGE_SECTION    "libs")
set(CPACK_DEBIAN_PACKAGE_PRIORITY   "optional")
set(CPACK_DEBIAN_PACKAGE_HOMEPAGE   "https://github.com/eclipse-iceoryx/iceoryx2")

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

# Base filename used by all generators (STGZ, TGZ, etc.).
set(CPACK_PACKAGE_FILE_NAME
    "libiceoryx2-cxx-${IOX2_VERSION}-${RUST_TARGET_TRIPLET}")

# -- lib component: shared library
set(CPACK_DEBIAN_LIB_PACKAGE_NAME       "libiceoryx2-cxx")
set(CPACK_DEBIAN_LIB_PACKAGE_SECTION    "libs")
set(CPACK_DEBIAN_LIB_FILE_NAME
    "libiceoryx2-cxx-${IOX2_VERSION}-${RUST_TARGET_TRIPLET}.deb")

# -- dev component: headers, static library, CMake config files, license documents
set(CPACK_DEBIAN_DEV_PACKAGE_NAME       "libiceoryx2-cxx-dev")
set(CPACK_DEBIAN_DEV_PACKAGE_SECTION    "libdevel")
set(CPACK_DEBIAN_DEV_DESCRIPTION
    "C++ bindings for iceoryx2 - development files (static library, headers, CMake config)")
set(CPACK_DEBIAN_DEV_PACKAGE_DEPENDS
    "libiceoryx2-cxx (= ${IOX2_VERSION}), libiceoryx2-bb-cxx-dev (= ${IOX2_VERSION}), libiceoryx2-c-dev (= ${IOX2_VERSION})")
set(CPACK_DEBIAN_DEV_FILE_NAME
    "libiceoryx2-cxx-dev-${IOX2_VERSION}-${RUST_TARGET_TRIPLET}.deb")

include(CPack)

# Tests
if(${BUILD_TESTING})
  enable_testing()
  add_subdirectory(tests)
endif()
