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

#ifndef IOX_SLICE_HPP
#define IOX_SLICE_HPP

#include "iox/assertions_addendum.hpp"

#include <cstdint>

namespace iox {
template <typename T>
class Slice {
  public:
    using Iterator = T*;
    using ConstIterator = const T*;
    using ValueType = T;

    auto size() const -> uint64_t {
        IOX_TODO();
    }
    auto operator[](const uint64_t n) const -> const T& {
        IOX_TODO();
    }
    auto operator[](const uint64_t n) -> T& {
        IOX_TODO();
    }
    auto begin() -> Iterator {
        IOX_TODO();
    }
    auto begin() const -> ConstIterator {
        IOX_TODO();
    }
    auto end() -> Iterator {
        IOX_TODO();
    }
    auto end() const -> ConstIterator {
        IOX_TODO();
    }
};

template <typename>
struct IsSlice {
    static constexpr bool VALUE = false;
};

template <typename T>
struct IsSlice<Slice<T>> {
    static constexpr bool VALUE = true;
};
} // namespace iox

#endif
