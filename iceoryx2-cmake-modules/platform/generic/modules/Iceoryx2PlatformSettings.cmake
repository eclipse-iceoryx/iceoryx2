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

set(ICEORYX2_CXX_STD_VALUE      14 CACHE INTERNAL "")
set(ICEORYX2_CXX_STD            cxx_std_${ICEORYX2_CXX_STD_VALUE} CACHE INTERNAL "")

set(ICEORYX2_C_FLAGS CACHE INTERNAL "")
set(ICEORYX2_CXX_FLAGS CACHE INTERNAL "")
set(ICEORYX2_TEST_CXX_FLAGS CACHE INTERNAL "")

set(ICEORYX2_C_WARNINGS
    -W
    -Wall
    -Wextra
    -Wpedantic
    -Wuninitialized
    -Wstrict-aliasing
    -Wcast-align
    -Wconversion
    CACHE INTERNAL ""
)
set(ICEORYX2_CXX_WARNINGS       ${ICEORYX2_C_WARNINGS} -Wno-noexcept-type CACHE INTERNAL "")

if(WARNING_AS_ERROR)
    set(ICEORYX2_C_WARNINGS     ${ICEORYX2_C_WARNINGS} -Werror CACHE INTERNAL "")
    set(ICEORYX2_CXX_WARNINGS   ${ICEORYX2_CXX_WARNINGS} -Werror CACHE INTERNAL "")
endif()

set(ICEORYX2_SANITZER_FLAGS CACHE INTERNAL "")
if(SANITIZERS)
    include(Iceoryx2Sanitizer)
endif()

set(ICEORYX2_COVERAGE_FLAGS CACHE INTERNAL "")

if(COVERAGE)

    if (NOT CMAKE_BUILD_TYPE MATCHES "Debug")
        message( FATAL_ERROR "You need to set -DCMAKE_BUILD_TYPE=Debug to run with Coverage" )
    endif()

    set(CMAKE_CXX_OUTPUT_EXTENSION_REPLACE 1)
    if (CMAKE_CXX_COMPILER_ID STREQUAL "GNU")
        set(ICEORYX2_COVERAGE_FLAGS --coverage -fprofile-abs-path CACHE INTERNAL "")
    else()
        message( FATAL_ERROR "You need to run gcov with gcc compiler." )
    endif()
endif()
