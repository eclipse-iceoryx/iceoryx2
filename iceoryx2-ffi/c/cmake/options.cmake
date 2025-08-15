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

if(NOT ICEORYX2_C_OPTIONS_AND_PARAMS_LISTED)
    set(ICEORYX2_C_OPTIONS_AND_PARAMS_LISTED true)

    message(STATUS "[i] iceoryx2-c options:")

    add_param(
        NAME RUST_BUILD_ARTIFACT_PATH
        DESCRIPTION "The path to the folder with the Rust build artifacts"
        DEFAULT_VALUE ""
    )

endif()
