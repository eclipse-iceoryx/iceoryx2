// Copyright (c) 2026 Contributors to the Eclipse Foundation
//
// See the NOTICE file(s) distributed with this work for additional
// information regarding copyright ownership.
//
// This program and the accompanying materials are made available under the
// terms of the Apache Software License 2.0 which is available at
// https://www.apache.org/licenses/LICENSE-2.0, or the MIT license
// which is available at https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

#ifndef IOX2_EXAMPLES_REQUEST_RESPONSE_WITH_EVENTS_REQUEST_HEADER_HPP
#define IOX2_EXAMPLES_REQUEST_RESPONSE_WITH_EVENTS_REQUEST_HEADER_HPP

#include <cstdint>

struct RequestResponseWithEventsHeader {
    static constexpr const char* IOX2_TYPE_NAME = "RequestResponseWithEventsHeader";
    // The native-endian bytes of the client's 128-bit UniqueListenerId.
    uint8_t listener_id[16]; // NOLINT
};

static_assert(sizeof(RequestResponseWithEventsHeader) == 16 && alignof(RequestResponseWithEventsHeader) == 1,
              "Request header must match Python's layout");

#endif
