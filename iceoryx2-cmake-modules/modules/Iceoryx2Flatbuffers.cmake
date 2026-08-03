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

if(IOX2_DEPENDENCY_USE_SYSTEM_FLATBUFFERS)

    find_package(flatbuffers REQUIRED)

else()

    include(FetchContent)

    FetchContent_Declare(
        flatbuffers
        GIT_REPOSITORY https://github.com/google/flatbuffers.git
        GIT_TAG        v25.12.19 # NOTE: keep in sync with Cargo.toml and MODULE.bazel
        OVERRIDE_FIND_PACKAGE
        EXCLUDE_FROM_ALL
    )

    FetchContent_GetProperties(flatbuffers)
    if(NOT flatbuffers_POPULATED)
        message(STATUS "flatbuffers not found! Using FetchContent!")
    endif()
    set(FLATBUFFERS_BUILD_FLATC ON CACHE BOOL "Build flatbuffers flatc compiler")
    set(FLATBUFFERS_BUILD_TESTS OFF CACHE BOOL "Skip building flatbuffers tests")
    FetchContent_MakeAvailable(flatbuffers)

    find_package(flatbuffers)

endif()

# it seems that the flatbuffer targets are not named consistently, depending on
# how it is obtained, either the namespace is missing or the alias are named differently
# -> add alias to be compatible
if(NOT TARGET flatbuffers::flatbuffers AND TARGET flatbuffers)
    add_library(flatbuffers::flatbuffers ALIAS flatbuffers)
elseif(NOT TARGET flatbuffers::flatbuffers AND TARGET flatbuffers::flatbuffers_shared)
    add_library(flatbuffers::flatbuffers ALIAS flatbuffers::flatbuffers_shared)
endif()
