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

cmake_minimum_required(VERSION 3.22)
project(example_cxx_request_response LANGUAGES CXX)

find_package(iceoryx2-cxx 0.7.0 REQUIRED)

add_executable(example_cxx_request_response_client src/client.cpp)
target_link_libraries(example_cxx_request_response_client iceoryx2-cxx::static-lib-cxx)

add_executable(example_cxx_request_response_server src/server.cpp)
target_link_libraries(example_cxx_request_response_server iceoryx2-cxx::static-lib-cxx)

