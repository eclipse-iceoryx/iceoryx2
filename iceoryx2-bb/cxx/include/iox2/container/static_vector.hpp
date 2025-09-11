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

#include "iox2/container/detail/raw_byte_storage.hpp"

#include <cstddef>
#include <cstdint>
#include <cstring>
#include <functional>
#include <memory>
#include <type_traits>

namespace iox2 {
namespace container {

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

        constexpr auto begin() noexcept -> Iterator {
            return m_parent->m_storage.pointer_from_index(0);
        }

        constexpr auto end() noexcept -> Iterator {
            return m_parent->m_storage.pointer_from_index(this->m_parent->m_storage.size());
        }

        constexpr auto data() noexcept -> Pointer {
            return m_parent->m_storage.pointer_from_index(0);
        }
    };

  private:
    template <typename, uint64_t>
    friend class StaticVector;
    detail::RawByteStorage<T, N> m_storage;

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

    template <uint64_t M, std::enable_if_t<(N >= M), bool> = true>
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

    auto operator=(StaticVector const&) -> StaticVector& = delete;
    auto operator=(StaticVector&&) -> StaticVector& = delete;

    template <typename... Args>
    constexpr auto try_emplace_back(Args&&... args) ->
        // NOLINTNEXTLINE(modernize-type-traits), _v requires C++17
        std::enable_if_t<std::is_constructible<T, Args...>::value, bool> {
        if (m_storage.size() < N) {
            m_storage.emplace_back(std::forward<Args>(args)...);
            return true;
        } else {
            return false;
        }
    }

    constexpr auto try_push_back(T const& value) -> bool {
        return try_emplace_back(value);
    }

    constexpr auto try_push_back(T&& value) -> bool {
        return try_emplace_back(std::move(value));
    }

    constexpr auto try_pop_back() -> bool {
        if (m_storage.size() > 0) {
            m_storage.resize_from_back(m_storage.size() - 1);
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

    auto front_element() -> OptionalReference {
        if (!empty()) {
            return *m_storage.pointer_from_index(0);
        } else {
            return nullopt;
        }
    }

    auto front_element() const -> OptionalReference {
        if (!empty()) {
            return *m_storage.pointer_from_index(0);
        } else {
            return nullopt;
        }
    }

    auto back_element() -> OptionalReference {
        if (!empty()) {
            return *m_storage.pointer_from_index(size() - 1);
        } else {
            return nullopt;
        }
    }

    auto back_element() const -> OptionalConstReference {
        if (!empty()) {
            return *m_storage.pointer_from_index(size() - 1);
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

    friend auto operator==(StaticVector const& lhs, StaticVector const& rhs) -> bool {
        if (lhs.size() != rhs.size()) {
            return false;
        } else {
            auto const lhs_unchecked = lhs.unchecked_access();
            auto const rhs_unchecked = rhs.unchecked_access();
            auto const lhs_it_end = lhs_unchecked.end();
            auto lhs_it = lhs_unchecked.begin();
            auto rhs_it = rhs_unchecked.begin();
            while (lhs_it != lhs_it_end) {
                if (!(*lhs_it == *rhs_it)) {
                    return false;
                }
                ++lhs_it;
                ++rhs_it;
            }
            return true;
        }
    }
};

} // namespace container
} // namespace iox2

#endif
