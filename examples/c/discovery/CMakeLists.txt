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
project(example_c_discovery LANGUAGES C)

find_package(iceoryx2-c 0.7.0 REQUIRED)

add_executable(example_c_discovery src/main.c)

# NOTE we use this example to test the builds for the shared library but this build
# is partially broken on macOS; see also #233
target_link_libraries(example_c_discovery
   $<$<NOT:$<PLATFORM_ID:Darwin>>:iceoryx2-c::shared-lib>
   $<$<PLATFORM_ID:Darwin>:iceoryx2-c::static-lib>
)
