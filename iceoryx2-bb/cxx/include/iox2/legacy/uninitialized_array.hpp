// Copyright (c) 2020 by Robert Bosch GmbH. All rights reserved.
// Copyright (c) 2021 - 2022 by Apex.AI Inc. All rights reserved.
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

#ifndef IOX2_BB_CONTAINER_UNINITIALIZED_ARRAY_HPP
#define IOX2_BB_CONTAINER_UNINITIALIZED_ARRAY_HPP

#include <cstdint>

namespace iox2 {
namespace legacy {
/// @brief struct used as policy parameter in UninitializedArray to wrap an array with all elements zeroed
/// @tparam ElementType is the array type
/// @tparam Capacity is the array size
// AXIVION DISABLE STYLE AutosarC++19_03-A9.6.1 : False positive. Used ElementTypes have defined size.
template <typename ElementType, uint64_t Capacity>
struct ZeroedBuffer {
    struct alignas(ElementType) element_t {
        // AXIVION Next Construct AutosarC++19_03-M0.1.3 : the field is intentionally unused and serves as a mean to provide memory
        // AXIVION Next Construct AutosarC++19_03-A1.1.1 : object size depends on template parameter and has to be taken care of at the specific template instantiation
        // AXIVION Next Construct AutosarC++19_03-A18.1.1 : required as low level building block, encapsulated in abstraction and not directly used
        // NOLINTNEXTLINE(cppcoreguidelines-avoid-c-arrays, hicpp-avoid-c-arrays)
        char data[sizeof(ElementType)];
    };
    // AXIVION Next Construct AutosarC++19_03-A1.1.1 : object size depends on template parameter and has to be taken care of at the specific template instantiation
    // AXIVION Next Construct AutosarC++19_03-A18.1.1 : required as low level building block, encapsulated in abstraction and not directly used
    // NOLINTNEXTLINE(cppcoreguidelines-avoid-c-arrays, hicpp-avoid-c-arrays)
    element_t value[Capacity] {};
};

/// @brief struct used as policy parameter in UninitializedArray to wrap an uninitialized array
/// @tparam ElementType is the array type
/// @tparam Capacity is the array size
template <typename ElementType, uint64_t Capacity>
struct NonZeroedBuffer {
    struct alignas(ElementType) element_t {
        // AXIVION Next Construct AutosarC++19_03-M0.1.3 : the field is intentionally unused and serves as a mean to provide memory
        // AXIVION Next Construct AutosarC++19_03-A1.1.1 : object size depends on template parameter and has to be taken care of at the specific template instantiation
        // AXIVION Next Construct AutosarC++19_03-A18.1.1 : required as low level building block, encapsulated in abstraction and not directly used
        // NOLINTNEXTLINE(cppcoreguidelines-avoid-c-arrays, hicpp-avoid-c-arrays)
        char data[sizeof(ElementType)];
    };
    // AXIVION Next Construct AutosarC++19_03-A1.1.1 : object size depends on template parameter and has to be taken care of at the specific template instantiation
    // AXIVION Next Construct AutosarC++19_03-A18.1.1 : required as low level building block, encapsulated in abstraction and not directly used
    // NOLINTNEXTLINE(cppcoreguidelines-avoid-c-arrays, hicpp-avoid-c-arrays)
    element_t value[Capacity];
};
// AXIVION ENABLE STYLE AutosarC++19_03-A9.6.1
/// @brief Wrapper class for a C-style array of type ElementType and size Capacity. Per default it is uninitialized but
/// all elements can be zeroed via template parameter ZeroedBuffer.
/// @tparam ElementType is the array type
/// @tparam Capacity is the array size
/// @tparam Buffer is the policy parameter to choose between an uninitialized, not zeroed array (=NonZeroedBuffer,
/// default) and an uninitialized array with all elements zeroed (=ZeroedBuffer)
/// @note Out of bounds access leads to undefined behavior
// AXIVION Next Construct AutosarC++19_03-A9.6.1 : type contains a single member that is a byte array whos size is defined by ElementType and Capacity
template <typename ElementType, uint64_t Capacity, template <typename, uint64_t> class Buffer = NonZeroedBuffer>
class UninitializedArray final {
    static_assert(Capacity > 0U, "The size of the UninitializedArray must be greater than 0!");

  public:
    using value_type = ElementType;
    using iterator = ElementType*;
    using const_iterator = const ElementType*;

    // The (empty) user-defined constructor is required.
    // Use of "= default" leads to value-initialization of class members.
    // AXIVION Next Construct AutosarC++19_03-A12.6.1 : This is a low-level building block which is supposed to provide uninitialized memory
    // NOLINTNEXTLINE(hicpp-use-equals-default)
    constexpr UninitializedArray() noexcept { };
    UninitializedArray(const UninitializedArray&) = delete;
    UninitializedArray(UninitializedArray&&) = delete;
    UninitializedArray& operator=(const UninitializedArray&) = delete;
    UninitializedArray& operator=(UninitializedArray&&) = delete;
    ~UninitializedArray() = default;

    /// @brief returns a reference to the element stored at index
    /// @param[in] index position of the element to return
    /// @return reference to the element
    /// @note out of bounds access leads to undefined behavior
    constexpr ElementType& operator[](const uint64_t index) noexcept;

    /// @brief returns a const reference to the element stored at index
    /// @param[in] index position of the element to return
    /// @return const reference to the element
    /// @note out of bounds access leads to undefined behavior
    constexpr const ElementType& operator[](const uint64_t index) const noexcept;

    /// @brief returns an iterator to the beginning of the UninitializedArray
    iterator begin() noexcept;

    /// @brief returns a const iterator to the beginning of the UninitializedArray
    const_iterator begin() const noexcept;

    /// @brief returns an iterator to the end of the UninitializedArray
    iterator end() noexcept;

    /// @brief returns a const iterator to the end of the UninitializedArray
    const_iterator end() const noexcept;

    /// @brief returns the array capacity
    static constexpr uint64_t capacity() noexcept;

  private:
    // AXIVION Next Construct AutosarC++19_03-A1.1.1 : object size depends on template parameter and has to be taken care of at the specific template instantiation
    Buffer<ElementType, Capacity> m_buffer;
};

/// @brief Returns N
/// @tparam T Type of the iox2::legacy::UninitializedArray
/// @tparam N Size of the iox2::legacy::UninitializedArray
/// @param An iox2::legacy::UninitializedArray
/// @return Size of the iox2::legacy::UninitializedArray
template <typename T, uint64_t N, template <typename, uint64_t> class Buffer>
constexpr uint64_t size(const UninitializedArray<T, N, Buffer>&) noexcept;

} // namespace legacy
} // namespace iox2

#include "iox2/legacy/detail/uninitialized_array.inl"

#endif // IOX2_BB_CONTAINER_UNINITIALIZED_ARRAY_HPP
