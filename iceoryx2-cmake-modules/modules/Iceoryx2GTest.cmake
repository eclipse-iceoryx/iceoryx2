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

if(USE_SYSTEM_GTEST)

    find_package(GTest REQUIRED)

else()

    include(FetchContent)

    FetchContent_Declare(
        googletest
        GIT_REPOSITORY https://github.com/google/googletest.git
        GIT_TAG        v1.14.0
        EXCLUDE_FROM_ALL
    )

    FetchContent_GetProperties(googletest)
    if(NOT googletest_POPULATED)
        message(STATUS "googletest not found! Using FetchContent!")
    endif()
    FetchContent_MakeAvailable(googletest)

endif()
