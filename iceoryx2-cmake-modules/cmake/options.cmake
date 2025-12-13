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

if(NOT ICEORYX2_CMAKE_MODULES_OPTIONS_AND_PARAMS_LISTED)
    set(ICEORYX2_CMAKE_MODULES_OPTIONS_AND_PARAMS_LISTED true)

    message(STATUS "[i] iceoryx2-cmake-modules options:")

    add_param(
        NAME IOX2_CXX_STD_VERSION
        DESCRIPTION "The C++ version to build iceoryx2"
        DEFAULT_VALUE ${IOX2_CXX_STD_VERSION_DEFAULT}
    )

endif()
