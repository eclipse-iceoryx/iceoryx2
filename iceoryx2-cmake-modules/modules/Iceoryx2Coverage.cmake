# Copyright (c) 2026 Contributors to the Eclipse Foundation
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

if (NOT CMAKE_BUILD_TYPE MATCHES "Debug")
    message( FATAL_ERROR "You need to set -DCMAKE_BUILD_TYPE=Debug to run with Coverage" )
endif()

set(CMAKE_CXX_OUTPUT_EXTENSION_REPLACE 1)
if (CMAKE_CXX_COMPILER_ID STREQUAL "GNU" OR (NOT CMAKE_CXX_COMPILER_ID AND CMAKE_C_COMPILER_ID STREQUAL "GNU"))
    set(ICEORYX2_COVERAGE_FLAGS --coverage -fprofile-abs-path CACHE INTERNAL "")
else()
    message( FATAL_ERROR "You need to run gcov with gcc compiler." )
endif()
