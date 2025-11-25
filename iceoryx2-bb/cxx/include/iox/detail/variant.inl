// Copyright (c) 2019 by Robert Bosch GmbH. All rights reserved.
// Copyright (c) 2021 - 2022 by Apex.AI Inc. All rights reserved.
// Copyright (c) 2021 by Perforce All rights reserved.
// Copyright (c) 2023 by ekxide IO GmbH. All rights reserved.
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
#ifndef IOX_HOOFS_VOCABULARY_VARIANT_INL
#define IOX_HOOFS_VOCABULARY_VARIANT_INL

#include "iox/logging.hpp"
#include "iox/variant.hpp"

namespace iox
{
template <typename... Types>
// AXIVION Next Construct AutosarC++19_03-A12.1.5: constructor delegation is not feasible here due to lack of sufficient common initialization
inline constexpr variant<Types...>::variant(const variant& rhs) noexcept
    : m_type_index(rhs.m_type_index)
{
    if (m_type_index != INVALID_VARIANT_INDEX)
    {
        internal::call_at_index<0, Types...>::copyConstructor(m_type_index, &rhs.m_storage, &m_storage);
    }
}

template <typename... Types>
template <uint64_t N, typename... CTorArguments>
// NOLINTJUSTIFICATION First param is helper struct only
// NOLINTNEXTLINE(hicpp-named-parameter)
inline constexpr variant<Types...>::variant(const in_place_index<N>&, CTorArguments&&... args) noexcept
{
    emplace_at_index<N>(std::forward<CTorArguments>(args)...);
}

template <typename... Types>
template <typename T, typename... CTorArguments>
// NOLINTJUSTIFICATION First param is helper struct only
// NOLINTNEXTLINE(hicpp-named-parameter)
inline constexpr variant<Types...>::variant(const in_place_type<T>&, CTorArguments&&... args) noexcept
{
    emplace<T>(std::forward<CTorArguments>(args)...);
}

template <typename... Types>
template <typename T,
          typename,
          typename std::enable_if_t<!internal::is_in_place_index<std::decay_t<T>>::value, bool>,
          typename std::enable_if_t<!internal::is_in_place_type<std::decay_t<T>>::value, bool>>
inline constexpr variant<Types...>::variant(T&& arg) noexcept
    : variant(in_place_type<std::decay_t<T>>(), std::forward<T>(arg))
{
}

// AXIVION Next Construct AutosarC++19_03-A13.3.1 : False positive. Overload excluded via std::enable_if in operator=(T&& rhs)
template <typename... Types>
inline constexpr variant<Types...>& variant<Types...>::operator=(const variant& rhs) noexcept
{
    if (this != &rhs)
    {
        if (m_type_index != rhs.m_type_index)
        {
            call_element_destructor();
            m_type_index = rhs.m_type_index;

            if (m_type_index != INVALID_VARIANT_INDEX)
            {
                internal::call_at_index<0, Types...>::copyConstructor(m_type_index, &rhs.m_storage, &m_storage);
            }
        }
        else
        {
            if (m_type_index != INVALID_VARIANT_INDEX)
            {
                internal::call_at_index<0, Types...>::copy(m_type_index, &rhs.m_storage, &m_storage);
            }
        }
    }
    return *this;
}

template <typename... Types>
inline constexpr variant<Types...>::variant(variant&& rhs) noexcept
    : m_type_index{std::move(rhs.m_type_index)}
{
    if (m_type_index != INVALID_VARIANT_INDEX)
    {
        internal::call_at_index<0, Types...>::moveConstructor(m_type_index, &rhs.m_storage, &m_storage);
    }
}

// AXIVION Next Construct AutosarC++19_03-A13.3.1 : False positive. Overload excluded via std::enable_if in operator=(T&& rhs)
template <typename... Types>
inline constexpr variant<Types...>& variant<Types...>::operator=(variant&& rhs) noexcept
{
    // AXIVION Next Construct AutosarC++19_03-M0.1.2, AutosarC++19_03-M0.1.9, FaultDetection-DeadBranches : False positive. Check needed to avoid self assignment.
    if (this != &rhs)
    {
        if (m_type_index != rhs.m_type_index)
        {
            call_element_destructor();
            m_type_index = std::move(rhs.m_type_index);
            if (m_type_index != INVALID_VARIANT_INDEX)
            {
                internal::call_at_index<0, Types...>::moveConstructor(m_type_index, &rhs.m_storage, &m_storage);
            }
        }
        else
        {
            if (m_type_index != INVALID_VARIANT_INDEX)
            {
                internal::call_at_index<0, Types...>::move(m_type_index, &rhs.m_storage, &m_storage);
            }
        }
    }
    return *this;
}

template <typename... Types>
inline variant<Types...>::~variant() noexcept
{
    call_element_destructor();
}

template <typename... Types>
inline void variant<Types...>::call_element_destructor() noexcept
{
    if (m_type_index != INVALID_VARIANT_INDEX)
    {
        internal::call_at_index<0, Types...>::destructor(m_type_index, &m_storage);
    }
}

// NOLINTJUSTIFICATION Correct return type is used through enable_if
// NOLINTNEXTLINE(cppcoreguidelines-c-copy-assignment-signature)
template <typename... Types>
template <typename T>
inline typename std::enable_if<!std::is_same<T, variant<Types...>&>::value, variant<Types...>>::type&
variant<Types...>::operator=(T&& rhs) noexcept
{
    if (m_type_index == INVALID_VARIANT_INDEX)
    {
        m_type_index = internal::get_index_of_type<0, T, Types...>::index;
    }

    if (!has_bad_variant_element_access<T>())
    {
        // AXIVION Next Construct AutosarC++19_03-M5.2.8: conversion to typed pointer is intentional, it is correctly aligned and points to sufficient memory for a T by design
        // NOLINTNEXTLINE(cppcoreguidelines-pro-type-reinterpret-cast)
        auto storage = reinterpret_cast<T*>(&m_storage);
        *storage = std::forward<T>(rhs);
    }
    else
    {
        error_message(__PRETTY_FUNCTION__,
                      "wrong variant type assignment, another type is already "
                      "set in variant");
    }

    return *this;
}

template <typename... Types>
template <uint64_t TypeIndex, typename... CTorArguments>
inline void variant<Types...>::emplace_at_index(CTorArguments&&... args) noexcept
{
    static_assert(TypeIndex <= sizeof...(Types), "TypeIndex is out of bounds");

    using T = typename internal::get_type_at_index<0, TypeIndex, Types...>::type;

    call_element_destructor();
    // AXIVION Next Construct AutosarC++19_03-A18.5.10, FaultDetection-IndirectAssignmentOverflow : m_storage is aligned to the maximum alignment of Types
    new (&m_storage) T(std::forward<CTorArguments>(args)...);
    m_type_index = TypeIndex;
}

template <typename... Types>
template <typename T, typename... CTorArguments>
inline void variant<Types...>::emplace(CTorArguments&&... args) noexcept
{
    static_assert(internal::does_contain_type<T, Types...>::value, "variant does not contain given type");

    call_element_destructor();

    new (&m_storage) T(std::forward<CTorArguments>(args)...);
    m_type_index = internal::get_index_of_type<0, T, Types...>::index;
}

template <typename... Types>
template <uint64_t TypeIndex>
inline typename internal::get_type_at_index<0, TypeIndex, Types...>::type* variant<Types...>::get_at_index() noexcept
{
    if (TypeIndex != m_type_index)
    {
        return nullptr;
    }

    return unsafe_get_at_index_unchecked<TypeIndex>();
}

template <typename... Types>
template <uint64_t TypeIndex>
inline typename internal::get_type_at_index<0, TypeIndex, Types...>::type*
variant<Types...>::unsafe_get_at_index_unchecked() noexcept
{
    using T = typename internal::get_type_at_index<0, TypeIndex, Types...>::type;

    // AXIVION Next Construct AutosarC++19_03-M5.2.8 : conversion to typed pointer is intentional, it is correctly aligned and points to sufficient memory for a T by design
    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-reinterpret-cast)
    return reinterpret_cast<T*>(&m_storage);
}

