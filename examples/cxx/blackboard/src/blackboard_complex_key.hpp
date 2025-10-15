// Copyright (c) 2025 Contributors to the Eclipse Foundation
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

#ifndef IOX2_EXAMPLES_BLACKBOARD_COMPLEX_KEY_HPP
#define IOX2_EXAMPLES_BLACKBOARD_COMPLEX_KEY_HPP

#include <cstdint>

struct BlackboardKey {
    // IOX2_TYPE_NAME is equivalent to the key type name used on the Rust side
    static constexpr const char* IOX2_TYPE_NAME = "BlackboardKey";
    std::uint32_t x; // NOLINT
    std::int64_t y;  // NOLINT
    std::uint16_t z; // NOLINT

    auto operator==(const BlackboardKey& rhs) const -> bool {
        return x == rhs.x && y == rhs.y && z == rhs.z;
    }
};

#endif
