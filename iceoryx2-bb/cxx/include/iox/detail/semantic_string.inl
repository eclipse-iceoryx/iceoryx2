// Copyright (c) 2023 by Apex.AI Inc. All rights reserved.
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
#ifndef IOX_HOOFS_VOCABULARY_SEMANTIC_STRING_INL
#define IOX_HOOFS_VOCABULARY_SEMANTIC_STRING_INL

#include "iox/logging.hpp"
#include "iox/semantic_string.hpp"
#include "iox/string.hpp"

namespace iox
{

template <typename Child,
          uint64_t Capacity,
          DoesContainInvalidContent<Capacity> DoesContainInvalidContentCall,
          DoesContainInvalidCharacter<Capacity> DoesContainInvalidCharacterCall>
template <uint64_t N>
inline SemanticString<Child, Capacity, DoesContainInvalidContentCall, DoesContainInvalidCharacterCall>::SemanticString(
    const string<N>& value) noexcept
    : m_data{value}
{
}

template <typename Child,
          uint64_t Capacity,
          DoesContainInvalidContent<Capacity> DoesContainInvalidContentCall,
          DoesContainInvalidCharacter<Capacity> DoesContainInvalidCharacterCall>
template <uint64_t N>
inline expected<Child, SemanticStringError>
SemanticString<Child, Capacity, DoesContainInvalidContentCall, DoesContainInvalidCharacterCall>::create_impl(
    const char* value) noexcept
{
    if (N > Capacity)
    {
        IOX_LOG(Debug,
                "Unable to create semantic string since the value \""
                    << value << "\" exceeds the maximum valid length of " << Capacity << ".");
        return err(SemanticStringError::ExceedsMaximumLength);
    }

    string<Capacity> str{TruncateToCapacity, value};

    if (DoesContainInvalidCharacterCall(str))
    {
        IOX_LOG(Debug,
                "Unable to create semantic string since the value \"" << value << "\" contains invalid characters");
        return err(SemanticStringError::ContainsInvalidCharacters);
    }

    if (DoesContainInvalidContentCall(str))
    {
        IOX_LOG(Debug, "Unable to create semantic string since the value \"" << value << "\" contains invalid content");
        return err(SemanticStringError::ContainsInvalidContent);
    }

    return ok(Child(str));
}

template <typename Child,
          uint64_t Capacity,
          DoesContainInvalidContent<Capacity> DoesContainInvalidContentCall,
          DoesContainInvalidCharacter<Capacity> DoesContainInvalidCharacterCall>
template <uint64_t N>
inline expected<Child, SemanticStringError>
SemanticString<Child, Capacity, DoesContainInvalidContentCall, DoesContainInvalidCharacterCall>::create(
    // avoid-c-arrays: justification in header
    // NOLINTNEXTLINE(hicpp-avoid-c-arrays, cppcoreguidelines-avoid-c-arrays, hicpp-explicit-conversions)
    const char (&value)[N]) noexcept
{
    return SemanticString<Child, Capacity, DoesContainInvalidContentCall, DoesContainInvalidCharacterCall>::
        template create_impl<N>(value);
}

template <typename Child,
          uint64_t Capacity,
          DoesContainInvalidContent<Capacity> DoesContainInvalidContentCall,
          DoesContainInvalidCharacter<Capacity> DoesContainInvalidCharacterCall>
template <uint64_t N>
inline expected<Child, SemanticStringError>
SemanticString<Child, Capacity, DoesContainInvalidContentCall, DoesContainInvalidCharacterCall>::create(
    const string<N>& value) noexcept
{
    return SemanticString<Child, Capacity, DoesContainInvalidContentCall, DoesContainInvalidCharacterCall>::
        template create_impl<N>(value.c_str());
}

template <typename Child,
          uint64_t Capacity,
          DoesContainInvalidContent<Capacity> DoesContainInvalidContentCall,
          DoesContainInvalidCharacter<Capacity> DoesContainInvalidCharacterCall>
inline constexpr uint64_t
SemanticString<Child, Capacity, DoesContainInvalidContentCall, DoesContainInvalidCharacterCall>::size() const noexcept
{
    return m_data.size();
}

template <typename Child,
          uint64_t Capacity,
          DoesContainInvalidContent<Capacity> DoesContainInvalidContentCall,
          DoesContainInvalidCharacter<Capacity> DoesContainInvalidCharacterCall>
inline constexpr uint64_t
SemanticString<Child, Capacity, DoesContainInvalidContentCall, DoesContainInvalidCharacterCall>::capacity() noexcept
{
    return Capacity;
}

template <typename Child,
          uint64_t Capacity,
          DoesContainInvalidContent<Capacity> DoesContainInvalidContentCall,
          DoesContainInvalidCharacter<Capacity> DoesContainInvalidCharacterCall>
inline constexpr const string<Capacity>&
SemanticString<Child, Capacity, DoesContainInvalidContentCall, DoesContainInvalidCharacterCall>::as_string()
    const noexcept
{
    return m_data;
}

template <typename Child,
          uint64_t Capacity,
          DoesContainInvalidContent<Capacity> DoesContainInvalidContentCall,
          DoesContainInvalidCharacter<Capacity> DoesContainInvalidCharacterCall>
template <typename T>
inline expected<void, SemanticStringError>
SemanticString<Child, Capacity, DoesContainInvalidContentCall, DoesContainInvalidCharacterCall>::append(
    const T& value) noexcept
{
    return insert(size(), value, internal::GetSize<T>::call(value));
}

template <typename Child,
          uint64_t Capacity,
          DoesContainInvalidContent<Capacity> DoesContainInvalidContentCall,
          DoesContainInvalidCharacter<Capacity> DoesContainInvalidCharacterCall>
template <typename T>
inline expected<void, SemanticStringError>
SemanticString<Child, Capacity, DoesContainInvalidContentCall, DoesContainInvalidCharacterCall>::insert(
    const uint64_t pos, const T& str, const uint64_t count) noexcept
{
    auto temp = m_data;
    if (!temp.insert(pos, str, count))
    {
        IOX_LOG(Debug,
                "Unable to insert the value \""
                    << str << "\" to the semantic string since it would exceed the maximum valid length of " << Capacity
                    << ".");
        return err(SemanticStringError::ExceedsMaximumLength);
    }

    if (DoesContainInvalidCharacterCall(temp))
    {
        IOX_LOG(Debug,
                "Unable to insert the value \"" << str
                                                << "\" to the semantic string since it contains invalid characters.");
        return err(SemanticStringError::ContainsInvalidCharacters);
    }

    if (DoesContainInvalidContentCall(str))
    {
        IOX_LOG(Debug,
                "Unable to insert the value \"" << str
                                                << "\" to the semantic string since it would lead to invalid content.");
        return err(SemanticStringError::ContainsInvalidContent);
    }

    m_data = temp;
    return ok();
}

template <typename Child,
          uint64_t Capacity,
          DoesContainInvalidContent<Capacity> DoesContainInvalidContentCall,
          DoesContainInvalidCharacter<Capacity> DoesContainInvalidCharacterCall>
inline bool SemanticString<Child, Capacity, DoesContainInvalidContentCall, DoesContainInvalidCharacterCall>::operator==(
    const SemanticString& rhs) const noexcept
{
    return as_string() == rhs.as_string();
}

template <typename Child,
          uint64_t Capacity,
          DoesContainInvalidContent<Capacity> DoesContainInvalidContentCall,
          DoesContainInvalidCharacter<Capacity> DoesContainInvalidCharacterCall>
template <typename T>
inline IsStringOrCharArray<T, bool>
SemanticString<Child, Capacity, DoesContainInvalidContentCall, DoesContainInvalidCharacterCall>::operator==(
    const T& rhs) const noexcept
{
    return as_string() == rhs;
}

template <typename Child,
          uint64_t Capacity,
          DoesContainInvalidContent<Capacity> DoesContainInvalidContentCall,
          DoesContainInvalidCharacter<Capacity> DoesContainInvalidCharacterCall>
inline bool SemanticString<Child, Capacity, DoesContainInvalidContentCall, DoesContainInvalidCharacterCall>::operator!=(
    const SemanticString& rhs) const noexcept
{
    return as_string() != rhs.as_string();
}

template <typename Child,
          uint64_t Capacity,
          DoesContainInvalidContent<Capacity> DoesContainInvalidContentCall,
          DoesContainInvalidCharacter<Capacity> DoesContainInvalidCharacterCall>
template <typename T>
inline IsStringOrCharArray<T, bool>
SemanticString<Child, Capacity, DoesContainInvalidContentCall, DoesContainInvalidCharacterCall>::operator!=(
    const T& rhs) const noexcept
{
    return as_string() != rhs;
}

template <typename Child,
          uint64_t Capacity,
          DoesContainInvalidContent<Capacity> DoesContainInvalidContentCall,
          DoesContainInvalidCharacter<Capacity> DoesContainInvalidCharacterCall>
inline bool SemanticString<Child, Capacity, DoesContainInvalidContentCall, DoesContainInvalidCharacterCall>::operator<=(
    const SemanticString& rhs) const noexcept
{
    return as_string() <= rhs.as_string();
}

template <typename Child,
          uint64_t Capacity,
          DoesContainInvalidContent<Capacity> DoesContainInvalidContentCall,
          DoesContainInvalidCharacter<Capacity> DoesContainInvalidCharacterCall>
template <typename T>
inline IsStringOrCharArray<T, bool>
SemanticString<Child, Capacity, DoesContainInvalidContentCall, DoesContainInvalidCharacterCall>::operator<=(
    const T& rhs) const noexcept
{
    return as_string() <= rhs;
}

template <typename Child,
          uint64_t Capacity,
          DoesContainInvalidContent<Capacity> DoesContainInvalidContentCall,
          DoesContainInvalidCharacter<Capacity> DoesContainInvalidCharacterCall>
inline bool SemanticString<Child, Capacity, DoesContainInvalidContentCall, DoesContainInvalidCharacterCall>::operator<(
    const SemanticString& rhs) const noexcept
{
    return as_string() < rhs.as_string();
}

template <typename Child,
          uint64_t Capacity,
          DoesContainInvalidContent<Capacity> DoesContainInvalidContentCall,
          DoesContainInvalidCharacter<Capacity> DoesContainInvalidCharacterCall>
template <typename T>
inline IsStringOrCharArray<T, bool>
SemanticString<Child, Capacity, DoesContainInvalidContentCall, DoesContainInvalidCharacterCall>::operator<(
    const T& rhs) const noexcept
{
    return as_string() < rhs;
}

template <typename Child,
          uint64_t Capacity,
          DoesContainInvalidContent<Capacity> DoesContainInvalidContentCall,
          DoesContainInvalidCharacter<Capacity> DoesContainInvalidCharacterCall>
inline bool SemanticString<Child, Capacity, DoesContainInvalidContentCall, DoesContainInvalidCharacterCall>::operator>=(
    const SemanticString& rhs) const noexcept
{
    return as_string() >= rhs.as_string();
}

template <typename Child,
          uint64_t Capacity,
          DoesContainInvalidContent<Capacity> DoesContainInvalidContentCall,
          DoesContainInvalidCharacter<Capacity> DoesContainInvalidCharacterCall>
template <typename T>
inline IsStringOrCharArray<T, bool>
SemanticString<Child, Capacity, DoesContainInvalidContentCall, DoesContainInvalidCharacterCall>::operator>=(
    const T& rhs) const noexcept
{
    return as_string() >= rhs;
}

template <typename Child,
          uint64_t Capacity,
          DoesContainInvalidContent<Capacity> DoesContainInvalidContentCall,
          DoesContainInvalidCharacter<Capacity> DoesContainInvalidCharacterCall>
inline bool SemanticString<Child, Capacity, DoesContainInvalidContentCall, DoesContainInvalidCharacterCall>::operator>(
    const SemanticString& rhs) const noexcept
{
    return as_string() > rhs.as_string();
}

template <typename Child,
          uint64_t Capacity,
          DoesContainInvalidContent<Capacity> DoesContainInvalidContentCall,
          DoesContainInvalidCharacter<Capacity> DoesContainInvalidCharacterCall>
template <typename T>
inline IsStringOrCharArray<T, bool>
SemanticString<Child, Capacity, DoesContainInvalidContentCall, DoesContainInvalidCharacterCall>::operator>(
    const T& rhs) const noexcept
{
    return as_string() > rhs;
}
} // namespace iox

#endif