template <typename... Types>
template <uint64_t TypeIndex>
inline const typename internal::get_type_at_index<0, TypeIndex, Types...>::type*
variant<Types...>::get_at_index() const noexcept
{
    using T = typename internal::get_type_at_index<0, TypeIndex, Types...>::type;
    // AXIVION Next Construct AutosarC++19_03-A5.2.3 : avoid code duplication
    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-const-cast)
    return const_cast<const T*>(const_cast<variant*>(this)->template get_at_index<TypeIndex>());
}

template <typename... Types>
template <uint64_t TypeIndex>
inline const typename internal::get_type_at_index<0, TypeIndex, Types...>::type*
variant<Types...>::unsafe_get_at_index_unchecked() const noexcept
{
    using T = typename internal::get_type_at_index<0, TypeIndex, Types...>::type;
    // AXIVION Next Construct AutosarC++19_03-A5.2.3 : avoid code duplication
    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-const-cast)
    return const_cast<const T*>(const_cast<variant*>(this)->template unsafe_get_at_index_unchecked<TypeIndex>());
}

template <typename... Types>
template <typename T>
inline const T* variant<Types...>::get() const noexcept
{
    if (has_bad_variant_element_access<T>())
    {
        return nullptr;
    }
    // AXIVION Next Construct AutosarC++19_03-M5.2.8 : conversion to typed pointer is intentional, it is correctly aligned and points to sufficient memory for a T by design
    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-reinterpret-cast)
    return reinterpret_cast<const T*>(&m_storage);
}

