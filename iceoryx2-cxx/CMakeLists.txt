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

set(ICEORYX_HOOFS_VERSION 2.95.7)
set(IOX2_VERSION 0.7.0)
project(iceoryx2-cxx VERSION ${IOX2_VERSION} LANGUAGES CXX)

set(PREFIX iceoryx2/v${CMAKE_PROJECT_VERSION})
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

find_package(iceoryx_hoofs ${ICEORYX_HOOFS_VERSION} REQUIRED)
find_package(iceoryx2-c REQUIRED)

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
    src/server_details.cpp
    src/service.cpp
    src/service_builder_event.cpp
    src/service_id.cpp
    src/service_name.cpp
    src/static_config.cpp
    src/static_config_event.cpp
    src/static_config_publish_subscribe.cpp
    src/static_config_request_response.cpp
    src/subscriber_details.cpp
    src/unique_port_id.cpp
    src/waitset.cpp
)

target_compile_options(
    iceoryx2-cxx-object-lib
    PRIVATE
    ${ICEORYX2_CXX_FLAGS}
    ${ICEORYX2_CXX_WARNINGS}
    ${ICEORYX2_SANITIZER_FLAGS}
)
target_compile_features(iceoryx2-cxx-object-lib PRIVATE ${ICEORYX2_CXX_STD})

set_target_properties(iceoryx2-cxx-object-lib
    PROPERTIES
    POSITION_INDEPENDENT_CODE ON
)

target_link_libraries(iceoryx2-cxx-object-lib
    PUBLIC
    iceoryx_hoofs::iceoryx_hoofs
    iceoryx2-c::includes-only
    iceoryx2-cxx::includes-only-cxx
    ${ICEORYX2_SANITIZER_FLAGS}
)

# static lib

add_library(static-lib-cxx STATIC $<TARGET_OBJECTS:iceoryx2-cxx-object-lib>)
add_library(iceoryx2-cxx::static-lib-cxx ALIAS static-lib-cxx)

target_compile_features(static-lib-cxx PUBLIC cxx_std_17)
set_target_properties(static-lib-cxx
    PROPERTIES
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

target_compile_features(shared-lib-cxx PUBLIC cxx_std_17)
set_target_properties(shared-lib-cxx
    PROPERTIES
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

# Tests
if(${BUILD_TESTING})
    enable_testing()
    add_subdirectory(tests)
endif()
