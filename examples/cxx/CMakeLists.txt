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
project(examples_cxx LANGUAGES CXX)

find_package(iceoryx2-cmake-modules REQUIRED)

include(Iceoryx2PlatformSettings)

set(CMAKE_CXX_STANDARD ${ICEORYX2_CXX_STD_VALUE})
set(CMAKE_CXX_STANDARD_REQUIRED ON)

string(JOIN " " ICEORYX2_C_FLAGS_STRING ${ICEORYX2_C_FLAGS})
string(JOIN " " ICEORYX2_C_WARNINGS_STRING ${ICEORYX2_C_WARNINGS})
string(JOIN " " ICEORYX2_CXX_FLAGS_STRING ${ICEORYX2_CXX_FLAGS})
string(JOIN " " ICEORYX2_CXX_WARNINGS_STRING ${ICEORYX2_CXX_WARNINGS})
string(JOIN " " ICEORYX2_SANITIZER_FLAGS_STRING ${ICEORYX2_SANITIZER_FLAGS})

set(CMAKE_C_FLAGS "\
    ${CMAKE_C_FLAGS} \
    ${ICEORYX2_C_FLAGS_STRING} \
    ${ICEORYX2_C_WARNINGS_STRING}\
")
set(CMAKE_CXX_FLAGS "\
    ${CMAKE_CXX_FLAGS} \
    ${ICEORYX2_CXX_FLAGS_STRING} \
    ${ICEORYX2_CXX_WARNINGS_STRING} \
    ${ICEORYX2_SANITIZER_FLAGS_STRING} \
")

add_subdirectory(complex_data_types)
add_subdirectory(discovery)
add_subdirectory(domains)
add_subdirectory(event)
add_subdirectory(event_multiplexing)
add_subdirectory(event_based_communication)
add_subdirectory(health_monitoring)
add_subdirectory(publish_subscribe)
add_subdirectory(publish_subscribe_cross_language)
add_subdirectory(publish_subscribe_dynamic_data)
add_subdirectory(publish_subscribe_with_user_header)
add_subdirectory(request_response)
add_subdirectory(request_response_dynamic_data)
add_subdirectory(service_attributes)
add_subdirectory(service_types)
