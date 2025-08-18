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

# check if iceoryx is in CMAKE_PREFIX_PATH
find_package(iceoryx_platform ${ICEORYX_HOOFS_VERSION} QUIET)
find_package(iceoryx_hoofs ${ICEORYX_HOOFS_VERSION} QUIET)

# fetch iceoryx if not found
if(NOT iceoryx_platform_FOUND OR NOT iceoryx_hoofs_FOUND)
    if(iceoryx_platform_FOUND)
        message(FATAL_ERROR "iceoryx_platform was found with 'find_package' but other parts were not found!")
    endif()
    if(iceoryx_hoofs_FOUND)
        message(FATAL_ERROR "iceoryx_hoofs was found with 'find_package' but other parts were not found!")
    endif()

    include(FetchContent)
    FetchContent_Declare(
        iceoryx
        GIT_REPOSITORY https://github.com/eclipse-iceoryx/iceoryx.git
        GIT_TAG v${ICEORYX_HOOFS_VERSION}
        EXCLUDE_FROM_ALL
    )
    FetchContent_GetProperties(iceoryx)
    if (NOT iceoryx_POPULATED)
        message(STATUS "iceoryx_hoofs not found! Using FetchContent!")
    endif()
    FetchContent_MakeAvailable(iceoryx)

    set(ICEORYX_WITH_FETCH_CONTENT true CACHE INTERNAL "")
    set(iceoryx_SOURCE_DIR ${iceoryx_SOURCE_DIR} CACHE INTERNAL "")
    set(iceoryx_BINARY_DIR ${iceoryx_BINARY_DIR} CACHE INTERNAL "")
endif()

if(ICEORYX_WITH_FETCH_CONTENT)
    # turn off every option which is not required to build iceoryx hoofs
    set(EXAMPLES OFF)
    set(BUILD_TEST OFF)

    # use iceoryx platform and hoofs in source code version
    add_subdirectory(${iceoryx_SOURCE_DIR}/iceoryx_platform  ${iceoryx_BINARY_DIR}/iceoryx_platform EXCLUDE_FROM_ALL)
    add_subdirectory(${iceoryx_SOURCE_DIR}/iceoryx_hoofs ${iceoryx_BINARY_DIR}/iceoryx_hoofs EXCLUDE_FROM_ALL)

    find_package(iceoryx_platform ${ICEORYX_HOOFS_VERSION} REQUIRED)
    find_package(iceoryx_hoofs ${ICEORYX_HOOFS_VERSION} REQUIRED)
endif()

if(ICEORYX_WITH_FETCH_CONTENT)
message(WARNING
    "The project was built by obtaining iceoryx with FetchContent. "
    "Language bindings produced by this build are not relocatable, "
    "so they have been removed from the install target. "
    "This is fine for development, but for production it is "
    "recommended to use an existing installation with\n"
    "'-DCMAKE_PREFIX_PATH=/full/path/to/installed/iceoryx'! "
)
endif()
