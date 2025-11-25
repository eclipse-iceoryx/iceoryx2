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
#ifndef IOX_HOOFS_VOCABULARY_OPTIONAL_INL
#define IOX_HOOFS_VOCABULARY_OPTIONAL_INL

#include "iox/attributes.hpp"
#include "iox/optional.hpp"

namespace iox
{
// AXIVION DISABLE STYLE AutosarC++19_03-A12.6.1 : m_data is not initialized here, since this is a
// constructor for an optional with no value; an access of the value would lead to the
// application's termination
// NOLINTNEXTLINE(cppcoreguidelines-pro-type-member-init, hicpp-member-init)
template <typename T>
inline optional<T>::optional(const nullopt_t) noexcept
{
}

// m_data is not initialized since this is a constructor for an optional with no value; an access of the value would
// lead to the application's termination
// NOLINTNEXTLINE(cppcoreguidelines-pro-type-member-init, hicpp-member-init)
template <typename T>
inline optional<T>::optional() noexcept
    : optional(nullopt_t())
{
}

// m_data is set inside construct_value
// NOLINTNEXTLINE(cppcoreguidelines-pro-type-member-init, hicpp-member-init)
template <typename T>
// NOLINTNEXTLINE(cppcoreguidelines-rvalue-reference-param-not-moved) perfect forwarding is used
inline optional<T>::optional(T&& value) noexcept
{
    // AXIVION Next Construct AutosarC++19_03-A18.9.2 : Perfect forwarding is intended here and
    // std::forward is the idiomatic way for this
    construct_value(std::forward<T>(value));
}

// m_data is set inside construct_value
// NOLINTNEXTLINE(cppcoreguidelines-pro-type-member-init, hicpp-member-init)
template <typename T>
inline optional<T>::optional(const T& value) noexcept
{
    construct_value(value);
}

// if rhs has a value m_data is set inside construct_value, otherwise this stays an optional with has no value
// NOLINTNEXTLINE(cppcoreguidelines-pro-type-member-init, hicpp-member-init)
template <typename T>
inline optional<T>::optional(const optional& rhs) noexcept
{
    if (rhs.m_hasValue)
    {
        construct_value(rhs.value());
    }
}

// if rhs has a value m_data is set inside construct_value, otherwise this stays an optional with has no value
// NOLINTNEXTLINE(cppcoreguidelines-pro-type-member-init, hicpp-member-init)
template <typename T>
inline optional<T>::optional(optional&& rhs) noexcept
{
    if (rhs.m_hasValue)
    {
        construct_value(std::move(rhs.value()));
        rhs.destruct_value();
    }
}

// m_data is set inside construct_value
// NOLINTNEXTLINE(cppcoreguidelines-pro-type-member-init, hicpp-member-init)
template <typename T>
template <typename... Targs>
// NOLINTNEXTLINE(readability-named-parameter, hicpp-named-parameter) justification in header
inline optional<T>::optional(in_place_t, Targs&&... args) noexcept
{
    construct_value(std::forward<Targs>(args)...);
}
// AXIVION ENABLE STYLE AutosarC++19_03-A12.6.1

// AXIVION Next Construct AutosarC++19_03-A13.3.1 : False positive. Overloading excluded via std::enable_if in
// typename std::enable_if<!std::is_same<U, optional&>::value, optional>::type& operator=(U&& value) noexcept.
template <typename T>
inline optional<T>& optional<T>::operator=(const optional& rhs) noexcept
{
    if (this != &rhs)
    {
        if (!rhs.m_hasValue && m_hasValue)
        {
            destruct_value();
        }
        else if (rhs.m_hasValue && m_hasValue)
        {
            // AXIVION Next Construct AutosarC++19_03-A5.0.1 : False positive! 'value()' != 'rhs.value()'
            value() = rhs.value();
        }
        else if (rhs.m_hasValue && !m_hasValue)
        {
            construct_value(rhs.value());
        }
        else
        {
            // do nothing since this and rhs contain no values
        }
    }
    return *this;
}

// AXIVION Next Construct AutosarC++19_03-A13.3.1 : False positive. Overloading excluded via std::enable_if in typename
// std::enable_if<!std::is_same<U, optional&>::value, optional>::type& operator=(U&& value) noexcept.
template <typename T>
inline optional<T>& optional<T>::operator=(optional&& rhs) noexcept
{
    if (this != &rhs)
    {
        if (!rhs.m_hasValue && m_hasValue)
        {
            destruct_value();
        }
        else if (rhs.m_hasValue && m_hasValue)
        {
            // AXIVION Next Construct AutosarC++19_03-A5.0.1 : False positive! 'value()' != 'rhs.value()'
            value() = std::move(rhs.value());
        }
        else if (rhs.m_hasValue && !m_hasValue)
        {
            construct_value(std::move(rhs.value()));
        }
        else
        {
            // do nothing since this and rhs contain no values
        }
        if (rhs.m_hasValue)
        {
            rhs.destruct_value();
        }
    }
    return *this;
}

template <typename T>
inline optional<T>::~optional() noexcept
{
    if (m_hasValue)
    {
        destruct_value();
    }
}

// NOLINTNEXTLINE(cppcoreguidelines-c-copy-assignment-signature) justification in header
template <typename T>
template <typename U>
inline typename std::enable_if<!std::is_same<U, optional<T>&>::value, optional<T>>::type&
optional<T>::operator=(U&& newValue) noexcept
{
    if (m_hasValue)
    {
        value() = std::forward<U>(newValue);
    }
    else
    {
        construct_value(std::forward<U>(newValue));
    }
    return *this;
}

template <typename T>
inline const T* optional<T>::operator->() const noexcept
{
    return &value();
}

template <typename T>
inline const T& optional<T>::operator*() const noexcept
{
    return value();
}

template <typename T>
inline T* optional<T>::operator->() noexcept
{
    return &value();
}

template <typename T>
inline T& optional<T>::operator*() noexcept
{
    return value();
}

template <typename T>
inline constexpr optional<T>::operator bool() const noexcept
{
    return m_hasValue;
}

template <typename T>
template <typename... Targs>
inline T& optional<T>::emplace(Targs&&... args) noexcept
{
    if (m_hasValue)
    {
        destruct_value();
    }

    construct_value(std::forward<Targs>(args)...);
    return value();
}

template <typename T>
inline constexpr bool optional<T>::has_value() const noexcept
{
    return m_hasValue;
}

template <typename T>
inline void optional<T>::reset() noexcept
{
    if (m_hasValue)
    {
        destruct_value();
    }
}

template <typename T>
inline T& optional<T>::value() & noexcept
{
    IOX_ENFORCE(has_value(), "Calling 'value' on an 'optional' which does not have a value");
    // AXIVION Next Construct AutosarC++19_03-M5.2.8 : The optional has the type T defined
    // during compile time and the type is unchangeable during the lifetime of the object.
    // All accesses to the underlying data is done via the same static type and therefore the
    // casts are always valid
    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-reinterpret-cast)
    return *reinterpret_cast<T*>(&m_data);
}

