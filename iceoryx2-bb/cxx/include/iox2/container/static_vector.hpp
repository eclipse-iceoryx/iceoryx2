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

#ifndef IOX2_INCLUDE_GUARD_CONTAINER_STATIC_VECTOR_HPP
#define IOX2_INCLUDE_GUARD_CONTAINER_STATIC_VECTOR_HPP

#include "iox2/container/optional.hpp"

#include "iox2/container/detail/raw_byte_storage.hpp"

#include <algorithm>
#include <cstddef>
#include <cstdint>
#include <cstring>
#include <functional>
#include <initializer_list>
#include <ostream>
#include <type_traits>

namespace iox2 {
namespace container {

/// A resizable container with compile-time fixed static capacity and contiguous inplace storage.
template <typename T, uint64_t Capacity>
class StaticVector {
    static_assert(Capacity > 0, "Static container with capacity 0 is not allowed.");
    // NOLINTNEXTLINE(modernize-type-traits), _v requires C++17
    static_assert(std::is_standard_layout<T>::value, "Containers can only be used with standard layout types.");

  public:
    using ValueType = T;
    using SizeType = size_t;
    using DifferenceType = ptrdiff_t;
    using Reference = T&;
    using ConstReference = T const&;
    using Pointer = T*;
    using ConstPointer = T const*;
    using Iterator = Pointer;
    using ConstIterator = ConstPointer;
    using OptionalReference = Optional<std::reference_wrapper<T>>;
    using OptionalConstReference = Optional<std::reference_wrapper<T const>>;

    /// Unchecked element access.
    /// Users of this class must ensure that all memory accesses stay within bounds of the accessed vector's memory.
    class UncheckedConstAccessor {
        friend class StaticVector;

      private:
        StaticVector const* m_parent;

        constexpr explicit UncheckedConstAccessor(StaticVector const& parent)
            : m_parent(&parent) {
        }

      public:
        ~UncheckedConstAccessor() = default;
        UncheckedConstAccessor(UncheckedConstAccessor const&) = delete;
        UncheckedConstAccessor(UncheckedConstAccessor&&) = delete;
        auto operator=(UncheckedConstAccessor const&) -> UncheckedConstAccessor& = delete;
        auto operator=(UncheckedConstAccessor&&) -> UncheckedConstAccessor& = delete;

        constexpr auto operator[](SizeType index) const&& -> ConstReference {
            return *m_parent->m_storage.pointer_from_index(index);
        }

        constexpr auto begin() const&& noexcept -> ConstIterator {
            return m_parent->m_storage.pointer_from_index(0);
        }

        constexpr auto end() const&& noexcept -> ConstIterator {
            return m_parent->m_storage.pointer_from_index(m_parent->m_storage.size());
        }

        constexpr auto data() const&& noexcept -> ConstPointer {
            return m_parent->m_storage.pointer_from_index(0);
        }
    };

    // Mutable unchecked element access.
    /// Users of this class must ensure that all memory accesses stay within bounds of the accessed vector's memory.
    class UncheckedAccessor {
        friend class StaticVector;

      private:
        StaticVector* m_parent;

        constexpr explicit UncheckedAccessor(StaticVector& parent)
            : m_parent(&parent) {
        }

      public:
        ~UncheckedAccessor() = default;
        UncheckedAccessor(UncheckedAccessor const&) = delete;
        UncheckedAccessor(UncheckedAccessor&&) = delete;
        auto operator=(UncheckedAccessor const&) -> UncheckedAccessor& = delete;
        auto operator=(UncheckedAccessor&&) -> UncheckedAccessor& = delete;

        constexpr auto operator[](SizeType index) && -> Reference {
            return *m_parent->m_storage.pointer_from_index(index);
        }

        constexpr auto begin() && noexcept -> Iterator {
            return m_parent->m_storage.pointer_from_index(0);
        }

        constexpr auto end() && noexcept -> Iterator {
            return m_parent->m_storage.pointer_from_index(this->m_parent->m_storage.size());
        }

        constexpr auto data() && noexcept -> Pointer {
            return m_parent->m_storage.pointer_from_index(0);
        }
    };

  private:
    template <typename, uint64_t>
    friend class StaticVector;
    using StorageType = detail::RawByteStorage<T, Capacity>;
    StorageType m_storage;

  public:
    // constructors
    constexpr StaticVector() noexcept = default;
    constexpr StaticVector(StaticVector const&) = default;
    constexpr StaticVector(StaticVector&&) = default;

    /// Copy construct from a vector with smaller capacity
    template <uint64_t M, std::enable_if_t<(Capacity >= M), bool> = true>
    // NOLINTNEXTLINE(hicpp-explicit-conversions), conceptually a copy constructor
    constexpr StaticVector(StaticVector<T, M> const& rhs)
        : m_storage(rhs.m_storage) {
    }

