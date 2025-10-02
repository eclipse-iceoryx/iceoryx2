// Copyright (c) 2024 Contributors to the Eclipse Foundation
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

#ifndef IOX2_EXAMPLES_MESSAGE_DATA_HPP
#define IOX2_EXAMPLES_MESSAGE_DATA_HPP

#include <cstdint>
#include <iostream>

struct TransmissionData {
    // IOX2_TYPE_NAME is equivalent to the payload type name used on the Rust side
    static constexpr const char* IOX2_TYPE_NAME = "TransmissionData";
    std::int32_t x;
    std::int32_t y;
    double funky;
};

inline auto operator<<(std::ostream& stream, const TransmissionData& value) -> std::ostream& {
    stream << "TransmissionData { x: " << value.x << ", y: " << value.y << ", funky: " << value.funky << " }";
    return stream;
}

struct CustomHeader {
    // IOX2_TYPE_NAME is equivalent to the user header type name used on the Rust side
    static constexpr const char* IOX2_TYPE_NAME = "CustomHeader";
    int32_t version;
    uint64_t timestamp;
};

inline auto operator<<(std::ostream& stream, const CustomHeader& value) -> std::ostream& {
    stream << "CustomHeader { version: " << value.version << ", timestamp: " << value.timestamp << "}";
    return stream;
}

#endif
