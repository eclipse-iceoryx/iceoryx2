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

# Parse SANITIZERS string and set appropriate flags
set(ICEORYX2_SANITIZER_FLAGS "" CACHE INTERNAL "")

if(NOT DEFINED SANITIZERS OR SANITIZERS STREQUAL "")
    # No sanitizers enabled - empty string is the default
elseif(SANITIZERS STREQUAL "address")
    set(ICEORYX2_SANITIZER_FLAGS -fsanitize=address CACHE INTERNAL "")
elseif(SANITIZERS STREQUAL "ub")
    set(ICEORYX2_SANITIZER_FLAGS -fsanitize=undefined CACHE INTERNAL "")
elseif(SANITIZERS STREQUAL "address;ub")
    set(ICEORYX2_SANITIZER_FLAGS -fsanitize=address -fsanitize=undefined CACHE INTERNAL "")
elseif(SANITIZERS STREQUAL "thread")
    set(ICEORYX2_SANITIZER_FLAGS -fsanitize=thread CACHE INTERNAL "")
else()
    message(FATAL_ERROR "Invalid SANITIZERS value: '${SANITIZERS}'. Valid options are: 'address', 'ub', 'address;ub', 'thread', or empty string (disabled)")
endif()
