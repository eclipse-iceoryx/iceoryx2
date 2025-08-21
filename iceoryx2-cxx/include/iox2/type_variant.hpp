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

#ifndef IOX2_TYPE_VARIANT_HPP
#define IOX2_TYPE_VARIANT_HPP

#include <cstdint>

namespace iox2 {
/// Defines if the type is a slice with a runtime-size
/// ([`TypeVariant::Dynamic`]) or if its a type that satisfies [`Sized`]
/// ([`TypeVariant::FixedSize`]).
enum class TypeVariant : uint8_t {
    /// A fixed size type like [`u64`]
    FixedSize,
    /// A dynamic sized type like a slice
    Dynamic,
};
} // namespace iox2

#endif
