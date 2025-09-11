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

#ifndef IOX2_INCLUDE_GUARD_CONTAINER_DETAIL_RAW_BYTE_STORAGE_HPP
#define IOX2_INCLUDE_GUARD_CONTAINER_DETAIL_RAW_BYTE_STORAGE_HPP

#include <cstdint>
#include <memory>
#include <new>
#include <type_traits>

namespace iox2 {
namespace container {
namespace detail {

/// A class for storing at most N objects of type T in a contiguous storage.
/// All operations on this class are unchecked.
template <typename T, uint64_t N>
class RawByteStorage {
    // NOLINTNEXTLINE(modernize-type-traits), _v requires C++17
    static_assert(std::is_standard_layout<T>::value, "Storage is only valid for standard layout types.");

  private:
    // NOLINTNEXTLINE(cppcoreguidelines-avoid-c-arrays,hicpp-avoid-c-arrays,modernize-avoid-c-arrays) raw storage, will not be used as array
    alignas(T) char m_bytes[sizeof(T) * N];
    uint64_t m_size;

  public:
    constexpr RawByteStorage() noexcept
        : m_bytes {}
        , m_size(0) {
    }

    constexpr RawByteStorage(RawByteStorage const& rhs)
        : m_bytes {}
        , m_size(rhs.m_size) {
        for (uint64_t index = 0; index < m_size; ++index) {
            new (pointer_from_index(index)) T(*rhs.pointer_from_index(index));
        }
    }

    constexpr RawByteStorage(RawByteStorage&& rhs) noexcept
        : m_bytes {}
        , m_size(rhs.m_size) {
        for (uint64_t index = 0; index < m_size; ++index) {
            new (pointer_from_index(index)) T(std::move_if_noexcept(*rhs.pointer_from_index(index)));
        }
    }

    template <uint64_t M, std::enable_if_t<(N > M), bool> = true>
    // NOLINTNEXTLINE(hicpp-explicit-conversions), conceptually a copy constructor
    constexpr RawByteStorage(RawByteStorage<T, M> const& rhs)
        : m_bytes {}
        , m_size(rhs.size()) {
        for (uint64_t index = 0; index < m_size; ++index) {
            new (pointer_from_index(index)) T(*rhs.pointer_from_index(index));
        }
    }

    template <uint64_t M, std::enable_if_t<(N > M), bool> = true>
    // NOLINTNEXTLINE(hicpp-explicit-conversions), conceptually a move constructor
    constexpr RawByteStorage(RawByteStorage<T, M>&& rhs)
        : m_bytes {}
        , m_size(rhs.size()) {
        for (uint64_t index = 0; index < m_size; ++index) {
            new (pointer_from_index(index)) T(std::move(*rhs.pointer_from_index(index)));
        }
    }

#if __cplusplus >= 202002L
    constexpr
#endif
        ~RawByteStorage() {
        for (uint64_t i = m_size; i != 0; --i) {
            uint64_t const index = i - 1;
            pointer_from_index(index)->~T();
        }
    }

    constexpr auto operator=(RawByteStorage const&) -> RawByteStorage& = delete;
    constexpr auto operator=(RawByteStorage&&) -> RawByteStorage& = delete;

    auto constexpr size() const noexcept -> uint64_t {
        return m_size;
    }

    // @pre size() < (N - 1)
    template <typename... Args>
    constexpr void emplace_back(Args&&... args) {
        new (pointer_from_index(size())) T(std::forward<Args>(args)...);
        ++m_size;
    }

    // @pre (size() < (N - 1)) && (index <= size())
    template <typename... Args>
    constexpr auto emplace_at(uint64_t index, Args&&... args) {
        make_room_at(index, 1);
        T& target = *pointer_from_index(index);
        target = T(std::forward<Args>(args)...);
    }

    // @pre (index <= size()) && (size() + gap_size < N)
    constexpr void make_room_at(uint64_t index, uint64_t gap_size) {
        uint64_t const old_size = m_size;
        // construct new elements at the back
        for (uint64_t i = 0; i < gap_size; ++i) {
            T& source = *pointer_from_index(m_size - gap_size);
            emplace_back(std::move(source));
        }
        // move remaining elements up
        for (uint64_t i = old_size; i != index; --i) {
            T& source = *pointer_from_index(i - 1);
            T& target = *pointer_from_index(i + gap_size - 1);
            target = std::move(source);
        }
    }

    // @pre (index < size())
    constexpr void erase_at(uint64_t index) {
        remove_at(index, 1);
        shrink_from_back(m_size - 1);
    }

    // @pre (index + range_size <= size())
    constexpr void remove_at(uint64_t index, uint64_t range_size) {
        for (uint64_t i = index; i < m_size - range_size; ++i) {
            T& source = *pointer_from_index(i + range_size);
            T& target = *pointer_from_index(i);
            target = std::move(source);
        }
    }

    // @pre target_size < size()
    constexpr void shrink_from_back(uint64_t target_size) {
        for (uint64_t i = m_size; i != target_size; --i) {
            uint64_t const index = i - 1;
            pointer_from_index(index)->~T();
        }
        m_size = target_size;
    }

    // @pre (idx >= 0) && (idx < size())
    auto pointer_from_index(uint64_t idx) -> T* {
        // NOLINTNEXTLINE(cppcoreguidelines-pro-type-reinterpret-cast), required for storage access
        return reinterpret_cast<T*>(m_bytes + (idx * sizeof(T)));
    }

    // @pre (idx >= 0) && (idx < size())
    auto pointer_from_index(uint64_t idx) const -> T const* {
        // NOLINTNEXTLINE(cppcoreguidelines-pro-type-reinterpret-cast), required for storage access
        return reinterpret_cast<T const*>(m_bytes + (idx * sizeof(T)));
    }
};
} // namespace detail
} // namespace container
} // namespace iox2

#endif
