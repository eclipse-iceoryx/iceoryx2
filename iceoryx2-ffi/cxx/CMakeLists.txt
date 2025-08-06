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

include(${CMAKE_CURRENT_LIST_DIR}/../cmake/common.cmake)
if(NOT DEFINED IOX2_VERSION_STRING OR IOX2_VERSION_STRING STREQUAL "")
    message(FATAL_ERROR 
        "IOX2_VERSION_STRING is not defined!\n"
    )
endif()

project(iceoryx2-cxx VERSION ${IOX2_VERSION_STRING} LANGUAGES CXX)

set(PREFIX iceoryx2/v${CMAKE_PROJECT_VERSION})
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

set(ICEORYX_HOOFS_VERSION 2.95.6)
include(${CMAKE_CURRENT_LIST_DIR}/cmake/fetch-iceoryx-hoofs.cmake)

message(STATUS "Building iceoryx2 C++ binding")
message(STATUS "iceoryx2 options:")

#
########## CMake Options ##########
#

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

if(WARNING_AS_ERROR)
    if(WIN32)
        set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS}")
    else()
        set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -Werror -Wall -Wextra -Wpedantic")
    endif()
endif()

if(SANITIZERS)
    set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -fsanitize=address -fsanitize=undefined")
endif()

find_package(iceoryx_hoofs ${ICEORYX_HOOFS_VERSION} REQUIRED)
find_package(iceoryx2-c REQUIRED)

if(NOT iceoryx2-c_FOUND)
    message(FATAL_ERROR 
        "iceoryx2-c binding not found!\n"
        "\n"
        "The C++ binding requires the C binding to be installed first.\n"
        "Please build and install the C binding:\n"
        "\n"
        "  cmake -B build-c iceoryx2-ffi/c [options...]\n"
        "  cmake --build build-c\n"
        "  cmake --install build-c --prefix /path/to/install\n"
        "\n"
        "Then add the install location to CMAKE_PREFIX_PATH:\n"
        "  cmake -B build-cxx iceoryx2-ffi/cxx -DCMAKE_PREFIX_PATH=/path/to/install\n"
        "\n"
        "Or set CMAKE_PREFIX_PATH environment variable before running cmake."
    )
endif()

message(STATUS "Found iceoryx2-c: ${iceoryx2-c_DIR}")

# include only lib -> includes are installed only once despite being used in static lib as well as shared lib

add_library(includes-only-cxx INTERFACE)
add_library(iceoryx2-cxx::includes-only-cxx ALIAS includes-only-cxx)

target_include_directories(includes-only-cxx
    INTERFACE
    $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>
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
    src/node_id.cpp
    src/node_name.cpp
    src/node_state.cpp
    src/notifier.cpp
    src/notifier_details.cpp
    src/port_factory_event.cpp
    src/port_factory_notifier.cpp
    src/publisher_details.cpp
    src/reader_details.cpp
    src/server_details.cpp
    src/service.cpp
    src/service_builder_event.cpp
    src/service_id.cpp
    src/service_name.cpp
    src/static_config.cpp
    src/static_config_blackboard.cpp
    src/static_config_event.cpp
    src/static_config_publish_subscribe.cpp
    src/static_config_request_response.cpp
    src/subscriber_details.cpp
    src/unique_port_id.cpp
    src/waitset.cpp
    src/writer_details.cpp
)

set_target_properties(iceoryx2-cxx-object-lib
    PROPERTIES
    CXX_STANDARD 17
    CXX_STANDARD_REQUIRED ON
    POSITION_INDEPENDENT_CODE ON
)

target_link_libraries(iceoryx2-cxx-object-lib
    PUBLIC
    iceoryx_hoofs::iceoryx_hoofs
    iceoryx2-c::includes-only
    iceoryx2-cxx::includes-only-cxx
)

# static lib

add_library(static-lib-cxx STATIC $<TARGET_OBJECTS:iceoryx2-cxx-object-lib>)
add_library(iceoryx2-cxx::static-lib-cxx ALIAS static-lib-cxx)

set_target_properties(static-lib-cxx
    PROPERTIES
    CXX_STANDARD 17
    CXX_STANDARD_REQUIRED ON
    POSITION_INDEPENDENT_CODE ON
    OUTPUT_NAME "iceoryx2_cxx"
)

target_link_libraries(static-lib-cxx
    PUBLIC
    iceoryx_hoofs::iceoryx_hoofs
    iceoryx2-c::static-lib
    iceoryx2-cxx::includes-only-cxx
)

# shared lib

add_library(shared-lib-cxx SHARED $<TARGET_OBJECTS:iceoryx2-cxx-object-lib>)
add_library(iceoryx2-cxx::shared-lib-cxx ALIAS shared-lib-cxx)

set_target_properties(shared-lib-cxx
    PROPERTIES
    CXX_STANDARD 17
    CXX_STANDARD_REQUIRED ON
    POSITION_INDEPENDENT_CODE ON
    OUTPUT_NAME "iceoryx2_cxx"
)

target_link_libraries(shared-lib-cxx
    PUBLIC
    iceoryx_hoofs::iceoryx_hoofs
    iceoryx2-c::shared-lib
    iceoryx2-cxx::includes-only-cxx
)

# include install setup

include(${CMAKE_CURRENT_LIST_DIR}/cmake/install.cmake)

# Examples
if(BUILD_EXAMPLES)
    if(EXISTS ${CMAKE_CURRENT_LIST_DIR}/../../examples/cxx)
        add_subdirectory(${CMAKE_CURRENT_LIST_DIR}/../../examples/cxx examples)
    endif()
endif()

# Tests
if(BUILD_TESTING)
    add_subdirectory(tests)
endif()