    /// Construct from a C-style array
    template <uint64_t M, std::enable_if_t<(Capacity >= M), bool> = true>
    // NOLINTNEXTLINE(cppcoreguidelines-avoid-c-arrays,hicpp-avoid-c-arrays,modernize-avoid-c-arrays), static bounds
    constexpr explicit StaticVector(T const (&element_array)[M]) {
        for (auto& element : element_array) {
            try_push_back(element);
        }
    }

    // destructor
#if __cplusplus >= 202002L
    constexpr
#endif
        ~StaticVector() = default;

    // assignment
    auto operator=(StaticVector const&) -> StaticVector& = default;
    auto operator=(StaticVector&&) -> StaticVector& = default;

    /// Construct a new vector with `count` occurrences of a default constructed value.
    /// @return Nullopt if `count` exceeds the vector capacity.
    ///         Otherwise a vector containing the desired elements.
    static constexpr auto from_value(SizeType count)
        // NOLINTNEXTLINE(modernize-type-traits), _v requires C++17
        -> std::enable_if_t<std::is_default_constructible<T>::value, Optional<StaticVector>> {
        if (count <= Capacity) {
            return from_value(count, T {});
        } else {
            return nullopt;
        }
    }

    /// Construct a new vector with `count` copies of `value`.
    static constexpr auto from_value(SizeType count, T const& value) -> Optional<StaticVector> {
        // we define ret here to encourage return-value-optimization
        Optional<StaticVector> ret;
        if (count <= Capacity) {
            ret = StaticVector {};
            ret->m_storage.insert_at(0, count, value);
        } else {
            ret = nullopt;
        }
        return ret;
    }

    /// Construct a vector from a range [`it_begin`, `it_end`).
    /// Users must ensure that `it_end` is reachable from `it_begin` without causing undefined behaviour.
    /// @return Nullopt if the range size exceeds the vector capacity.
    ///         Otherwise a vector containing copies of the range elements.
    template <typename Iter,
              typename Sentinel,
              std::enable_if_t<
                  // NOLINTNEXTLINE(modernize-type-traits), _v requires C++17
                  std::is_constructible<T, decltype(*std::declval<Iter>())>::value
                      // NOLINTNEXTLINE(modernize-type-traits), _v requires C++17
                      && std::is_convertible<decltype(std::declval<Iter>() == std::declval<Sentinel>()), bool>::value,
                  bool> = true>
    static constexpr auto from_range_unchecked(Iter it_begin, Sentinel it_end) -> Optional<StaticVector> {
        // we define ret here to encourage return-value-optimization
        Optional<StaticVector> ret = StaticVector {};
        for (auto it = it_begin; it != it_end; ++it) {
            if (!ret->try_push_back(*it)) {
                ret = nullopt;
                break;
            }
        }
        return ret;
    }

    /// Constructs a vector from a range [`begin(rng)`, `end(rng)`).
    /// Users must ensure that `rng` is represents a valid range object.
    /// @return Nullopt if the range size exceeds the vector capacity.
    ///         Otherwise a vector containing copies of the range elements.
    template <typename Range>
    static constexpr auto from_range_unchecked(Range const& rng) -> Optional<StaticVector> {
        using std::begin;
        using std::end;
        return from_range_unchecked(begin(rng), end(rng));
    }


    /// Constructs a vector from the elements of the initializer list `init_list`.
    /// @return Nullopt if the initializer list size exceeds the vector capacity.
    ///         Otherwise a vector containing copies of the list elements.
    static constexpr auto from_initializer_list(std::initializer_list<T> init_list) -> Optional<StaticVector> {
        if (init_list.size() > Capacity) {
            return nullopt;
        } else {
            return from_range_unchecked(begin(init_list), end(init_list));
        }
    }

    /// Attempts to construct a new element from the constructor arguments `args` at the back of the vector.
    /// @return true on success.
    ///         false if the operation would exceed the vector's capacity.
    template <typename... Args>
    constexpr auto try_emplace_back(Args&&... args) ->
        // NOLINTNEXTLINE(modernize-type-traits), _v requires C++17
        std::enable_if_t<std::is_constructible<T, Args...>::value, bool> {
        if (m_storage.size() < Capacity) {
            m_storage.emplace_back(std::forward<Args>(args)...);
            return true;
        } else {
            return false;
        }
    }

