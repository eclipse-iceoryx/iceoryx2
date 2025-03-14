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

#ifndef IOX2_EXAMPLES_TRANSMISSION_DATA_HPP
#define IOX2_EXAMPLES_TRANSMISSION_DATA_HPP

// TODOs:
// - cmake executable names
// - example name
// - copyright years
// - custom user header in same example? adapt rust user header example to use transmission data
#include <cstdint>
#include <iostream>

struct TransmissionData {
    // PAYLOAD_TYPE_NAME is equivalent to the payload type name used on the Rust side and was determined with
    // `core::any::type_name::<TransmissionData>()`
    static constexpr const char* PAYLOAD_TYPE_NAME = "examples_common::transmission_data::TransmissionData";
    std::int32_t x;
    std::int32_t y;
    double funky;
};

inline auto operator<<(std::ostream& stream, const TransmissionData& value) -> std::ostream& {
    stream << "TransmissionData { x: " << value.x << ", y: " << value.y << ", funky: " << value.funky << " }";
    return stream;
}

#endif