template <typename T>
inline const T& optional<T>::value() const& noexcept
{
    // AXIVION Next Construct AutosarC++19_03-A5.2.3 : Avoid code duplication
    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-const-cast)
    return const_cast<optional<T>*>(this)->value();
}

template <typename T>
inline T&& optional<T>::value() && noexcept
{
    IOX_ENFORCE(has_value(), "Calling 'value' on an 'optional' which does not have a value");
    // AXIVION Next Construct AutosarC++19_03-M5.2.8 : The optional has the type T defined
    // during compile time and the type is unchangeable during the lifetime of the object.
    // All accesses to the underlying data is done via the same static type and therefore the
    // casts are always valid
    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-reinterpret-cast)
    return std::move(*reinterpret_cast<T*>(&m_data));
}

template <typename T>
inline const T&& optional<T>::value() const&& noexcept
{
    // AXIVION Next Construct AutosarC++19_03-A5.2.3 : Avoid code duplication
    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-const-cast)
    return std::move(*const_cast<optional<T>*>(this)->value());
}

template <typename T>
template <typename... Targs>
inline void optional<T>::construct_value(Targs&&... args) noexcept
{
    new (&m_data) T(std::forward<Targs>(args)...);
    m_hasValue = true;
}

template <typename T>
inline void optional<T>::destruct_value() noexcept
{
    value().~T();
    m_hasValue = false;
}
// AXIVION Next Construct AutosarC++19_03-M17.0.3 : make_optional is defined within the iox namespace which prevents easy misuse
template <typename OptionalBaseType, typename... Targs>
inline optional<OptionalBaseType> make_optional(Targs&&... args) noexcept
{
    optional<OptionalBaseType> returnValue{nullopt_t()};
    returnValue.emplace(std::forward<Targs>(args)...);
    return returnValue;
}

template <typename T>
bool operator==(const optional<T>& lhs, const optional<T>& rhs) noexcept
{
    const auto bothNull = !lhs.has_value() && !rhs.has_value();
    const auto bothValuesEqual = (lhs.has_value() && rhs.has_value()) && (*lhs == *rhs);
    return bothNull || bothValuesEqual;
}

template <typename T>
bool operator!=(const optional<T>& lhs, const optional<T>& rhs) noexcept
{
    const auto onlyLhsNul = !lhs.has_value() && rhs.has_value();
    const auto onlyRhsNul = lhs.has_value() && !rhs.has_value();
    const auto bothValuesUnequal = (lhs.has_value() && rhs.has_value()) && (*lhs != *rhs);
    return (bothValuesUnequal || onlyRhsNul) || onlyLhsNul;
}

// AXIVION DISABLE STYLE AutosarC++19_03-A13.5.5: Comparison with nullopt_t is required
template <typename T>
bool operator==(const optional<T>& lhs, const nullopt_t) noexcept
{
    return !lhs.has_value();
}

template <typename T>
bool operator==(const nullopt_t, const optional<T>& rhs) noexcept
{
    return !rhs.has_value();
}

template <typename T>
bool operator!=(const optional<T>& lhs, const nullopt_t) noexcept
{
    return lhs.has_value();
}

template <typename T>
bool operator!=(const nullopt_t, const optional<T>& rhs) noexcept
{
    return rhs.has_value();
}
// AXIVION ENABLE STYLE AutosarC++19_03-A13.5.5
} // namespace iox

#endif // IOX_HOOFS_VOCABULARY_OPTIONAL_INL