    /// Attempts to construct a new element from the constructor arguments `args` at the specified `index`.
    /// @return true on success.
    ///         false if `index` is greater than the current size of the vector or
    ///         if the operation would exceed the vector's capacity.
    template <typename... Args>
    constexpr auto try_emplace_at(SizeType index, Args&&... args) ->
        // NOLINTNEXTLINE(modernize-type-traits), _v requires C++17
        std::enable_if_t<std::is_constructible<T, Args...>::value, bool> {
        if ((m_storage.size() < Capacity) && (index <= m_storage.size())) {
            m_storage.emplace_at(index, std::forward<Args>(args)...);
            return true;
        } else {
            return false;
        }
    }

    /// Attempts to erase the element at the specified `index`.
    /// @return true on success.
    ///         false if `index` is not the index of an existing element.
    constexpr auto try_erase_at(SizeType index) -> bool {
        if (index < m_storage.size()) {
            m_storage.erase_at(index);
            return true;
        } else {
            return false;
        }
    }

    /// Attempts to erase all elements in the index range [`begin_index`, `end_index`).
    /// @return true on success.
    ///         false if the index range is not a valid range of element indices.
    constexpr auto try_erase_at(SizeType begin_index, SizeType end_index) -> bool {
        if ((end_index <= m_storage.size()) && (begin_index <= end_index)) {
            m_storage.erase_at(begin_index, end_index);
            return true;
        } else {
            return false;
        }
    }

    /// Attempts to insert a single `value` at `index`.
    /// This function will copy the input value into place.
    /// @return true on success.
    ///         false if `index` is greater than the current size of the vector.
    constexpr auto try_insert_at(SizeType index, T const& value) -> bool {
        return try_emplace_at(index, value);
    }

    /// Attempts to insert a single `value` at `index`.
    /// This function will move the input value into place.
    /// @return true on success.
    ///         false if `index` is greater than the current size of the vector.
    constexpr auto try_insert_at(SizeType index, T&& value) -> bool {
        return try_emplace_at(index, std::move(value));
    }

    /// Attempts to insert `count` copies of `value` at `index`.
    /// @return true on success.
    ///         false if `index` is greater than the current size of the vector or
    ///         if the operation would exceed the vector's capacity.
    constexpr auto try_insert_at(SizeType index, SizeType count, T const& value) -> bool {
        if ((index <= m_storage.size()) && (m_storage.size() + count <= Capacity)) {
            m_storage.insert_at(index, count, value);
            return true;
        } else {
            return false;
        }
    }

    /// Attempts to insert the elements from the range [`it_begin`, `it_end`) at `index`.
    /// Users must ensure that `it_end` is reachable from `it_begin` without causing undefined behaviour.
    /// @return true on success.
    ///         false if `index` is greater than the current size of the vector or
    ///         if the operation would exceed the vector's capacity.
    template <typename Iter,
              typename Sentinel,
              std::enable_if_t<
                  // NOLINTNEXTLINE(modernize-type-traits), _v requires C++17
                  std::is_constructible<T, decltype(*std::declval<Iter>())>::value
                      // NOLINTNEXTLINE(modernize-type-traits), _v requires C++17
                      && std::is_convertible<decltype(std::declval<Iter>() == std::declval<Sentinel>()), bool>::value,
                  bool> = true>
    constexpr auto try_insert_at_unchecked(SizeType index, Iter it_begin, Sentinel it_end) -> bool {
        if (index <= m_storage.size()) {
            auto const old_size = size();
            for (auto it = it_begin; it != it_end; ++it) {
                if (!try_push_back(*it)) {
                    m_storage.shrink_from_back(old_size);
                    return false;
                }
            }
            m_storage.rotate_from_back(index, old_size);
            return true;
        } else {
            return false;
        }
    }

    /// Attempts to insert the elements from the initializer list `init_list` at `index`.
    /// @return true on success.
    ///         false if `index` is greater than the current size of the vector or
    ///         if the operation would exceed the vector's capacity.
    constexpr auto try_insert_at_unchecked(SizeType index, std::initializer_list<T> init_list) {
        return try_insert_at_unchecked(index, init_list.begin(), init_list.end());
    }

    /// Clears all elements from the vector.
    /// After this operation, the vector will be empty.
    constexpr void clear() {
        m_storage.erase_at(0, m_storage.size());
    }

    /// Attempts to insert a single `value` at the back of the vector.
    /// This function will copy the input value into place.
    /// @return true on success.
    ///         false if `index` is greater than the current size of the vector.
    constexpr auto try_push_back(T const& value) -> bool {
        return try_emplace_back(value);
    }

    /// Attempts to insert a single `value` at the back of the vector.
    /// This function will move the input value into place.
    /// @return true on success.
    ///         false if `index` is greater than the current size of the vector.
    constexpr auto try_push_back(T&& value) -> bool {
        return try_emplace_back(std::move(value));
    }

