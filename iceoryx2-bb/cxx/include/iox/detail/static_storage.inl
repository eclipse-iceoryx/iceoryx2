// Copyright (c) 2020 - 2022 by Apex.AI Inc. All rights reserved.
//
// This program and the accompanying materials are made available under the
// terms of the Apache Software License 2.0 which is available at
// https://www.apache.org/licenses/LICENSE-2.0, or the MIT license
// which is available at https://opensource.org/licenses/MIT.
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

#ifndef IOX_HOOFS_MEMORY_STATIC_STORAGE_INL
#define IOX_HOOFS_MEMORY_STATIC_STORAGE_INL

#include "iox/detail/static_storage.hpp"

namespace iox
{
template <uint64_t Capacity, uint64_t Align>
constexpr uint64_t static_storage<Capacity, Align>::align_mismatch(uint64_t align, uint64_t requiredAlign) noexcept
{
    const uint64_t r = align % requiredAlign;

    // If r != 0 we are not aligned with requiredAlign and need to add r to an align
    // aligned address to be aligned with requiredAlign.
    // In the worst case r is requiredAlign - 1
    return (r != 0) ? (requiredAlign - r) : 0;
}

template <uint64_t Capacity, uint64_t Align>
static_storage<Capacity, Align>::~static_storage() noexcept
{
    deallocate();
}

template <uint64_t Capacity, uint64_t Align>
template <typename T>
constexpr bool static_storage<Capacity, Align>::is_allocatable() noexcept
{
    // note that we can guarantee it to be allocatable if we have
    // Capacity >= sizeof(T) + alignof(T) - 1
    return allocation_size<T>() <= Capacity;
}

template <uint64_t Capacity, uint64_t Align>
template <typename T>
constexpr T* static_storage<Capacity, Align>::allocate() noexcept
{
    static_assert(is_allocatable<T>(), "type does not fit into static storage");
    // AXIVION Next Construct AutosarC++19_03-M5.2.8: conversion to typed pointer is intentional,
    // it is correctly aligned and points to sufficient memory for a T by design
    return static_cast<T*>(allocate(alignof(T), sizeof(T)));
}

template <uint64_t Capacity, uint64_t Align>
constexpr void* static_storage<Capacity, Align>::allocate(const uint64_t align, const uint64_t size) noexcept
{
    if (m_ptr != nullptr)
    {
        return nullptr; // cannot allocate, already in use
    }

    size_t space{Capacity};
    m_ptr = m_bytes;
    if (std::align(static_cast<size_t>(align), static_cast<size_t>(size), m_ptr, space) != nullptr)
    {
        // fits, ptr was potentially modified to reflect alignment
        return m_ptr;
    }

    // does not fit
    return nullptr;
}

template <uint64_t Capacity, uint64_t Align>
constexpr void static_storage<Capacity, Align>::deallocate() noexcept
{
    m_ptr = nullptr;
}

template <uint64_t Capacity, uint64_t Align>
constexpr bool static_storage<Capacity, Align>::clear() noexcept
{
    if (m_ptr == nullptr)
    {
        std::memset(m_bytes, 0, Capacity);
        return true;
    }
    return false;
}

template <uint64_t Capacity, uint64_t Align>
constexpr uint64_t static_storage<Capacity, Align>::capacity() noexcept
{
    return Capacity;
}

template <uint64_t Capacity, uint64_t Align>
template <typename T>
constexpr uint64_t static_storage<Capacity, Align>::allocation_size() noexcept
{
    return sizeof(T) + align_mismatch(Align, alignof(T));
}

} // namespace iox


#endif // IOX_HOOFS_MEMORY_STATIC_STORAGE_INL
