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

#ifndef IOX2_INCLUDE_GUARD_BB_OPTIONAL_HPP
#define IOX2_INCLUDE_GUARD_BB_OPTIONAL_HPP

#include "iox2/bb/duration.hpp"
#include "iox2/bb/variation/optional_adaption.hpp"

#include <ostream>

namespace iox2 {
namespace bb {

template <typename T>
using Optional = iox2::bb::variation::Optional<T>;
using NulloptT = iox2::bb::variation::NulloptT;

constexpr NulloptT NULLOPT = iox2::bb::variation::NULLOPT;

} // namespace bb
} // namespace iox2

/// A custom stream operator implementation. The default operator<< cannot be used since the
/// optional can be exchanged with a customer based implementation that may already come with
/// a stream operator implementation.
template <typename T>
auto stream_operator(std::ostream& stream, const iox2::bb::Optional<T>& value) -> std::ostream& {
    stream << "Optional { ";
    if (value.has_value()) {
        stream << "value: " << value.value();
    } else {
        stream << "NULLOPT";
    }
    stream << " }";

    return stream;
}

#endif // IOX2_INCLUDE_GUARD_BB_OPTIONAL_HPP
