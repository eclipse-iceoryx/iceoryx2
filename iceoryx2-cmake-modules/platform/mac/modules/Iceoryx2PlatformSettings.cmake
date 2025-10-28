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

set(ICEORYX2_CXX_STD_VALUE      17 CACHE INTERNAL "")
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

set(ICEORYX2_LINK_FLAGS CACHE INTERNAL "")

if(DEFINED SANITIZERS AND NOT SANITIZERS STREQUAL "")
    include(Iceoryx2Sanitizer)

    set(ICEORYX2_C_FLAGS        ${ICEORYX2_C_FLAGS} ${ICEORYX2_SANITIZER_FLAGS} CACHE INTERNAL "")
    set(ICEORYX2_CXX_FLAGS      ${ICEORYX2_CXX_FLAGS} ${ICEORYX2_SANITIZER_FLAGS} CACHE INTERNAL "")
    set(ICEORYX2_TEST_CXX_FLAGS ${ICEORYX2_TEST_CXX_FLAGS} ${ICEORYX2_SANITIZER_FLAGS} CACHE INTERNAL "")
    
    # Sanitizer flags must also be added to linker flags
    set(ICEORYX2_LINK_FLAGS     ${ICEORYX2_LINK_FLAGS} ${ICEORYX2_SANITIZER_FLAGS} CACHE INTERNAL "")
    
endif()
