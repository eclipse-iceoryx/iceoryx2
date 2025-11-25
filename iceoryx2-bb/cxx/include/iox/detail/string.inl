// Copyright (c) 2019 by Robert Bosch GmbH. All rights reserved.
// Copyright (c) 2021 - 2023 by Apex.AI Inc. All rights reserved.
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
#ifndef IOX_HOOFS_VOCABULARY_STRING_INL
#define IOX_HOOFS_VOCABULARY_STRING_INL

#include "iox/string.hpp"

#include "iox/assertions.hpp"
#include "iox/logging.hpp"

namespace iox
{
template <uint64_t Capacity>
inline string<Capacity>::string(const string& other) noexcept
{
    copy(other);
}

template <uint64_t Capacity>
inline string<Capacity>::string(string&& other) noexcept
{
    move(std::move(other));
}

template <uint64_t Capacity>
inline string<Capacity>& string<Capacity>::operator=(const string& rhs) noexcept
{
    if (this == &rhs)
    {
        return *this;
    }
    // NOLINTNEXTLINE(cppcoreguidelines-c-copy-assignment-signature) copy() returns *this
    return copy(rhs);
}

template <uint64_t Capacity>
inline string<Capacity>& string<Capacity>::operator=(string&& rhs) noexcept
{
    if (this == &rhs)
    {
        return *this;
    }
    // NOLINTNEXTLINE(cppcoreguidelines-c-copy-assignment-signature) move() returns *this
    return move(std::move(rhs));
}

template <uint64_t Capacity>
template <uint64_t N>
inline string<Capacity>::string(const string<N>& other) noexcept
{
    static_assert(N <= Capacity,
                  "Construction failed. The capacity of the given fixed string is larger than the capacity of this.");
    copy(other);
}

template <uint64_t Capacity>
template <uint64_t N>
inline string<Capacity>::string(string<N>&& other) noexcept
{
    static_assert(N <= Capacity,
                  "Construction failed. The capacity of the given fixed string is larger than the capacity of this.");
    move(std::move(other));
}

template <uint64_t Capacity>
template <uint64_t N>
inline string<Capacity>& string<Capacity>::operator=(const string<N>& rhs) noexcept
{
    static_assert(N <= Capacity,
                  "Assignment failed. The capacity of the given fixed string is larger than the capacity of this.");
    // NOLINTNEXTLINE(cppcoreguidelines-c-copy-assignment-signature) copy() returns *this
    return copy(rhs);
}

template <uint64_t Capacity>
template <uint64_t N>
inline string<Capacity>& string<Capacity>::operator=(string<N>&& rhs) noexcept
{
    static_assert(N <= Capacity,
                  "Assignment failed. The capacity of the given fixed string is larger than the capacity of this.");
    // NOLINTNEXTLINE(cppcoreguidelines-c-copy-assignment-signature) move() returns *this
    return move(std::move(rhs));
}

template <uint64_t Capacity>
template <uint64_t N>
// AXIVION Next Construct AutosarC++19_03-A18.1.1 : C-array type usage is intentional
// NOLINTNEXTLINE(hicpp-avoid-c-arrays, cppcoreguidelines-avoid-c-arrays) iox::string wraps char array
inline string<Capacity>::string(const char (&other)[N]) noexcept
{
    *this = other;
}

template <uint64_t Capacity>
// NOLINTNEXTLINE(hicpp-named-parameter, readability-named-parameter) justification in header
inline string<Capacity>::string(TruncateToCapacity_t, const char* const other) noexcept
    : string(TruncateToCapacity, other, [&other]() noexcept -> uint64_t {
        return (other != nullptr) ? strnlen(other, Capacity) : 0U;
    }())
{
}

template <uint64_t Capacity>
// TruncateToCapacity_t is a compile time variable to distinguish between constructors
// NOLINTNEXTLINE(hicpp-named-parameter, readability-named-parameter)
inline string<Capacity>::string(TruncateToCapacity_t, const char* const other, const uint64_t count) noexcept
{
    if (other == nullptr)
    {
        clear();
    }
    else if (Capacity < count)
    {
// AXIVION DISABLE STYLE AutosarC++19_03-A16.0.1: pre-processor is required for setting gcc diagnostics, since gcc 8 incorrectly warns here about out of bounds array access
// AXIVION DISABLE STYLE AutosarC++19_03-A16.7.1: see rule 'A16.0.1' above
#if (defined(__GNUC__) && (__GNUC__ == 8)) && (__GNUC_MINOR__ >= 3)
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Warray-bounds"
#endif
        std::memcpy(m_rawstring, other, Capacity);
#if (defined(__GNUC__) && (__GNUC__ == 8)) && (__GNUC_MINOR__ >= 3)
#pragma GCC diagnostic pop
#endif
        // AXIVION ENABLE STYLE AutosarC++19_03-A16.7.1
        // AXIVION ENABLE STYLE AutosarC++19_03-A16.0.1

        m_rawstring[Capacity] = '\0';
        m_rawstringSize = Capacity;
        IOX_LOG(Warn,
                "Constructor truncates the last " << count - Capacity << " characters of " << other
                                                  << ", because the char array length is larger than the capacity.");
    }
    else
    {
        std::memcpy(m_rawstring, other, static_cast<size_t>(count));
        m_rawstring[count] = '\0';
        m_rawstringSize = count;
    }
}

template <uint64_t Capacity>
template <uint64_t N>
// AXIVION Next Construct AutosarC++19_03-A18.1.1 : C-array type usage is intentional
// NOLINTNEXTLINE(hicpp-avoid-c-arrays, cppcoreguidelines-avoid-c-arrays) iox::string wraps char array
inline string<Capacity>& string<Capacity>::operator=(const char (&rhs)[N]) noexcept
{
    static_assert(N <= (Capacity + 1U),
                  "Assignment failed. The given char array is larger than the capacity of the fixed string.");

    if (c_str() == rhs)
    {
        return *this;
    }

    const auto sourceRawStringSize = static_cast<uint64_t>(strnlen(&rhs[0], N));
    if (sourceRawStringSize <= Capacity)
    {
        m_rawstringSize = sourceRawStringSize;
    }
    else
    {
        m_rawstringSize = Capacity;

        IOX_LOG(
            Warn,
            "iox::string: Assignment of array which is not zero-terminated! Last value of array overwritten with 0!");
    }

    // AXIVION DISABLE STYLE AutosarC++19_03-A16.0.1: pre-processor is required for setting gcc diagnostics, since gcc 8 incorrectly warns here about out of bounds array access
    // AXIVION DISABLE STYLE AutosarC++19_03-A16.7.1: see rule 'A16.0.1' above
#if (defined(__GNUC__) && (__GNUC__ == 8)) && (__GNUC_MINOR__ >= 3)
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Warray-bounds"
#endif
    std::memcpy(m_rawstring, rhs, static_cast<size_t>(m_rawstringSize));
#if (defined(__GNUC__) && (__GNUC__ == 8)) && (__GNUC_MINOR__ >= 3)
#pragma GCC diagnostic pop
#endif
    // AXIVION ENABLE STYLE AutosarC++19_03-A16.7.1
    // AXIVION ENABLE STYLE AutosarC++19_03-A16.0.1

    m_rawstring[m_rawstringSize] = '\0';

    return *this;
}

template <uint64_t Capacity>
template <uint64_t N>
inline string<Capacity>& string<Capacity>::assign(const string<N>& str) noexcept
{
    static_assert(N <= Capacity,
                  "Assignment failed. The capacity of the given fixed string is larger than the capacity of this.");
    *this = str;
    return *this;
}

template <uint64_t Capacity>
template <uint64_t N>
// AXIVION Next Construct AutosarC++19_03-A18.1.1 : C-array type usage is intentional
// NOLINTNEXTLINE(hicpp-avoid-c-arrays, cppcoreguidelines-avoid-c-arrays) iox::string wraps char array
inline string<Capacity>& string<Capacity>::assign(const char (&str)[N]) noexcept
{
    *this = str;
    return *this;
}

template <uint64_t Capacity>
inline bool string<Capacity>::unsafe_assign(const char* const str) noexcept
{
    if ((c_str() == str) || (str == nullptr))
    {
        return false;
    }
    const uint64_t strSize{strnlen(str, Capacity + 1U)};
    if (Capacity < strSize)
    {
        IOX_LOG(Debug,
                "Assignment failed. The given cstring is larger (" << strSize << ") than the capacity (" << Capacity
                                                                   << ") of the fixed string.");
        return false;
    }
    std::memcpy(m_rawstring, str, static_cast<size_t>(strSize));
    m_rawstring[strSize] = '\0';
    m_rawstringSize = strSize;
    return true;
}

template <uint64_t Capacity>
inline void
string<Capacity>::unsafe_raw_access(const iox::function_ref<uint64_t(char*, const iox::BufferInfo info)>& func) noexcept
{
    iox::BufferInfo info{m_rawstringSize, Capacity + 1};
    uint64_t len = func(m_rawstring, info);

    if (len > Capacity)
    {
        IOX_PANIC("'unsafe_auto_raw_access' failed. Data wrote outside the maximun string capacity.");
    }
    else if (m_rawstring[len] != '\0')
    {
        IOX_PANIC("String does not have the terminator at the returned size");
    }
    m_rawstringSize = len;
}


template <uint64_t Capacity>
template <typename T>
inline IsStringOrCharArray<T, int64_t> string<Capacity>::compare(const T& other) const noexcept
{
    const uint64_t otherSize{internal::GetSize<T>::call(other)};
    const auto result =
        memcmp(c_str(), internal::GetData<T>::call(other), static_cast<size_t>(std::min(m_rawstringSize, otherSize)));
    if (result == 0)
    {
        if (m_rawstringSize < otherSize)
        {
            return -1;
        }
        const int64_t isLargerThanOther{(m_rawstringSize > otherSize) ? 1L : 0L};
        return isLargerThanOther;
    }
    return result;
}

template <uint64_t Capacity>
inline int64_t string<Capacity>::compare(char other) const noexcept
{
    const auto result = memcmp(c_str(), &other, 1U);
    if (result == 0)
    {
        if (empty())
        {
            return -1;
        }
        return ((m_rawstringSize > 1U) ? 1L : 0L);
    }
    return result;
}

template <uint64_t Capacity>
inline const char* string<Capacity>::c_str() const noexcept
{
    return m_rawstring;
}

template <uint64_t Capacity>
inline constexpr uint64_t string<Capacity>::size() const noexcept
{
    return m_rawstringSize;
}

template <uint64_t Capacity>
inline constexpr uint64_t string<Capacity>::capacity() noexcept
{
    return Capacity;
}

template <uint64_t Capacity>
inline constexpr bool string<Capacity>::empty() const noexcept
{
    return m_rawstringSize == 0U;
}

template <uint64_t Capacity>
inline constexpr void string<Capacity>::clear() noexcept
{
    m_rawstring[0U] = '\0';
    m_rawstringSize = 0U;
}

template <uint64_t Capacity>
template <uint64_t N>
inline string<Capacity>& string<Capacity>::copy(const string<N>& rhs) noexcept
{
    static_assert(N <= Capacity,
                  "Assignment failed. The capacity of the given fixed string is larger than the capacity of this.");
    const uint64_t strSize{rhs.size()};
    std::memcpy(m_rawstring, rhs.c_str(), static_cast<size_t>(strSize));
    m_rawstring[strSize] = '\0';
    m_rawstringSize = strSize;
    return *this;
}

template <uint64_t Capacity>
template <uint64_t N>
// NOLINTNEXTLINE(cppcoreguidelines-rvalue-reference-param-not-moved) false positive, the underlying data is moved
inline string<Capacity>& string<Capacity>::move(string<N>&& rhs) noexcept
{
    static_assert(N <= Capacity,
                  "Assignment failed. The capacity of the given fixed string is larger than the capacity of this.");
    const uint64_t strSize{rhs.size()};
    std::memcpy(m_rawstring, rhs.c_str(), static_cast<size_t>(strSize));
    m_rawstring[strSize] = '\0';
    m_rawstringSize = strSize;
    rhs.clear();
    return *this;
}

// AXIVION Next Construct AutosarC++19_03-M5.17.1: This is not used as shift operator but as stream operator and does
// not require to implement '<<='
template <uint64_t Capacity>
inline log::LogStream& operator<<(log::LogStream& stream, const string<Capacity>& str) noexcept
{
    stream << str.c_str();
    return stream;
}

template <uint64_t Capacity>
template <typename T>
// NOLINTNEXTLINE(hicpp-named-parameter, readability-named-parameter) method is disabled via static_assert
inline string<Capacity>& string<Capacity>::operator+=(const T&) noexcept
{
    static_assert(always_false_v<string<Capacity>>,
                  "operator += is not supported by iox::string, please use append or unsafe_append instead");
    return *this;
}

template <typename T1, typename T2>
inline IsIoxStringOrCharArrayOrChar<T1, T2, string<internal::SumCapa<T1, T2>::value>>
// AXIVION Next Line AutosarC++19_03-M3.2.1 : False positive, the return value is compatible with the declaration
concatenate(const T1& str1, const T2& str2) noexcept
{
    uint64_t size1{internal::GetSize<T1>::call(str1)};
    uint64_t size2{internal::GetSize<T2>::call(str2)};
    using NewStringType = string<internal::SumCapa<T1, T2>::value>;
    NewStringType newString;
    std::memcpy(newString.m_rawstring, internal::GetData<T1>::call(str1), static_cast<size_t>(size1));
    std::memcpy(&newString.m_rawstring[size1], internal::GetData<T2>::call(str2), static_cast<size_t>(size2));
    newString.m_rawstring[size1 + size2] = '\0';
    newString.m_rawstringSize = size1 + size2;

    return newString;
}

template <typename T1, typename T2, typename... Targs>
inline IsIoxStringOrCharArrayOrChar<T1, T2, string<internal::SumCapa<T1, T2, Targs...>::value>>
concatenate(const T1& str1, const T2& str2, const Targs&... targs) noexcept
{
    return concatenate(concatenate(str1, str2), targs...);
}

template <typename T1, typename T2>
// AXIVION Next Construct AutosarC++19_03-M17.0.3 : operator+ is defined within iox namespace which prevents easy
// misuse
inline IsIoxStringAndIoxStringOrCharArrayOrChar<T1, T2, string<internal::SumCapa<T1, T2>::value>>
operator+(const T1& str1, const T2& str2) noexcept
{
    return concatenate(str1, str2);
}

template <uint64_t Capacity>
template <typename T>
inline IsStringOrCharArrayOrChar<T, bool> string<Capacity>::unsafe_append(const T& str) noexcept
{
    const uint64_t tSize{internal::GetSize<T>::call(str)};
    const char* const tData{internal::GetData<T>::call(str)};
    const uint64_t clampedTSize{std::min(Capacity - m_rawstringSize, tSize)};

    if (tSize > clampedTSize)
    {
        IOX_LOG(Debug, "Appending failed because the sum of sizes exceeds this' capacity.");
        return false;
    }

    std::memcpy(&(m_rawstring[m_rawstringSize]), tData, static_cast<size_t>(clampedTSize));
    m_rawstringSize += clampedTSize;
    m_rawstring[m_rawstringSize] = '\0';
    return true;
}

template <uint64_t Capacity>
template <typename T>
// TruncateToCapacity_t is a compile time variable to distinguish between constructors
// NOLINTNEXTLINE(hicpp-named-parameter, readability-named-parameter)
inline IsStringOrCharArrayOrChar<T, string<Capacity>&> string<Capacity>::append(TruncateToCapacity_t,
                                                                                const T& str) noexcept
{
    const uint64_t tSize{internal::GetSize<T>::call(str)};
    const char* const tData{internal::GetData<T>::call(str)};
    uint64_t const clampedTSize{std::min(Capacity - m_rawstringSize, tSize)};

    std::memcpy(&(m_rawstring[m_rawstringSize]), tData, static_cast<size_t>(clampedTSize));
    if (tSize > clampedTSize)
    {
        IOX_LOG(Warn,
                "The last " << (tSize - clampedTSize) << " characters of " << tData
                            << " are truncated, because the length is larger than the capacity.");
    }

    m_rawstringSize += clampedTSize;
    m_rawstring[m_rawstringSize] = '\0';
    return *this;
}

template <uint64_t Capacity>
// TruncateToCapacity_t is a compile time variable to distinguish between constructors
// NOLINTNEXTLINE(hicpp-named-parameter, readability-named-parameter)
inline string<Capacity>& string<Capacity>::append(TruncateToCapacity_t, char cstr) noexcept
{
    if (m_rawstringSize == Capacity)
    {
        IOX_LOG(Warn,
                "Appending of " << static_cast<unsigned char>(cstr)
                                << " failed because this' capacity would be exceeded.");
        return *this;
    }
    m_rawstring[m_rawstringSize] = cstr;
    m_rawstringSize += 1U;
    m_rawstring[m_rawstringSize] = '\0';
    return *this;
}

template <uint64_t Capacity>
template <typename T>
inline IsIoxStringOrCharArray<T, bool>
string<Capacity>::insert(const uint64_t pos, const T& str, const uint64_t count) noexcept
{
    // AXIVION Next Construct AutosarC++19_03-M0.1.2, AutosarC++19_03-M0.1.9, FaultDetection-DeadBranches : False positive! Branching depends on input parameter
    if (count > internal::GetSize<T>::call(str))
    {
        return false;
    }
    const uint64_t new_size{m_rawstringSize + count};
    // check if the new size would exceed capacity or a size overflow occured
    if ((new_size > Capacity) || (new_size < m_rawstringSize))
    {
        return false;
    }

    // AXIVION Next Construct AutosarC++19_03-M0.1.2, AutosarC++19_03-M0.1.9, FaultDetection-DeadBranches : False positive! Branching depends on input parameter
    if (pos > m_rawstringSize)
    {
        return false;
    }
    auto number_of_characters_to_move = static_cast<size_t>(m_rawstringSize) - static_cast<size_t>(pos);
    std::memmove(&m_rawstring[pos + count], &m_rawstring[pos], number_of_characters_to_move);
    std::memcpy(&m_rawstring[pos], internal::GetData<T>::call(str), static_cast<size_t>(count));

    m_rawstring[new_size] = '\0';
    m_rawstringSize = new_size;

    return true;
}

template <uint64_t Capacity>
inline optional<string<Capacity>> string<Capacity>::substr(const uint64_t pos, const uint64_t count) const noexcept
{
    if (pos > m_rawstringSize)
    {
        return nullopt;
    }

    const uint64_t length{std::min(count, m_rawstringSize - pos)};
    string subString;
    std::memcpy(subString.m_rawstring, &m_rawstring[pos], static_cast<size_t>(length));
    subString.m_rawstring[length] = '\0';
    subString.m_rawstringSize = length;
    return subString;
}

template <uint64_t Capacity>
inline optional<string<Capacity>> string<Capacity>::substr(const uint64_t pos) const noexcept
{
    return substr(pos, m_rawstringSize);
}

template <uint64_t Capacity>
template <typename T>
inline IsStringOrCharArray<T, optional<uint64_t>> string<Capacity>::find(const T& str,
                                                                         const uint64_t pos) const noexcept
{
    if (pos > m_rawstringSize)
    {
        return nullopt;
    }
    const char* found{std::strstr(c_str() + pos, internal::GetData<T>::call(str))};
    if (found == nullptr)
    {
        return nullopt;
    }
    return static_cast<uint64_t>(found - c_str());
}

template <uint64_t Capacity>
template <typename T>
inline IsStringOrCharArray<T, optional<uint64_t>> string<Capacity>::find_first_of(const T& str,
                                                                                  const uint64_t pos) const noexcept
{
    // AXIVION Next Construct AutosarC++19_03-M0.1.2, AutosarC++19_03-M0.1.9, FaultDetection-DeadBranches : False positive! Branching depends on input parameter
    if (pos > m_rawstringSize)
    {
        return nullopt;
    }
    const char* const data{internal::GetData<T>::call(str)};
    const uint64_t dataSize{internal::GetSize<T>::call(str)};
    for (auto p = pos; p < m_rawstringSize; ++p)
    {
        const void* const found{memchr(data, m_rawstring[p], static_cast<size_t>(dataSize))};
        if (found != nullptr)
        {
            return p;
        }
    }
    return nullopt;
}

template <uint64_t Capacity>
template <typename T>
inline IsStringOrCharArray<T, optional<uint64_t>> string<Capacity>::find_last_of(const T& str,
                                                                                 const uint64_t pos) const noexcept
{
    if (m_rawstringSize == 0U)
    {
        return nullopt;
    }

    uint64_t p{pos};
    if ((m_rawstringSize - 1U) < p)
    {
        p = m_rawstringSize - 1U;
    }
    const char* const data{internal::GetData<T>::call(str)};
    const uint64_t dataSize{internal::GetSize<T>::call(str)};
    for (; p > 0U; --p)
    {
        const void* const found{memchr(data, m_rawstring[p], static_cast<size_t>(dataSize))};
        if (found != nullptr)
        {
            return p;
        }
    }
    const void* const found{memchr(data, m_rawstring[p], static_cast<size_t>(dataSize))};
    if (found != nullptr)
    {
        return 0U;
    }
    return nullopt;
}

template <uint64_t Capacity>
inline constexpr char& string<Capacity>::at(const uint64_t pos) noexcept
{
    // AXIVION Next Construct AutosarC++19_03-A5.2.3 : const_cast to avoid code duplication, safe since it's first
    // casted to a const type and then the const is removed
    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-const-cast)
    return const_cast<char&>(const_cast<const string<Capacity>*>(this)->at(pos));
}