    /// Attempts to remove a single value from the back of the vector.
    /// @return true on success.
    ///         false if the vector is empty.
    constexpr auto try_pop_back() -> bool {
        if (m_storage.size() > 0) {
            m_storage.shrink_from_back(m_storage.size() - 1);
            return true;
        } else {
            return false;
        }
    }

    /// Retrieves the static capacity of the vector.
    static constexpr auto capacity() noexcept -> SizeType {
        return Capacity;
    }

    /// Retrieves the size of the vector.
    constexpr auto size() const noexcept -> SizeType {
        return m_storage.size();
    }

    /// Checks whether the vector is currently empty.
    constexpr auto empty() const -> bool {
        return size() == 0;
    }

    /// Attempts to retrieve the element at `index`.
    /// @return Nullopt if `index` is not 0 <= `index` < size().
    ///         Otherwise a reference to the element at the requested index.
    auto element_at(SizeType index) -> OptionalReference {
        if (index < m_storage.size()) {
            return *m_storage.pointer_from_index(index);
        } else {
            return nullopt;
        }
    }

    /// Attempts to retrieve the element at `index`.
    /// @return Nullopt if `index` is not 0 <= `index` < size().
    ///         Otherwise a reference to the element at the requested index.
    auto element_at(SizeType index) const -> OptionalConstReference {
        if (index < m_storage.size()) {
            return *m_storage.pointer_from_index(index);
        } else {
            return nullopt;
        }
    }

    /// Attempts to retrieve the first element.
    /// @return Nullopt if size() == 0.
    ///         Otherwise a reference to the first element.
    auto front_element() -> OptionalReference {
        if (!empty()) {
            return *m_storage.pointer_from_index(0);
        } else {
            return nullopt;
        }
    }

    /// Attempts to retrieve the first element.
    /// @return Nullopt if size() == 0.
    ///         Otherwise a reference to the first element.
    auto front_element() const -> OptionalConstReference {
        if (!empty()) {
            return *m_storage.pointer_from_index(0);
        } else {
            return nullopt;
        }
    }

    /// Attempts to retrieve the last element.
    /// @return Nullopt if size() == 0.
    ///         Otherwise a reference to the last element.
    auto back_element() -> OptionalReference {
        if (!empty()) {
            return *m_storage.pointer_from_index(size() - 1);
        } else {
            return nullopt;
        }
    }

    /// Attempts to retrieve the last element.
    /// @return Nullopt if size() == 0.
    ///         Otherwise a reference to the last element.
    auto back_element() const -> OptionalConstReference {
        if (!empty()) {
            return *m_storage.pointer_from_index(size() - 1);
        } else {
            return nullopt;
        }
    }

    /// Unchecked mutable access to the vector contents.
    auto unchecked_access() -> UncheckedAccessor {
        return UncheckedAccessor { *this };
    }

    /// Unchecked immutable access to the vector contents.
    auto unchecked_access() const -> UncheckedConstAccessor {
        return UncheckedConstAccessor { *this };
    }

    // comparison operators
    friend auto operator==(StaticVector const& lhs, StaticVector const& rhs) -> bool {
        return std::equal(lhs.unchecked_access().begin(),
                          lhs.unchecked_access().end(),
                          rhs.unchecked_access().begin(),
                          rhs.unchecked_access().end());
    }

    friend auto operator!=(StaticVector const& lhs, StaticVector const& rhs) -> bool {
        return !(lhs == rhs);
    }

    /// Obtains metrics about the internal memory layout of the vector.
    /// This function is intended for internal use only.
    constexpr auto static_memory_layout_metrics() noexcept {
        struct VectorMemoryLayoutMetrics {
            size_t vector_alignment;
            size_t vector_size;
            typename StorageType::StorageMemoryLayoutMetrics storage_metrics;
        } ret;
        using Self = std::remove_reference_t<decltype(*this)>;
        ret.vector_size = sizeof(Self);
        ret.vector_alignment = alignof(Self);
        ret.storage_metrics = m_storage.static_memory_layout_metrics();
        return ret;
    }
};

template <typename>
struct IsStaticVector : std::false_type { };

template <typename T, uint64_t N>
struct IsStaticVector<StaticVector<T, N>> : std::true_type { };

} // namespace container
} // namespace iox2

template <typename T, uint64_t N>
auto operator<<(std::ostream& stream, const iox2::container::StaticVector<T, N>& value) -> std::ostream& {
    stream << "StaticVector::<" << N << "> { m_size: " << value.size() << ", m_data: [ ";
    if (!value.empty()) {
        stream << value.unchecked_access()[0];
    }
    for (uint64_t idx = 1; idx < value.size(); ++idx) {
        stream << ", " << value.unchecked_access()[idx];
    }
    stream << " ] }";
    return stream;
}

#endif
