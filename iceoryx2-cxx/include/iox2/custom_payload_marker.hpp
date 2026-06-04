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

#ifndef IOX2_CUSTOM_PAYLOAD_MARKER_HPP
#define IOX2_CUSTOM_PAYLOAD_MARKER_HPP

#include <cstdint>

namespace iox2 {

/// Payload element for a service whose payload type details are set at runtime.
struct CustomPayloadMarker {
    uint8_t value;
};

} // namespace iox2

#endif
