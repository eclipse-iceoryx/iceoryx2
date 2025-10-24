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

#ifndef IOX2_EXAMPLES_BLACKBOARD_COMPLEX_KEY_H
#define IOX2_EXAMPLES_BLACKBOARD_COMPLEX_KEY_H

#include <stdbool.h>
#include <stdint.h>

// IOX2_KEY_TYPE_NAME is equivalent to the key type name used on the Rust side
// NOLINTNEXTLINE, C idiomatic way for compile time const variables
#define IOX2_KEY_TYPE_NAME "BlackboardKey"
struct BlackboardKey {
    uint32_t x;
    int64_t y;
    uint16_t z;
};

// To store and retrieve a key in the blackboard, a comparison function must be
// provided.
bool key_cmp(const void* lhs, const void* rhs) {
    const struct BlackboardKey LHS = *((const struct BlackboardKey*) lhs);
    const struct BlackboardKey RHS = *((const struct BlackboardKey*) rhs);
    return LHS.x == RHS.x && LHS.y == RHS.y && LHS.z == RHS.z;
}

#endif
