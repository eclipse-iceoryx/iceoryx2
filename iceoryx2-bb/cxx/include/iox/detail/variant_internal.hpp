// Copyright (c) 2019 by Robert Bosch GmbH. All rights reserved.
// Copyright (c) 2021 - 2022 by Apex.AI Inc. All rights reserved.
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
#ifndef IOX_HOOFS_VOCABULARY_VARIANT_INTERNAL_HPP
#define IOX_HOOFS_VOCABULARY_VARIANT_INTERNAL_HPP

#include "iox/assertions.hpp"

#include <cstdint>
#include <type_traits>
#include <utility>

namespace iox
{
template <uint64_t N>
struct in_place_index;

template <typename T>
struct in_place_type;
namespace internal
{
template <typename N>
struct is_in_place_index : std::false_type
{
};

template <uint64_t N>
struct is_in_place_index<in_place_index<N>> : std::true_type
{
};

template <typename T>
struct is_in_place_type : std::false_type
{
};

template <typename T>
struct is_in_place_type<in_place_type<T>> : std::true_type
{
};

template <typename TypeToCheck, typename T, typename... Targs>
struct does_contain_type
{
    static constexpr bool value{std::is_same<TypeToCheck, T>::value || does_contain_type<TypeToCheck, Targs...>::value};
};

template <typename TypeToCheck, typename T>
struct does_contain_type<TypeToCheck, T>
{
    static constexpr bool value{std::is_same<TypeToCheck, T>::value};
};

template <uint64_t N, typename Type, typename T, typename... Targs>
struct get_index_of_type
{
    static constexpr uint64_t index{get_index_of_type<N + 1, Type, Targs...>::index};
};

template <uint64_t N, typename Type, typename... Targs>
struct get_index_of_type<N, Type, Type, Targs...>
{
    static constexpr uint64_t index{N};
};

template <uint64_t N, uint64_t Index, typename T, typename... Targs>
struct get_type_at_index
{
    using type = typename get_type_at_index<N + 1, Index, Targs...>::type;
};

template <uint64_t N, typename T, typename... Targs>
struct get_type_at_index<N, N, T, Targs...>
{
    using type = T;
};

// AXIVION DISABLE STYLE AutosarC++19_03-M5.2.8 : conversion to typed pointer is intentional, it is correctly aligned and points to sufficient memory for a T by design of variant; note that this is an internal class used by variant
// AXIVION DISABLE STYLE AutosarC++19_03-A5.2.4 : conversion to typed pointer is intentional, it is correctly aligned and points to sufficient memory for a T by design of variant; note that this is an internal class used by variant
// AXIVION DISABLE STYLE AutosarC++19_03-A18.5.10 : destination is aligned to the maximum alignment of Types, see variant.hpp
// AXIVION DISABLE STYLE FaultDetection-IndirectAssignmentOverflow : destination is aligned to the maximum alignment of Types, see variant.hpp
// AXIVION DISABLE STYLE AutosarC++19_03-A5.3.2 : this is an internal struct used only by variant; all pointer arguments are checked in the variant class before the static functions are called
// NOLINTBEGIN(cppcoreguidelines-pro-type-reinterpret-cast)
template <uint64_t N, typename T, typename... Targs>
struct call_at_index
{
    static void destructor(const uint64_t index, void* ptr) noexcept
    {
        if (N == index)
        {
            reinterpret_cast<T*>(ptr)->~T();
        }
        else
        {
            call_at_index<N + 1, Targs...>::destructor(index, ptr);
        }
    }

    static void move(const uint64_t index, void* source, void* const destination) noexcept
    {
        if (N == index)
        {
            *reinterpret_cast<T*>(destination) = std::move(*reinterpret_cast<T*>(source));
        }
        else
        {
            call_at_index<N + 1, Targs...>::move(index, source, destination);
        }
    }

    static void moveConstructor(const uint64_t index, void* source, void* const destination) noexcept
    {
        if (N == index)
        {
            new (destination) T(std::move(*reinterpret_cast<T*>(source)));
        }
        else
        {
            call_at_index<N + 1, Targs...>::moveConstructor(index, source, destination);
        }
    }

    static void copy(const uint64_t index, const void* const source, void* const destination) noexcept
    {
        if (N == index)
        {
            *reinterpret_cast<T*>(destination) = *reinterpret_cast<const T*>(source);
        }
        else
        {
            call_at_index<N + 1, Targs...>::copy(index, source, destination);
        }
    }

    static void copyConstructor(const uint64_t index, const void* const source, void* const destination) noexcept
    {
        if (N == index)
        {
            new (destination) T(*reinterpret_cast<const T*>(source));
        }
        else
        {
            call_at_index<N + 1, Targs...>::copyConstructor(index, source, destination);
        }
    }

    static bool equality(const uint64_t index, const void* const lhs, const void* const rhs) noexcept
    {
        if (N == index)
        {
            return *reinterpret_cast<const T*>(lhs) == *reinterpret_cast<const T*>(rhs);
        }
        return call_at_index<N + 1, Targs...>::equality(index, lhs, rhs);
    }
};

template <uint64_t N, typename T>
struct call_at_index<N, T>
{
    static void destructor(const uint64_t index, void* ptr) noexcept
    {
        if (N == index)
        {
            reinterpret_cast<T*>(ptr)->~T();
        }
        else
        {
            IOX_PANIC("Could not call destructor for variant element");
        }
    }

    static void move(const uint64_t index, void* source, void* const destination) noexcept
    {
        if (N == index)
        {
            *reinterpret_cast<T*>(destination) = std::move(*reinterpret_cast<T*>(source));
        }
        else
        {
            IOX_PANIC("Could not call move assignment for variant element");
        }
    }

    static void moveConstructor(const uint64_t index, void* source, void* const destination) noexcept
    {
        if (N == index)
        {
            new (destination) T(std::move(*reinterpret_cast<T*>(source)));
        }
        else
        {
            IOX_PANIC("Could not call move constructor for variant element");
        }
    }

    static void copy(const uint64_t index, const void* const source, void* const destination) noexcept
    {
        if (N == index)
        {
            *reinterpret_cast<T*>(destination) = *reinterpret_cast<const T*>(source);
        }
        else
        {
            IOX_PANIC("Could not call copy assignment for variant element");
        }
    }

    static void copyConstructor(const uint64_t index, const void* const source, void* const destination) noexcept
    {
        if (N == index)
        {
            new (destination) T(*reinterpret_cast<const T*>(source));
        }
        else
        {
            IOX_PANIC("Could not call copy constructor for variant element");
        }
    }

    static bool equality(const uint64_t index, const void* const lhs, const void* const rhs) noexcept
    {
        if (N == index)
        {
            return *reinterpret_cast<const T*>(lhs) == *reinterpret_cast<const T*>(rhs);
        }
        IOX_PANIC("Could not call equality operator for variant element");
        return false;
    }
};
// NOLINTEND(cppcoreguidelines-pro-type-reinterpret-cast)
// AXIVION ENABLE STYLE AutosarC++19_03-A5.3.2
// AXIVION ENABLE STYLE FaultDetection-IndirectAssignmentOverflow
// AXIVION ENABLE STYLE AutosarC++19_03-A18.5.10
// AXIVION ENABLE STYLE AutosarC++19_03-A5.2.4
// AXIVION ENABLE STYLE AutosarC++19_03-M5.2.8

} // namespace internal
} // namespace iox

#endif // IOX_HOOFS_VOCABULARY_VARIANT_INTERNAL_HPP
