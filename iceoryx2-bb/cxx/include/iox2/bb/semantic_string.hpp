// Copyright (c) 2023 by Apex.AI Inc. All rights reserved.
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

#ifndef IOX2_BB_SEMANTIC_STRING_HPP
#define IOX2_BB_SEMANTIC_STRING_HPP

#include "iox2/bb/expected.hpp"
#include "iox2/bb/static_string.hpp"
#include "iox2/legacy/logging.hpp"

#include <cstdint>

namespace iox2 {
namespace bb {
/// @brief Defines errors which can occur when modifying or creating a SemanticString
enum class SemanticStringError : uint8_t {
    InvalidContent,
    ExceedsMaximumLength
};

template <uint64_t Capacity>
using DoesContainInvalidCharacter = bool (*)(const bb::StaticString<Capacity>& value);

template <uint64_t Capacity>
using DoesContainInvalidContent = bool (*)(const bb::StaticString<Capacity>& value);

/// @brief The SemanticString is a string which has an inner syntax and restrictions
///         to valid content. Examples are for instance
///         * FileNames, no slashes etc.
///        The SemanticString shall be a basic building block to create string
///        types with a semantic contract. It is used via inheritance. The user
///        has to also define the maximum capacity, a callable which defines
///        invalid characters as well as a callable which defines invalid content.
/// @code
/// bool file_name_does_contain_invalid_characters(const StaticString<platform::IOX2_MAX_FILENAME_LENGTH>& value)
/// noexcept;
/// bool file_name_does_contain_invalid_content(const StaticString<platform::IOX2_MAX_FILENAME_LENGTH>& value) noexcept;
///
/// // define custom semantic string FileName
/// class FileName : public SemanticString<FileName,
///                                        platform::IOX2_MAX_FILENAME_LENGTH,
///                                        details::file_name_does_contain_invalid_content,
///                                        details::file_name_does_contain_invalid_characters>
/// {
///     using Parent = SemanticString<FileName,
///                                  platform::IOX2_MAX_FILENAME_LENGTH,
///                                  details::file_name_does_contain_invalid_content,
///                                  details::file_name_does_contain_invalid_characters>;
///     using Parent::Parent;
/// };
/// @endcode
/// @note Since the inner logic of the SemanticString is always the same additional
///         implementations can be verified directly by the test suite defined in
///         'test_vocabulary_semantic_string.cpp'.
///         One has to only add the specific implementation to the 'Implementations'
///         type list.
template <typename Child,
          uint64_t Capacity,
          DoesContainInvalidContent<Capacity> DoesContainInvalidContentCall,
          DoesContainInvalidCharacter<Capacity> DoesContainInvalidCharacterCall>
class SemanticString {
  private:
    bb::StaticString<Capacity> m_data;

  public:
    /// @brief Creates a new SemanticString from the provided string literal.
    ///         If the value contains invalid characters or invalid content
    ///         the expected returns an error describing the cause.
    /// @param[in] value the value of the SemanticString
    /// @return expected either containing the new SemanticString or an error
    template <uint64_t N>
    // avoid-c-arrays: we would like to assign string literals, safe since it is known
    //                 at compile time.
    // NOLINTNEXTLINE(hicpp-avoid-c-arrays, cppcoreguidelines-avoid-c-arrays, hicpp-explicit-conversions, modernize-avoid-c-arrays)
    static auto create(const char (&value)[N]) noexcept -> bb::Expected<Child, SemanticStringError>;

    /// @brief Creates a new SemanticString from the provided string.
    ///         If the value contains invalid characters or invalid content
    ///         the expected returns an error describing the cause.
    /// @param[in] value the value of the SemanticString
    /// @return expected either containing the new SemanticString or an error
    template <uint64_t N>
    static auto create(const bb::StaticString<N>& value) noexcept -> bb::Expected<Child, SemanticStringError>;

    /// @brief Returns the number of characters.
    /// @return number of characters
    constexpr auto size() const noexcept -> uint64_t;

    /// @brief Returns the capacity of the string.
    /// @return the maximum amount of characters which can be stored.
    static constexpr auto capacity() noexcept -> uint64_t;

    /// @brief Returns a const reference to the underlying string. It is const
    ///         and shall not be modified to guarantee the contract that a
    ///         SemanticString contains always a valid value.
    /// @return string reference containing the actual value.
    constexpr auto as_string() const noexcept -> const bb::StaticString<Capacity>&;

