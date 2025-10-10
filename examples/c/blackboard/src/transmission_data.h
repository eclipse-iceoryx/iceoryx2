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

#ifndef IOX2_EXAMPLES_TRANSMISSION_DATA_H
#define IOX2_EXAMPLES_TRANSMISSION_DATA_H

#include <stdbool.h>
#include <stdint.h>

struct Foo {
    uint32_t x;
    int64_t y;
    uint16_t z;
};

bool key_cmp(const uint8_t* lhs, const uint8_t* rhs) {
    const struct Foo* lhs_foo = (const struct Foo*) lhs; // NOLINT
    const struct Foo* rhs_foo = (const struct Foo*) rhs; // NOLINT
    return lhs_foo->x == rhs_foo->x && lhs_foo->y == rhs_foo->y && lhs_foo->z == rhs_foo->z;
}

#endif
