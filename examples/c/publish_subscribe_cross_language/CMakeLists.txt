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
project(example_c_publish_subscribe_cross_language LANGUAGES C)

find_package(iceoryx2-c 0.7.0 REQUIRED)

add_executable(example_c_cross_language_publisher src/publisher.c)
target_link_libraries(example_c_cross_language_publisher iceoryx2-c::static-lib)

add_executable(example_c_cross_language_subscriber src/subscriber.c)
target_link_libraries(example_c_cross_language_subscriber iceoryx2-c::static-lib)
