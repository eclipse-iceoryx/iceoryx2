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
#ifndef IOX_SLICE_HPP_
#define IOX_SLICE_HPP_

#include <cstdint>

namespace iox {
template <typename T>
class Slice {
   public:
    using iterator = T*;
    using const_iterator = const T*;

    uint64_t size() const {}
    const T& operator[](const uint64_t n) const {}
    T& operator[](const uint64_t n) {}

    iterator begin() {}
    const_iterator begin() const {}
    iterator end() {}
    const_iterator end() const {}
};
}  // namespace iox

#endif
