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

    Slice(T* data, uint64_t number_of_elements);
    Slice(const T* data, uint64_t number_of_elements);

    /// @brief Returns the size of the slice in bytes.
    /// @return The size of the slice in bytes.
    auto size() const -> uint64_t;

    /// @brief Returns the number of elements in the slice.
    /// @return The number of elements in the slice.
    auto number_of_elements() const -> uint64_t;

    /// @brief Accesses the element at the specified index (const version).
    /// @param[in] n The index of the element to access.
    /// @return A const reference to the element at the specified index.
    /// @pre The index must be less than the number of elements in the slice.
    auto operator[](uint64_t n) const -> const T&;

    /// @brief Accesses the element at the specified index (non-const version).
    /// @param[in] n The index of the element to access.
    /// @return A reference to the element at the specified index.
    /// @pre The index must be less than the number of elements in the slice.
    auto operator[](uint64_t n) -> T&;

    /// @brief Returns an iterator to the beginning of the slice.
    /// @return An iterator pointing to the first element of the slice.
    auto begin() -> Iterator;

    /// @brief Returns a const iterator to the beginning of the slice.
    /// @return A const iterator pointing to the first element of the slice.
    auto begin() const -> ConstIterator;

    /// @brief Returns an iterator to the end of the slice.
    /// @return An iterator pointing one past the last element of the slice.
    auto end() -> Iterator;

    /// @brief Returns a const iterator to the end of the slice.
    /// @return A const iterator pointing one past the last element of the slice.
    auto end() const -> ConstIterator;

    /// @brief Returns a pointer to the underlying data of the slice.
    /// @return A pointer to the first element of the slice.
    auto data() -> T*;

    /// @brief Returns a const pointer to the underlying data of the slice.
    /// @return A const pointer to the first element of the slice.
    auto data() const -> const T*;

  private:
    T* m_data;
    uint64_t m_number_of_elements;
};

template <typename T>
Slice<T>::Slice(T* data, uint64_t number_of_elements)
    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-const-cast) constness protected by const class specification
    : m_data { data }
    , m_number_of_elements { number_of_elements } {
}

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
