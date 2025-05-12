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
project(examples_c LANGUAGES C)

add_subdirectory(discovery)
add_subdirectory(domains)
add_subdirectory(event)
add_subdirectory(event_multiplexing)
add_subdirectory(publish_subscribe)
add_subdirectory(publish_subscribe_cross_language)
add_subdirectory(publish_subscribe_with_user_header)
add_subdirectory(request_response)
