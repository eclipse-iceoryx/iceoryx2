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

if(NOT ICEORYX2_CXX_OPTIONS_AND_PARAMS_LISTED)
    set(ICEORYX2_CXX_OPTIONS_AND_PARAMS_LISTED true)

    message(STATUS "[i] iceoryx2-cxx options:")

    add_option(
        NAME IOX2_DEPENDENCY_USE_SYSTEM_FLATBUFFERS
        DESCRIPTION "Use the flatbuffers provided by the system instead of using FetchContent"
        DEFAULT_VALUE OFF
    )

endif()
