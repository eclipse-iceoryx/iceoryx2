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
#include <type_traits>

namespace iox {
template <typename T>
class Slice {
  public:
    using Iterator = T*;
    using ConstIterator = const T*;
    using ValueType = T;

    Slice(const T* data, uint64_t number_of_elements);

    auto size() const -> uint64_t;
    auto number_of_elements() const -> uint64_t;

    auto operator[](uint64_t n) const -> const T&;
    auto operator[](uint64_t n) -> T&;

    auto begin() -> Iterator;
    auto begin() const -> ConstIterator;
    auto end() -> Iterator;
    auto end() const -> ConstIterator;

    auto data() -> T*;
    auto data() const -> const T*;

  private:
    T* m_data;
    uint64_t m_number_of_elements;
};

template <typename T>
Slice<T>::Slice(const T* data, uint64_t number_of_elements)
    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-const-cast) constness protected by const class specification
    : m_data { const_cast<T*>(data) }
    , m_number_of_elements { number_of_elements } {
}

template <typename T>
auto Slice<T>::size() const -> uint64_t {
    return (sizeof(ValueType) * m_number_of_elements + alignof(ValueType) - 1) & ~(alignof(ValueType) - 1);
}

template <typename T>
auto Slice<T>::number_of_elements() const -> uint64_t {
    return m_number_of_elements;
}

template <typename T>
auto Slice<T>::operator[](const uint64_t n) const -> const T& {
    IOX_ASSERT(n < m_number_of_elements, "Index out of bounds");
    return *(m_data + n);
}

template <typename T>
auto Slice<T>::operator[](const uint64_t n) -> T& {
    IOX_ASSERT(n < m_number_of_elements, "Index out of bounds");
    return *(m_data + n);
}

template <typename T>
auto Slice<T>::begin() -> Iterator {
    return m_data;
}

template <typename T>
auto Slice<T>::begin() const -> ConstIterator {
    return m_data;
}

template <typename T>
auto Slice<T>::end() -> Iterator {
    static_assert(!std::is_same_v<T, void>, "Slice<void> is not allowed");
    return m_data + m_number_of_elements;
}

template <typename T>
auto Slice<T>::end() const -> ConstIterator {
    static_assert(!std::is_same_v<T, void>, "Slice<void> is not allowed");
    return m_data + m_number_of_elements;
}

template <typename T>
auto Slice<T>::data() -> T* {
    return m_data;
}

template <typename T>
auto Slice<T>::data() const -> const T* {
    return m_data;
}

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
