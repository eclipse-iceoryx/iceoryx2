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

#ifndef IOX2_EXAMPLES_CROSS_LANGUAGE_COMPLEX_TYPE_HPP
#define IOX2_EXAMPLES_CROSS_LANGUAGE_COMPLEX_TYPE_HPP

#include "iox2/bb/static_string.hpp"
#include "iox2/bb/static_vector.hpp"

#include <cstdint>

struct FullName {
    iox2::bb::StaticString<256> first_name; // NOLINT
    iox2::bb::StaticString<256> last_name;  // NOLINT
};

struct ComplexType {
    // IOX2_TYPE_NAME is equivalent to the payload type name used on the Rust side
    static constexpr const char* IOX2_TYPE_NAME = "ComplexType";

    iox2::bb::StaticVector<FullName, 16384> address_book;                     // NOLINT
    iox2::bb::StaticVector<iox2::bb::StaticVector<double, 8>, 8> some_matrix; // NOLINT
    uint16_t some_value;
    uint32_t another_value;
};

#endif