template <uint64_t Capacity>
inline constexpr const char& string<Capacity>::at(const uint64_t pos) const noexcept
{
    IOX_ENFORCE((pos < size()), "Out of bounds access!");
    return m_rawstring[pos];
}

template <uint64_t Capacity>
inline constexpr char& string<Capacity>::unchecked_at(const uint64_t pos) noexcept
{
    // AXIVION Next Construct AutosarC++19_03-A5.2.3 : const_cast to avoid code duplication, safe since it's first
    // casted to a const type and then the const is removed
    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-const-cast)
    return const_cast<char&>(const_cast<const string<Capacity>*>(this)->unchecked_at(pos));
}

template <uint64_t Capacity>
inline constexpr const char& string<Capacity>::unchecked_at(const uint64_t pos) const noexcept
{
    return m_rawstring[pos];
}

template <uint64_t Capacity>
inline constexpr char& string<Capacity>::operator[](const uint64_t pos) noexcept
{
    return at(pos);
}

template <uint64_t Capacity>
inline constexpr const char& string<Capacity>::operator[](const uint64_t pos) const noexcept
{
    return at(pos);
}

// AXIVION DISABLE STYLE AutosarC++19_03-A13.5.5: Comparison with custom string, char array or
// char is also intended
template <typename T, uint64_t Capacity>
inline IsCustomStringOrCharArrayOrChar<T, bool> operator==(const T& lhs, const string<Capacity>& rhs) noexcept
{
    return (rhs.compare(lhs) == 0);
}

