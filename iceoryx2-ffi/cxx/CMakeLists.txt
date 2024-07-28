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

set(ICEORYX_HOOFS_VERSION 2.95.0)
include(cmake/fetch-iceoryx-hoofs.cmake)

project(iceoryx2-cxx VERSION ${IOX2_VERSION_STRING} LANGUAGES CXX)

set(PREFIX iceoryx2/v${CMAKE_PROJECT_VERSION})

find_package(iceoryx_hoofs ${ICEORYX_HOOFS_VERSION} REQUIRED)
find_package(iceoryx2-c REQUIRED)

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
    src/config.cpp
    src/event_id.cpp
    src/listener.cpp
    src/message_type_details.cpp
    src/node.cpp
    src/node_details.cpp
    src/node_name.cpp
    src/node_state.cpp
    src/notifier.cpp
    src/port_factory_event.cpp
    src/port_factory_listener.cpp
    src/port_factory_notifier.cpp
    src/service.cpp
    src/service_builder_event.cpp
    src/service_name.cpp
    src/static_config.cpp
    src/static_config_event.cpp
    src/static_config_publish_subscribe.cpp
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

include(cmake/install.cmake)

# add tests

if(${BUILD_TESTING})
    add_subdirectory(tests)
endif()
