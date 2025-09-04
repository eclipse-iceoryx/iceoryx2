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

#ifndef INCLUDE_GUARD_IOX2_CONTAINER_DETAIL_RAW_BYTE_STORAGE_HPP
#define INCLUDE_GUARD_IOX2_CONTAINER_DETAIL_RAW_BYTE_STORAGE_HPP

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
    constexpr void increment_size() {
        ++m_size;
    }

    // @pre size() > 0
    constexpr void decrement_size() {
        --m_size;
    }

    // @pre (target_size >= 0) && (target_size < N)
    constexpr void adjust_size_to(uint64_t target_size) {
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