    /// @brief Appends another string to the SemanticString. If the value contains
    ///        invalid characters or the result would end up in invalid content
    ///        it fails.
    /// @param[in] value the value which should be added
    /// @return on failure the error inside the expected describes the failure
    template <typename T>
    auto append(const T& value) noexcept -> bb::Expected<void, SemanticStringError>;

    /// @brief Inserts another string into the SemanticString. If the value contains
    ///        invalid characters or the result would end up in invalid content
    ///        it fails.
    /// @param[in] pos the position where the string should be inserted
    /// @param[in] str the value which should be added
    /// @param[in] count how many characters of str shall be inserted
    /// @return on failure the error inside the expected describes the failure
    template <typename T>
    auto insert(uint64_t pos, const T& str, uint64_t count) noexcept -> bb::Expected<void, SemanticStringError>;

    /// @brief checks if another SemanticString is equal to this string
    /// @param [in] rhs the other SemanticString
    /// @return true if the contents are equal, otherwise false
    auto operator==(const SemanticString& rhs) const noexcept -> bool;

    /// @brief checks if another string or char array is equal to this string
    /// @param [in] rhs the other string
    /// @return true if the contents are equal, otherwise false
    template <typename T>
    auto operator==(const T& rhs) const noexcept -> bb::RequireStaticStringOrCharArray<T, bool>;

    /// @brief checks if another SemanticString is not equal to this string
    /// @param [in] rhs the other SemanticString
    /// @return true if the contents are not equal, otherwise false
    auto operator!=(const SemanticString& rhs) const noexcept -> bool;

    /// @brief checks if another string or char array is not equal to this string
    /// @param [in] rhs the other string
    /// @return true if the contents are not equal, otherwise false
    template <typename T>
    auto operator!=(const T& rhs) const noexcept -> bb::RequireStaticStringOrCharArray<T, bool>;

    /// @brief checks if another SemanticString is less than or equal this string
    /// @param [in] rhs the other SemanticString
    /// @return true if the contents are less than or equal rhs, otherwise false
    auto operator<=(const SemanticString& rhs) const noexcept -> bool;

    /// @brief checks if another string or char array is less than or equal this string
    /// @param [in] rhs the other string
    /// @return true if the contents are less than or equal rhs, otherwise false
    template <typename T>
    auto operator<=(const T& rhs) const noexcept -> bb::RequireStaticStringOrCharArray<T, bool>;

    /// @brief checks if another SemanticString is less than this string
    /// @param [in] rhs the other SemanticString
    /// @return true if the contents are less than rhs, otherwise false
    auto operator<(const SemanticString& rhs) const noexcept -> bool;

    /// @brief checks if another string or char array is less than this string
    /// @param [in] rhs the other string
    /// @return true if the contents are less than rhs, otherwise false
    template <typename T>
    auto operator<(const T& rhs) const noexcept -> bb::RequireStaticStringOrCharArray<T, bool>;

    /// @brief checks if another SemanticString is greater than or equal this string
    /// @param [in] rhs the other SemanticString
    /// @return true if the contents are greater than or equal rhs, otherwise false
    auto operator>=(const SemanticString& rhs) const noexcept -> bool;

    /// @brief checks if another string or char array is greater than or equal this string
    /// @param [in] rhs the other string
    /// @return true if the contents are greater than or equal rhs, otherwise false
    template <typename T>
    auto operator>=(const T& rhs) const noexcept -> bb::RequireStaticStringOrCharArray<T, bool>;

    /// @brief checks if another SemanticString is greater than this string
    /// @param [in] rhs the other SemanticString
    /// @return true if the contents are greater than rhs, otherwise false
    auto operator>(const SemanticString& rhs) const noexcept -> bool;

    /// @brief checks if another string or char array is greater than this string
    /// @param [in] rhs the other string
    /// @return true if the contents are greater than rhs, otherwise false
    template <typename T>
    auto operator>(const T& rhs) const noexcept -> bb::RequireStaticStringOrCharArray<T, bool>;

  protected:
    template <uint64_t N>
    explicit SemanticString(const bb::StaticString<N>& value) noexcept;

