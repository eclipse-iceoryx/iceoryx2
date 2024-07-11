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

#ifndef IOX2_MESSAGE_TYPE_DETAILS_HPP_
#define IOX2_MESSAGE_TYPE_DETAILS_HPP_

#include <cstdint>
#include <string>

namespace iox2 {
/// Defines if the type is a slice with a runtime-size
/// ([`TypeVariant::Dynamic`]) or if its a type that satisfies [`Sized`]
/// ([`TypeVariant::FixedSize`]).
enum class TypeVariant {
    /// A fixed size type like [`u64`]
    FixedSize,
    /// A dynamic sized type like a slice
    Dynamic,
};

/// Contains all type details required to connect to a
/// [`crate::service::Service`]
struct TypeDetail {
    /// The [`TypeVariant`] of the type
    TypeVariant variant;
    /// Contains the output of [`core::any::type_name()`].
    std::string type_name;
    /// The size of the underlying type.
    uint64_t size;
    /// The alignment of the underlying type.
    uint64_t alignment;
};

struct MessageTypeDetails {
    /// The [`TypeDetail`] of the header of a message, the first iceoryx2
    /// internal part.
    TypeDetail header;
    /// The [`TypeDetail`] of the user_header or the custom header, is located
    /// directly after the header.
    TypeDetail user_header;
    /// The [`TypeDetail`] of the payload of the message, the last part.
    TypeDetail payload;
};
} // namespace iox2

#endif
