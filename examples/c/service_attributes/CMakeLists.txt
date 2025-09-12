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
project(example_c_service_attributes LANGUAGES C)

find_package(iceoryx2-c 0.7.0 REQUIRED)

add_executable(example_c_service_attributes_creator src/creator.c)
target_link_libraries(example_c_service_attributes_creator iceoryx2-c::static-lib)

add_executable(example_c_service_attributes_opener src/opener.c)
target_link_libraries(example_c_service_attributes_opener iceoryx2-c::static-lib)

add_executable(example_c_service_attributes_incompatible src/incompatible.c)
target_link_libraries(example_c_service_attributes_incompatible iceoryx2-c::static-lib)