template <typename... Types>
template <typename T>
inline T* variant<Types...>::get() noexcept
{
    // AXIVION Next Construct AutosarC++19_03-A5.2.3 : avoid code duplication
    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-const-cast)
    return const_cast<T*>(const_cast<const variant*>(this)->get<T>());
}

template <typename... Types>
template <typename T>
inline T* variant<Types...>::get_if(T* defaultValue) noexcept
{
    // AXIVION Next Construct AutosarC++19_03-A5.2.3 : avoid code duplication
    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-const-cast)
    return const_cast<T*>(const_cast<const variant*>(this)->get_if<T>(const_cast<const T*>(defaultValue)));
}

template <typename... Types>
template <typename T>
inline const T* variant<Types...>::get_if(const T* defaultValue) const noexcept
{
    if (has_bad_variant_element_access<T>())
    {
        return defaultValue;
    }

    return this->get<T>();
}

template <typename... Types>
constexpr uint64_t variant<Types...>::index() const noexcept
{
    return m_type_index;
}

template <typename... Types>
template <typename T>
inline bool variant<Types...>::has_bad_variant_element_access() const noexcept
{
    static_assert(internal::does_contain_type<T, Types...>::value, "variant does not contain given type");
    return (m_type_index != internal::get_index_of_type<0, T, Types...>::index);
}

template <typename... Types>
// AXIVION Next Construct AutosarC++19_03-A3.9.1 : see at declaration in header
inline void variant<Types...>::error_message(const char* source, const char* msg) noexcept
{
    IOX_LOG(Error, source << " ::: " << msg);
}

template <typename T, typename... Types>
inline constexpr bool holds_alternative(const variant<Types...>& variant) noexcept
{
    return variant.template get<T>() != nullptr;
}

template <typename... Types>
inline constexpr bool operator==(const variant<Types...>& lhs, const variant<Types...>& rhs) noexcept
{
    if ((lhs.index() == INVALID_VARIANT_INDEX) && (rhs.index() == INVALID_VARIANT_INDEX))
    {
        return true;
    }
    if (lhs.index() != rhs.index())
    {
        return false;
    }
    return internal::call_at_index<0, Types...>::equality(lhs.index(), &lhs.m_storage, &rhs.m_storage);
}

template <typename... Types>
inline constexpr bool operator!=(const variant<Types...>& lhs, const variant<Types...>& rhs) noexcept
{
    return !(lhs == rhs);
}
} // namespace iox

#endif // IOX_HOOFS_VOCABULARY_VARIANT_INL
