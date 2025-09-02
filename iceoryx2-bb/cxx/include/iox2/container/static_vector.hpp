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

#include "iox2/container/config.hpp"
#include "iox2/container/optional.hpp"

#include <cstddef>
#include <cstdint>
#include <cstring>
#include <functional>
#include <memory>
#include <type_traits>

namespace iox2 {
namespace container {

namespace detail {
template <typename T, uint64_t N>
class StaticVectorStorage {
    // NOLINTNEXTLINE(modernize-type-traits), _v requires C++17
    static_assert(std::is_standard_layout<T>::value, "Storage is only valid for standard layout types.");

  private:
    // NOLINTNEXTLINE(cppcoreguidelines-avoid-c-arrays,hicpp-avoid-c-arrays,modernize-avoid-c-arrays) raw storage, will not be used as array
    alignas(T) char m_bytes[sizeof(T) * N];
    uint64_t m_size;

  public:
    constexpr StaticVectorStorage() noexcept
        : m_bytes {}
        , m_size(0) {
    }

    constexpr StaticVectorStorage(StaticVectorStorage const& rhs)
        : m_bytes {}
        , m_size(rhs.m_size) {
        for (uint64_t index = 0; index < m_size; ++index) {
            new (pointer_from_index(index)) T(*rhs.pointer_from_index(index));
        }
    }

    constexpr StaticVectorStorage(StaticVectorStorage&& rhs) noexcept
        : m_bytes {}
        , m_size(rhs.m_size) {
        for (uint64_t index = 0; index < m_size; ++index) {
            new (pointer_from_index(index)) T(std::move_if_noexcept(*rhs.pointer_from_index(index)));
        }
    }

    template <uint64_t M, std::enable_if_t<(N > M), bool> = true>
    // NOLINTNEXTLINE(hicpp-explicit-conversions), conceptually a copy constructor
    constexpr StaticVectorStorage(StaticVectorStorage<T, M> const& rhs)
        : m_bytes {}
        , m_size(rhs.size()) {
        for (uint64_t index = 0; index < m_size; ++index) {
            new (pointer_from_index(index)) T(*rhs.pointer_from_index(index));
        }
    }

    template <uint64_t M, std::enable_if_t<(N > M), bool> = true>
    // NOLINTNEXTLINE(hicpp-explicit-conversions), conceptually a move constructor
    constexpr StaticVectorStorage(StaticVectorStorage<T, M>&& rhs)
        : m_bytes {}
        , m_size(rhs.size()) {
        for (uint64_t index = 0; index < m_size; ++index) {
            new (pointer_from_index(index)) T(std::move(*rhs.pointer_from_index(index)));
        }
    }

#if __cplusplus >= 202002L
    constexpr
#endif
        ~StaticVectorStorage() {
        for (uint64_t i = m_size; i != 0; --i) {
            uint64_t const index = i - 1;
            pointer_from_index(index)->~T();
        }
    }

    constexpr auto operator=(StaticVectorStorage const&) -> StaticVectorStorage& = delete;
    constexpr auto operator=(StaticVectorStorage&&) -> StaticVectorStorage& = delete;

    auto constexpr size() const noexcept -> uint64_t {
        return m_size;
    }

    constexpr void increment_size() {
        ++m_size;
    }

    auto pointer_from_index(uint64_t idx) -> T* {
        // NOLINTNEXTLINE(cppcoreguidelines-pro-type-reinterpret-cast), required for storage access
        return reinterpret_cast<T*>(m_bytes + (idx * sizeof(T)));
    }

    auto pointer_from_index(uint64_t idx) const -> T const* {
        // NOLINTNEXTLINE(cppcoreguidelines-pro-type-reinterpret-cast), required for storage access
        return reinterpret_cast<T const*>(m_bytes + (idx * sizeof(T)));
    }
};
} // namespace detail

/// A resizable container with compile-time fixed static capacity and contiguous inplace storage.
template <typename T, uint64_t N>
class StaticVector {
    static_assert(N > 0, "Static container with capacity 0 is not allowed.");
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

    // Unchecked element access
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

        constexpr auto operator[](SizeType index) const -> ConstReference {
            return *m_parent->m_storage.pointer_from_index(index);
        }

