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

include(Iceoryx2OptionAndParamMacros)

if(NOT ICEORYX2_COMMON_OPTIONS_AND_PARAMS_LISTED)
    set(ICEORYX2_COMMON_OPTIONS_AND_PARAMS_LISTED true)

    message(STATUS "[i] iceoryx2 common options and params:")

    add_option(
        NAME BUILD_TESTING
        DESCRIPTION "Build tests"
        DEFAULT_VALUE OFF
    )

    add_option(
        NAME SANITIZERS
        DESCRIPTION "Build with undefined-behavior- and address-sanitizer"
        DEFAULT_VALUE OFF
    )

    add_option(
        NAME WARNING_AS_ERROR
        DESCRIPTION "Fails if the compiler emits a warning"
        DEFAULT_VALUE OFF
    )

endif()
