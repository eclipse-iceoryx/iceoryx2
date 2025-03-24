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

// TODOs:
// - cmake executable names
// - example name
// - copyright years
// - rust bazel, TODO
// - add readme to overall example readme
#include <cstdint>
#include <iostream>

struct TransmissionData {
    // TYPE_NAME is equivalent to the payload type name used on the Rust side and was determined with
    // `core::any::type_name::<TransmissionData>()`
    static constexpr const char* TYPE_NAME = "examples_common::transmission_data::TransmissionData";
    std::int32_t x;
    std::int32_t y;
    double funky;
};

inline auto operator<<(std::ostream& stream, const TransmissionData& value) -> std::ostream& {
    stream << "TransmissionData { x: " << value.x << ", y: " << value.y << ", funky: " << value.funky << " }";
    return stream;
}

struct CustomHeader {
    // TYPE_NAME is equivalent to the user header type name used on the Rust side and was determined with
    // `core::any::type_name::<CustomHeader>()`
    static constexpr const char* TYPE_NAME = "examples_common::custom_header::CustomHeader";
    int32_t version;
    uint64_t timestamp;
};

inline auto operator<<(std::ostream& stream, const CustomHeader& value) -> std::ostream& {
    stream << "CustomHeader { version: " << value.version << ", timestamp: " << value.timestamp << "}";
    return stream;
}

#endif
