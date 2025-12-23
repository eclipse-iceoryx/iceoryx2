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

if(NOT ICEORYX2_BB_CXX_OPTIONS_AND_PARAMS_LISTED)
    set(ICEORYX2_BB_CXX_OPTIONS_AND_PARAMS_LISTED true)

    message(STATUS "[i] iceoryx2-bb-cxx options:")

    add_option(
        NAME IOX2_BB_CXX_CONFIG_USE_STD_EXPECTED
        DESCRIPTION "Use the STL 'expected' instead of the iceoryx2 implementation"
        DEFAULT_VALUE OFF
    )

    add_option(
        NAME IOX2_BB_CXX_CONFIG_USE_STD_OPTIONAL
        DESCRIPTION "Use the STL 'optional' instead of the iceoryx2 implementation"
        DEFAULT_VALUE OFF
    )

    add_option(
        NAME IOX2_BB_CXX_CONFIG_USE_CUSTOM_VOCABULARY_TYPES
        DESCRIPTION "Use custom 'optional' and 'expected' instead of the iceoryx2 implementation"
        DEFAULT_VALUE OFF
    )

    add_param(
        NAME IOX2_BB_CXX_CONFIG_CUSTOM_VOCABULARY_TYPES_CMAKE_TARGET
        DESCRIPTION "CMake target for custom 'optional' and 'expected'"
        DEFAULT_VALUE ""
    )

endif()
