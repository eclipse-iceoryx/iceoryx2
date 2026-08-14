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

#ifndef IOX2_MARKER_HPP
#define IOX2_MARKER_HPP

#include "iox2/iceoryx2_cxx_deployment.hpp"

#include <cstdint>

namespace iox2 {
#if IOX2_FEATURE_FLATBUFFERS
/// Identifies payloads that are serialized via flatbuffer.
template <typename T>
struct Flatbuffer {
    using ValueType = T;
    /// IOX2_TYPE_NAME is equivalent to the payload type name used on the Rust side
    static constexpr const char* IOX2_TYPE_NAME = "iox2::Flatbuffer";
};
#endif // IOX2_FEATURE_FLATBUFFERS

/// User header type for a service whose user header type details are set at runtime.
struct CustomHeaderMarker { };

/// Payload element for a service whose payload type details are set at runtime.
struct CustomPayloadMarker {
    uint8_t value = 0;
};


namespace internal {
template <typename Given, template <typename> class Required>
struct HasMarker {
    static const bool VALUE = false;
};

template <typename T, template <typename> class Required>
struct HasMarker<Required<T>, Required> {
    static const bool VALUE = true;
};
} // namespace internal

template <typename T>
constexpr auto has_flatbuffer_marker() -> bool {
#if IOX2_FEATURE_FLATBUFFERS
    return internal::HasMarker<T, Flatbuffer>::VALUE;
#else
    return false;
#endif // IOX2_FEATURE_FLATBUFFERS
}
} // namespace iox2

#endif
