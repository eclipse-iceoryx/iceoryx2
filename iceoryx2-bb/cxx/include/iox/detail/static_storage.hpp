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

#ifndef IOX_HOOFS_MEMORY_STATIC_STORAGE_HPP
#define IOX_HOOFS_MEMORY_STATIC_STORAGE_HPP

#include <cstdint>
#include <cstring>
#include <memory>

namespace iox
{
/// @brief Static storage class to allocate memory for objects of type not yet known.
///        This storage is not aware of any underlying type.
///        It can be used where abstract static memory for some object is required.
///        Currently this memory is allocated on the stack but it could be implemented
///        to use memory from the static memory segment.
/// @tparam Capacity number of bytes the static_storage will allocate statically.
/// @tparam Align alignment of the allocated memory.

/// @note We can define optimized_storage (or dynamic_storage) with a similar interface
///       but other allocation policies and use them where we need to store objects
///       with some interchangable storage policy (e.g. in storable_function)
///       optimized_storage would have a dynamic memory fallback when static memory is
///       insufficent.
template <uint64_t Capacity, uint64_t Align = 1>
// NOLINTJUSTIFICATION static_storage provides uninitialized memory, correct initialization is the users
//                     responsibility whenever memory with "allocate" is acquired
// NOLINTNEXTLINE(cppcoreguidelines-pro-type-member-init,hicpp-member-init)
class static_storage final
{
  public:
    constexpr static_storage() noexcept = default;

    ~static_storage() noexcept;

    /// @note: It is not supposed to be copied or moved for now
    /// (construct a new one explicitly and populate it instead).
    /// Move would just be a copy and copy has the problem
    /// that it would just can perform a memcpy regardless of what is stored
    /// leading to potentially missed dtor calls
    static_storage(const static_storage&) = delete;
    static_storage& operator=(const static_storage&) = delete;
    static_storage(static_storage&&) = delete;
    static_storage& operator=(static_storage&&) = delete;

    /// @brief check whether the type T will fit in the buffer
    /// @return true if the type fits in the buffer, false otherwise
    /// @note can be checked at compile time
    template <typename T>
    static constexpr bool is_allocatable() noexcept;

    /// @brief provide static memory for an object of type T
    /// @return pointer to memory where a T can be constructed if memory is available, nullptr otherwise
    /// @note  compilation fails if storage is insufficient for objects of type T
    template <typename T>
    constexpr T* allocate() noexcept;

    /// @brief request aligned memory with a specific size in bytes
    /// @param align alignment of the memory requested
    /// @param size number of bytes of the memory requested
    /// @return pointer to aligned memory of the requested size if available, nullptr otherwise
    constexpr void* allocate(const uint64_t align, const uint64_t size) noexcept;

    /// @brief mark the static memory as unused
    /// @note no dtor of the stored type is called (we do not know the type)
    ///       nor is it overwritten. Setting the memory to zero can be done with clear.
    constexpr void deallocate() noexcept;

    /// @brief set the managed static memory to all zeros if there is no object currently stored
    /// @return true if the memory was set to zero, false otherwise
    constexpr bool clear() noexcept;

    /// @brief get the storage capacity in bytes
    /// @return maximum number of bytes available in the static storage
    /// @note this is an upper bound for the object size it can store
    static constexpr uint64_t capacity() noexcept;

    /// @brief return the number of bytes that need to be allocated to store a T in this storage class
    /// @return number of bytes the storage will allocate if we allocate a T
    /// @note the returned size s satisfies sizeof(T) <= s < sizeof(T) + alignof(T)
    template <typename T>
    static constexpr uint64_t allocation_size() noexcept;

  private:
    // AXIVION Next Construct AutosarC++19_03-A18.1.1 : safe access is guaranteed since the c-array is wrapped inside the static_storage
    // NOLINTNEXTLINE(hicpp-avoid-c-arrays,cppcoreguidelines-avoid-c-arrays)
    alignas(Align) uint8_t m_bytes[Capacity];
    void* m_ptr{nullptr};

    static constexpr uint64_t align_mismatch(uint64_t align, uint64_t requiredAlign) noexcept;
};

} // namespace iox

#include "iox/detail/static_storage.inl"
#endif // IOX_HOOFS_MEMORY_STATIC_STORAGE_HPP
