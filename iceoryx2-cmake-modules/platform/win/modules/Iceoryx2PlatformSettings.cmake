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

if(CMAKE_C_COMPILER_ID MATCHES MSVC)
    set(ICEORYX2_CXX_FLAGS      "/EHsc" CACHE INTERNAL "") # TODO same as iceoryx classic; check if we should change the exception handling for iceoryx2
    set(ICEORYX2_TEST_CXX_FLAGS "/bigobj" CACHE INTERNAL "")
endif()

set(ICEORYX2_C_WARNINGS CACHE INTERNAL "")

if(CMAKE_C_COMPILER_ID MATCHES MSVC)
    set(ICEORYX2_C_WARNINGS     "${ICEORYX2_C_WARNINGS} /W0" CACHE INTERNAL "") # TODO same as iceoryx classic; check if we can increase the warning level
endif()
set(ICEORYX2_CXX_WARNINGS       ${ICEORYX2_C_WARNINGS} CACHE INTERNAL "")

if(WARNING_AS_ERROR)
    if(CMAKE_C_COMPILER_ID MATCHES MSVC)
        set(ICEORYX2_C_WARNINGS "${ICEORYX2_C_WARNINGS} /W0" CACHE INTERNAL "") # TODO same as iceoryx classic; set to /WX if possible
    endif()
    set(ICEORYX2_CXX_WARNINGS   "${ICEORYX_CXX_WARNINGS} /W0" CACHE INTERNAL "") # TODO same as iceoryx classic; set to /WX if possible
endif()