        constexpr auto begin() const noexcept -> ConstIterator {
            return m_parent->m_storage.pointer_from_index(0);
        }

        constexpr auto end() const noexcept -> ConstIterator {
            return m_parent->m_storage.pointer_from_index(m_parent->m_storage.size);
        }

        constexpr auto data() const noexcept -> ConstPointer {
            return m_parent->m_storage.pointer_from_index(0);
        }
    };

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

        constexpr auto operator[](SizeType index) -> Reference {
            return *m_parent->m_storage.pointer_from_index(index);
        }

        constexpr auto operator[](SizeType index) const -> ConstReference {
            return *m_parent->m_storage.pointer_from_index(index);
        }

        constexpr auto begin() noexcept -> Iterator {
            return m_parent->m_storage.pointer_from_index(0);
        }

        constexpr auto begin() const noexcept -> ConstIterator {
            return m_parent->m_storage.pointer_from_index(0);
        }

        constexpr auto end() noexcept -> Iterator {
            return m_parent->m_storage.pointer_from_index(this->m_parent->m_storage.size());
        }

        constexpr auto end() const noexcept -> ConstIterator {
            return m_parent->m_storage.pointer_from_index(this->m_parent->m_storage.size());
        }

        constexpr auto data() noexcept -> Pointer {
            return m_parent->m_storage.pointer_from_index(0);
        }

        constexpr auto data() const noexcept -> ConstPointer {
            return m_parent->m_storage.pointer_from_index(0);
        }
    };

  private:
    template <typename, uint64_t>
    friend class StaticVector;
    detail::StaticVectorStorage<T, N> m_storage;

  public:
    // constructors
    constexpr StaticVector() noexcept = default;
    constexpr StaticVector(StaticVector const&) = default;
    constexpr StaticVector(StaticVector&&) = default;

    template <uint64_t M, std::enable_if_t<(N >= M), bool> = true>
    // NOLINTNEXTLINE(hicpp-explicit-conversions), conceptually a copy constructor
    constexpr StaticVector(StaticVector<T, M> const& rhs)
        : m_storage(rhs.m_storage) {
    }

    // destructor
#if __cplusplus >= 202002L
    constexpr
#endif
        ~StaticVector() = default;

    auto operator=(StaticVector const&) -> StaticVector& = delete;
    auto operator=(StaticVector&&) -> StaticVector& = delete;

    template <typename... Args>
    constexpr auto try_emplace_back(Args&&... args) ->
        // NOLINTNEXTLINE(modernize-type-traits), _v requires C++17
        std::enable_if_t<std::is_constructible<T, Args...>::value, bool> {
        if (m_storage.size() < N) {
            new (m_storage.pointer_from_index(m_storage.size)) T(std::forward<Args>(args)...);
            m_storage.increment_size();
            return true;
        } else {
            return false;
        }
    }

    constexpr auto try_push_back(T const& value) -> bool {
        if (m_storage.size() < N) {
            new (m_storage.pointer_from_index(m_storage.size())) T(value);
            m_storage.increment_size();
            return true;
        } else {
            return false;
        }
    }

    constexpr auto try_push_back(T&& value) -> bool {
        if (m_storage.size() < N) {
            new (m_storage.pointer_from_index(m_storage.size())) T(std::move(value));
            m_storage.increment_size();
            return true;
        } else {
            return false;
        }
    }

    static constexpr auto capacity() noexcept -> SizeType {
        return N;
    }

    constexpr auto size() const noexcept -> SizeType {
        return m_storage.size();
    }

    constexpr auto empty() const -> bool {
        return size() == 0;
    }

    auto element_at(SizeType index) -> OptionalReference {
        if (index < m_storage.size()) {
            return *m_storage.pointer_from_index(index);
        } else {
            return nullopt;
        }
    }

    auto element_at(SizeType index) const -> OptionalConstReference {
        if (index < m_storage.size()) {
            return *m_storage.pointer_from_index(index);
        } else {
            return nullopt;
        }
    }

    auto unchecked_access() -> UncheckedAccessor {
        return UncheckedAccessor { *this };
    }

    auto unchecked_access() const -> UncheckedConstAccessor {
        return UncheckedConstAccessor { *this };
    }
};

} // namespace container
} // namespace iox2

#endif