  private:
    template <uint64_t N>
    static auto create_impl(const char* value) noexcept -> bb::Expected<Child, SemanticStringError>;
};


template <typename Child,
          uint64_t Capacity,
          DoesContainInvalidContent<Capacity> DoesContainInvalidContentCall,
          DoesContainInvalidCharacter<Capacity> DoesContainInvalidCharacterCall>
template <uint64_t N>
inline SemanticString<Child, Capacity, DoesContainInvalidContentCall, DoesContainInvalidCharacterCall>::SemanticString(
    const bb::StaticString<N>& value) noexcept
    : m_data { value } {
}

template <typename Child,
          uint64_t Capacity,
          DoesContainInvalidContent<Capacity> DoesContainInvalidContentCall,
          DoesContainInvalidCharacter<Capacity> DoesContainInvalidCharacterCall>
template <uint64_t N>
inline auto
SemanticString<Child, Capacity, DoesContainInvalidContentCall, DoesContainInvalidCharacterCall>::create_impl(
    const char* value) noexcept -> bb::Expected<Child, SemanticStringError> {
    if (N > Capacity) {
        IOX2_LOG(Debug,
                 "Unable to create semantic string since the value \""
                     << value << "\" exceeds the maximum valid length of " << Capacity << ".");
        return bb::err(SemanticStringError::ExceedsMaximumLength);
    }

    auto str = bb::StaticString<Capacity>::from_utf8_null_terminated_unchecked_truncated(value, N);

    if (DoesContainInvalidCharacterCall(str)) {
        IOX2_LOG(Debug,
                 "Unable to create semantic string since the value \"" << value
                                                                       << "\" contains invalid characters as content");
        return bb::err(SemanticStringError::InvalidContent);
    }

    if (DoesContainInvalidContentCall(str)) {
        IOX2_LOG(Debug,
                 "Unable to create semantic string since the value \"" << value << "\" contains invalid content");
        return bb::err(SemanticStringError::InvalidContent);
    }

    return Child(str);
}

template <typename Child,
          uint64_t Capacity,
          DoesContainInvalidContent<Capacity> DoesContainInvalidContentCall,
          DoesContainInvalidCharacter<Capacity> DoesContainInvalidCharacterCall>
template <uint64_t N>
inline auto SemanticString<Child, Capacity, DoesContainInvalidContentCall, DoesContainInvalidCharacterCall>::create(
    // avoid-c-arrays: justification in header
    // NOLINTNEXTLINE(hicpp-avoid-c-arrays, cppcoreguidelines-avoid-c-arrays, hicpp-explicit-conversions, modernize-avoid-c-arrays)
    const char (&value)[N]) noexcept -> bb::Expected<Child, SemanticStringError> {
    return SemanticString<Child, Capacity, DoesContainInvalidContentCall, DoesContainInvalidCharacterCall>::
        template create_impl<N>(value);
}

template <typename Child,
          uint64_t Capacity,
          DoesContainInvalidContent<Capacity> DoesContainInvalidContentCall,
          DoesContainInvalidCharacter<Capacity> DoesContainInvalidCharacterCall>
template <uint64_t N>
inline auto SemanticString<Child, Capacity, DoesContainInvalidContentCall, DoesContainInvalidCharacterCall>::create(
    const bb::StaticString<N>& value) noexcept -> bb::Expected<Child, SemanticStringError> {
    return SemanticString<Child, Capacity, DoesContainInvalidContentCall, DoesContainInvalidCharacterCall>::
        template create_impl<N>(value.unchecked_access().c_str());
}

template <typename Child,
          uint64_t Capacity,
          DoesContainInvalidContent<Capacity> DoesContainInvalidContentCall,
          DoesContainInvalidCharacter<Capacity> DoesContainInvalidCharacterCall>
constexpr auto
SemanticString<Child, Capacity, DoesContainInvalidContentCall, DoesContainInvalidCharacterCall>::size() const noexcept
    -> uint64_t {
    return m_data.size();
}

template <typename Child,
          uint64_t Capacity,
          DoesContainInvalidContent<Capacity> DoesContainInvalidContentCall,
          DoesContainInvalidCharacter<Capacity> DoesContainInvalidCharacterCall>
constexpr auto
SemanticString<Child, Capacity, DoesContainInvalidContentCall, DoesContainInvalidCharacterCall>::capacity() noexcept
    -> uint64_t {
    return Capacity;
}

template <typename Child,
          uint64_t Capacity,
          DoesContainInvalidContent<Capacity> DoesContainInvalidContentCall,
          DoesContainInvalidCharacter<Capacity> DoesContainInvalidCharacterCall>
constexpr auto
SemanticString<Child, Capacity, DoesContainInvalidContentCall, DoesContainInvalidCharacterCall>::as_string()
    const noexcept -> const bb::StaticString<Capacity>& {
    return m_data;
}

template <typename Child,
          uint64_t Capacity,
          DoesContainInvalidContent<Capacity> DoesContainInvalidContentCall,
          DoesContainInvalidCharacter<Capacity> DoesContainInvalidCharacterCall>
template <typename T>
inline auto SemanticString<Child, Capacity, DoesContainInvalidContentCall, DoesContainInvalidCharacterCall>::append(
    const T& value) noexcept -> bb::Expected<void, SemanticStringError> {
    return insert(size(), value, bb::detail::get_size(value));
}

template <typename Child,
          uint64_t Capacity,
          DoesContainInvalidContent<Capacity> DoesContainInvalidContentCall,
          DoesContainInvalidCharacter<Capacity> DoesContainInvalidCharacterCall>
template <typename T>
inline auto SemanticString<Child, Capacity, DoesContainInvalidContentCall, DoesContainInvalidCharacterCall>::insert(
    const uint64_t pos, const T& str, const uint64_t count) noexcept -> bb::Expected<void, SemanticStringError> {
    auto temp = m_data;
    if (!temp.unchecked_code_units().insert(pos, str, 0, count)) {
        IOX2_LOG(Debug,
                 "Unable to insert the value \""
                     << str.unchecked_access().c_str()
                     << "\" to the semantic string since it would exceed the maximum valid length of " << Capacity
                     << ".");
        return bb::err(SemanticStringError::ExceedsMaximumLength);
    }

    if (DoesContainInvalidCharacterCall(temp)) {
        IOX2_LOG(Debug,
                 "Unable to insert the value \""
                     << str.unchecked_access().c_str()
                     << "\" to the semantic string since it contains invalid characters as content.");
        return bb::err(SemanticStringError::InvalidContent);
    }

    if (DoesContainInvalidContentCall(temp)) {
        IOX2_LOG(Debug,
                 "Unable to insert the value \""
                     << str.unchecked_access().c_str()
                     << "\" to the semantic string since it would lead to invalid content.");
        return bb::err(SemanticStringError::InvalidContent);
    }

    m_data = temp;
    return {};
}

template <typename Child,
          uint64_t Capacity,
          DoesContainInvalidContent<Capacity> DoesContainInvalidContentCall,
          DoesContainInvalidCharacter<Capacity> DoesContainInvalidCharacterCall>
inline auto SemanticString<Child, Capacity, DoesContainInvalidContentCall, DoesContainInvalidCharacterCall>::operator==(
    const SemanticString& rhs) const noexcept -> bool {
    return as_string() == rhs.as_string();
}

template <typename Child,
          uint64_t Capacity,
          DoesContainInvalidContent<Capacity> DoesContainInvalidContentCall,
          DoesContainInvalidCharacter<Capacity> DoesContainInvalidCharacterCall>
template <typename T>
inline auto SemanticString<Child, Capacity, DoesContainInvalidContentCall, DoesContainInvalidCharacterCall>::operator==(
    const T& rhs) const noexcept -> bb::RequireStaticStringOrCharArray<T, bool> {
    return as_string() == rhs;
}

template <typename Child,
          uint64_t Capacity,
          DoesContainInvalidContent<Capacity> DoesContainInvalidContentCall,
          DoesContainInvalidCharacter<Capacity> DoesContainInvalidCharacterCall>
inline auto SemanticString<Child, Capacity, DoesContainInvalidContentCall, DoesContainInvalidCharacterCall>::operator!=(
    const SemanticString& rhs) const noexcept -> bool {
    return as_string() != rhs.as_string();
}

template <typename Child,
          uint64_t Capacity,
          DoesContainInvalidContent<Capacity> DoesContainInvalidContentCall,
          DoesContainInvalidCharacter<Capacity> DoesContainInvalidCharacterCall>
template <typename T>
inline auto SemanticString<Child, Capacity, DoesContainInvalidContentCall, DoesContainInvalidCharacterCall>::operator!=(
    const T& rhs) const noexcept -> bb::RequireStaticStringOrCharArray<T, bool> {
    return as_string() != rhs;
}

template <typename Child,
          uint64_t Capacity,
          DoesContainInvalidContent<Capacity> DoesContainInvalidContentCall,
          DoesContainInvalidCharacter<Capacity> DoesContainInvalidCharacterCall>
inline auto SemanticString<Child, Capacity, DoesContainInvalidContentCall, DoesContainInvalidCharacterCall>::operator<=(
    const SemanticString& rhs) const noexcept -> bool {
    return as_string() <= rhs.as_string();
}

template <typename Child,
          uint64_t Capacity,
          DoesContainInvalidContent<Capacity> DoesContainInvalidContentCall,
          DoesContainInvalidCharacter<Capacity> DoesContainInvalidCharacterCall>
template <typename T>
inline auto SemanticString<Child, Capacity, DoesContainInvalidContentCall, DoesContainInvalidCharacterCall>::operator<=(
    const T& rhs) const noexcept -> bb::RequireStaticStringOrCharArray<T, bool> {
    return as_string() <= rhs;
}

template <typename Child,
          uint64_t Capacity,
          DoesContainInvalidContent<Capacity> DoesContainInvalidContentCall,
          DoesContainInvalidCharacter<Capacity> DoesContainInvalidCharacterCall>
inline auto SemanticString<Child, Capacity, DoesContainInvalidContentCall, DoesContainInvalidCharacterCall>::operator<(
    const SemanticString& rhs) const noexcept -> bool {
    return as_string() < rhs.as_string();
}

template <typename Child,
          uint64_t Capacity,
          DoesContainInvalidContent<Capacity> DoesContainInvalidContentCall,
          DoesContainInvalidCharacter<Capacity> DoesContainInvalidCharacterCall>
template <typename T>
inline auto SemanticString<Child, Capacity, DoesContainInvalidContentCall, DoesContainInvalidCharacterCall>::operator<(
    const T& rhs) const noexcept -> bb::RequireStaticStringOrCharArray<T, bool> {
    return as_string() < rhs;
}

template <typename Child,
          uint64_t Capacity,
          DoesContainInvalidContent<Capacity> DoesContainInvalidContentCall,
          DoesContainInvalidCharacter<Capacity> DoesContainInvalidCharacterCall>
inline auto SemanticString<Child, Capacity, DoesContainInvalidContentCall, DoesContainInvalidCharacterCall>::operator>=(
    const SemanticString& rhs) const noexcept -> bool {
    return as_string() >= rhs.as_string();
}

template <typename Child,
          uint64_t Capacity,
          DoesContainInvalidContent<Capacity> DoesContainInvalidContentCall,
          DoesContainInvalidCharacter<Capacity> DoesContainInvalidCharacterCall>
template <typename T>
inline auto SemanticString<Child, Capacity, DoesContainInvalidContentCall, DoesContainInvalidCharacterCall>::operator>=(
    const T& rhs) const noexcept -> bb::RequireStaticStringOrCharArray<T, bool> {
    return as_string() >= rhs;
}

template <typename Child,
          uint64_t Capacity,
          DoesContainInvalidContent<Capacity> DoesContainInvalidContentCall,
          DoesContainInvalidCharacter<Capacity> DoesContainInvalidCharacterCall>
inline auto SemanticString<Child, Capacity, DoesContainInvalidContentCall, DoesContainInvalidCharacterCall>::operator>(
    const SemanticString& rhs) const noexcept -> bool {
    return as_string() > rhs.as_string();
}

template <typename Child,
          uint64_t Capacity,
          DoesContainInvalidContent<Capacity> DoesContainInvalidContentCall,
          DoesContainInvalidCharacter<Capacity> DoesContainInvalidCharacterCall>
template <typename T>
inline auto SemanticString<Child, Capacity, DoesContainInvalidContentCall, DoesContainInvalidCharacterCall>::operator>(
    const T& rhs) const noexcept -> bb::RequireStaticStringOrCharArray<T, bool> {
    return as_string() > rhs;
}

} // namespace bb
} // namespace iox2

#endif // IOX2_BB_SEMANTIC_STRING_HPP