template <typename T, uint64_t Capacity>
inline IsCustomStringOrCharArrayOrChar<T, bool> operator!=(const T& lhs, const string<Capacity>& rhs) noexcept
{
    return (rhs.compare(lhs) != 0);
}

template <typename T, uint64_t Capacity>
inline IsCustomStringOrCharArrayOrChar<T, bool> operator<(const T& lhs, const string<Capacity>& rhs) noexcept
{
    return (rhs.compare(lhs) > 0);
}

template <typename T, uint64_t Capacity>
inline IsCustomStringOrCharArrayOrChar<T, bool> operator<=(const T& lhs, const string<Capacity>& rhs) noexcept
{
    return (rhs.compare(lhs) >= 0);
}

template <typename T, uint64_t Capacity>
inline IsCustomStringOrCharArrayOrChar<T, bool> operator>(const T& lhs, const string<Capacity>& rhs) noexcept
{
    return (rhs.compare(lhs) < 0);
}

template <typename T, uint64_t Capacity>
inline IsCustomStringOrCharArrayOrChar<T, bool> operator>=(const T& lhs, const string<Capacity>& rhs) noexcept
{
    return (rhs.compare(lhs) <= 0);
}

// AXIVION Next Construct AutosarC++19_03-A13.5.4 : Code reuse is established by a helper function
template <typename T, uint64_t Capacity>
inline IsStringOrCharArrayOrChar<T, bool> operator==(const string<Capacity>& lhs, const T& rhs) noexcept
{
    return (lhs.compare(rhs) == 0);
}

