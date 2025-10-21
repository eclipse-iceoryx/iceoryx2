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

#include <algorithm>
#include <cstdint>
#include <memory>
#include <new>
#include <type_traits>

namespace iox2 {
namespace container {
namespace detail {

/// A class for storing at most N objects of type T in a contiguous storage.
/// All operations on this class are unchecked.
template <typename T, uint64_t Capacity>
class RawByteStorage {
    // NOLINTNEXTLINE(modernize-type-traits), _v requires C++17
    static_assert(std::is_standard_layout<T>::value, "Storage is only valid for standard layout types.");

  private:
    // NOLINTNEXTLINE(cppcoreguidelines-avoid-c-arrays,hicpp-avoid-c-arrays,modernize-avoid-c-arrays) raw storage, will not be used as array
    alignas(T) char m_bytes[sizeof(T) * Capacity];
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

    template <uint64_t M, std::enable_if_t<(Capacity > M), bool> = true>
    // NOLINTNEXTLINE(hicpp-explicit-conversions), conceptually a copy constructor
    constexpr RawByteStorage(RawByteStorage<T, M> const& rhs)
        : m_bytes {}
        , m_size(rhs.size()) {
        for (uint64_t index = 0; index < m_size; ++index) {
            new (pointer_from_index(index)) T(*rhs.pointer_from_index(index));
        }
    }

    template <uint64_t M, std::enable_if_t<(Capacity > M), bool> = true>
    // NOLINTNEXTLINE(hicpp-explicit-conversions), conceptually a move constructor
    constexpr RawByteStorage(RawByteStorage<T, M>&& rhs) noexcept
        : m_bytes {}
        , m_size(rhs.size()) {
        for (uint64_t index = 0; index < m_size; ++index) {
            new (pointer_from_index(index)) T(std::move_if_noexcept(*rhs.pointer_from_index(index)));
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

    constexpr auto operator=(RawByteStorage const& rhs) -> RawByteStorage& {
        if (&rhs != this) {
            for (uint64_t i = 0; i < std::min(m_size, rhs.m_size); ++i) {
                *pointer_from_index(i) = *rhs.pointer_from_index(i);
            }
            for (uint64_t i = m_size; i < rhs.m_size; ++i) {
                //NOLINTNEXTLINE(clang-analyzer-cplusplus.PlacementNew) false positive in clang-tidy version 19 only
                new (pointer_from_index(i)) T(*rhs.pointer_from_index(i));
            }
            for (uint64_t i = rhs.m_size; i < m_size; ++i) {
                pointer_from_index(i)->~T();
            }
            m_size = rhs.m_size;
        }
        return *this;
    }

    constexpr auto operator=(RawByteStorage&& rhs) noexcept(std::is_nothrow_move_constructible_v<T>
                                                            && std::is_nothrow_move_assignable_v<T>)
        -> RawByteStorage& {
        if (&rhs != this) {
            for (uint64_t i = 0; i < std::min(m_size, rhs.m_size); ++i) {
                *pointer_from_index(i) = std::move(*rhs.pointer_from_index(i));
            }
            for (uint64_t i = m_size; i < rhs.m_size; ++i) {
                new (pointer_from_index(i)) T(std::move(*rhs.pointer_from_index(i)));
            }
            for (uint64_t i = rhs.m_size; i < m_size; ++i) {
                pointer_from_index(i)->~T();
            }
            m_size = rhs.m_size;
        }
        return *this;
    }

    auto constexpr size() const noexcept -> uint64_t {
        return m_size;
    }

    // @pre size() < Capacity
    template <typename... Args>
    constexpr void emplace_back(Args&&... args) {
        new (pointer_from_index(size())) T(std::forward<Args>(args)...);
        ++m_size;
    }

    // @pre (size() < Capacity) && (index <= size())
    template <typename... Args>
    constexpr void emplace_at(uint64_t index, Args&&... args) {
        emplace_back(std::forward<Args>(args)...);
        rotate_from_back(index, m_size - 1);
    }

    // @pre (index <= size()) && (size() + count <= Capacity)
    constexpr void insert_at(uint64_t index, uint64_t count, T const& value) {
        for (uint64_t i = 0; i < count; ++i) {
            emplace_back(value);
        }
        rotate_from_back(index, m_size - count);
    }

    // @pre (index < size())
    constexpr void erase_at(uint64_t index) {
        remove_at(index, 1);
        shrink_from_back(m_size - 1);
    }

    // @pre (end_index <= size()) && (begin_index <= end_index)
    constexpr void erase_at(uint64_t begin_index, uint64_t end_index) {
        uint64_t const range = end_index - begin_index;
        remove_at(begin_index, range);
        shrink_from_back(m_size - range);
    }

    // @pre (index + range_size <= size())
    constexpr void remove_at(uint64_t index, uint64_t range_size) {
// gcc 15 generates a false positive here, where range checks performed
// by outer functions are not taken into account correctly
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Warray-bounds"
        std::move(pointer_from_index(index + range_size), pointer_from_index(m_size), pointer_from_index(index));
#pragma GCC diagnostic pop
    }

    // @pre target_size < size()
    constexpr void shrink_from_back(uint64_t target_size) {
        for (uint64_t i = m_size; i != target_size; --i) {
            uint64_t const index = i - 1;
            pointer_from_index(index)->~T();
        }
        m_size = target_size;
    }

    // @pre (index_first_from < size()) && (index_to < index_first_from)
    constexpr void rotate_from_back(uint64_t index_to, uint64_t index_first_from) {
        std::rotate(pointer_from_index(index_to), pointer_from_index(index_first_from), pointer_from_index(m_size));
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

    // Memory layout metrics.
    // Contains all relevant metrics to verify the exact layout of the storage in memory.
    struct StorageMemoryLayoutMetrics {
        size_t storage_alignment;
        size_t storage_size;
        size_t sizeof_bytes;
        size_t offset_bytes;
        size_t sizeof_size;
        size_t offset_size;
        bool size_is_unsigned;
    };

    constexpr auto static_memory_layout_metrics() noexcept -> StorageMemoryLayoutMetrics {
        using Self = std::remove_reference_t<decltype(*this)>;
        StorageMemoryLayoutMetrics ret;
        ret.storage_alignment = alignof(Self);
        ret.storage_size = sizeof(Self);
        ret.sizeof_bytes = sizeof(m_bytes);
        ret.offset_bytes = offsetof(Self, m_bytes);
        ret.sizeof_size = sizeof(m_size);
        ret.offset_size = offsetof(Self, m_size);
        // NOLINTNEXTLINE(modernize-type-traits), _v requires C++17
        ret.size_is_unsigned = std::is_unsigned<decltype(m_size)>::value;
        return ret;
    }
};
} // namespace detail
} // namespace container
} // namespace iox2

#endif