// AXIVION Next Construct AutosarC++19_03-A13.5.4 : Code reuse is established by a helper function
template <typename T, uint64_t Capacity>
inline IsStringOrCharArrayOrChar<T, bool> operator!=(const string<Capacity>& lhs, const T& rhs) noexcept
{
    return (lhs.compare(rhs) != 0);
}

template <typename T, uint64_t Capacity>
inline IsStringOrCharArrayOrChar<T, bool> operator<(const string<Capacity>& lhs, const T& rhs) noexcept
{
    return (lhs.compare(rhs) < 0);
}

template <typename T, uint64_t Capacity>
inline IsStringOrCharArrayOrChar<T, bool> operator<=(const string<Capacity>& lhs, const T& rhs) noexcept
{
    return (lhs.compare(rhs) <= 0);
}

template <typename T, uint64_t Capacity>
inline IsStringOrCharArrayOrChar<T, bool> operator>(const string<Capacity>& lhs, const T& rhs) noexcept
{
    return (lhs.compare(rhs) > 0);
}

template <typename T, uint64_t Capacity>
inline IsStringOrCharArrayOrChar<T, bool> operator>=(const string<Capacity>& lhs, const T& rhs) noexcept
{
    return (lhs.compare(rhs) >= 0);
}
// AXIVION ENABLE Style AutosarC++19_03-A13.5.5
} // namespace iox

#endif // IOX_HOOFS_VOCABULARY_STRING_INL
