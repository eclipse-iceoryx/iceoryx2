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

#ifndef IOX2_INCLUDE_GUARD_BB_STATIC_STRING_HPP
#define IOX2_INCLUDE_GUARD_BB_STATIC_STRING_HPP

#include "iox2/bb/detail/string_internal.hpp"
#include "iox2/bb/optional.hpp"
#include "iox2/legacy/type_traits.hpp"

#include <algorithm>
#include <cstddef>
#include <cstdint>
#include <cstring>
#include <functional>
#include <ostream>
#include <type_traits>

namespace iox2 {
namespace bb {
template <uint64_t Capacity>
using DoesContainInvalidCharacter = bool (*)(const StaticString<Capacity>& value);

template <uint64_t Capacity>
using DoesContainInvalidContent = bool (*)(const StaticString<Capacity>& value);

template <typename, uint64_t Capacity, DoesContainInvalidContent<Capacity>, DoesContainInvalidCharacter<Capacity>>
class SemanticString;

namespace detail {
/// @brief verifies if the given string is a valid path to a file
/// @param[in] name the string to verify
/// @return true if the string is a path to a file, otherwise false
template <uint64_t StringCapacity>
auto is_valid_path_to_file(const StaticString<StringCapacity>& name) noexcept -> bool;

/// @brief returns true if the provided name is a valid path, otherwise false
/// @param[in] name the string to verify
template <uint64_t StringCapacity>
auto is_valid_path_to_directory(const StaticString<StringCapacity>& name) noexcept -> bool;

} // namespace detail

template <uint64_t>
class StaticString;

template <typename>
struct IsStaticString : std::false_type { };

template <uint64_t N>
struct IsStaticString<StaticString<N>> : std::true_type { };

template <typename T, typename ReturnType>
using RequireStaticStringOrCharArray =
    typename std::enable_if_t<IsStaticString<T>::value || legacy::is_char_array<T>::value, ReturnType>;

/// A UTF-8 string with fixed static capacity and contiguous inplace storage.
/// The string class uses Unicode (ISO/IEC 10646) terminology throughout its interface. In particular:
/// - A code point is the numerical index assigned to a character in the Unicode standard.
/// - A code unit is the basic component of a character encoding system. For UTF-8, the code unit has a size of 8-bits
/// For example, the code point U+0041 represents the letter 'A' and can be encoded in a single 8-bit code unit in
/// UTF-8. The code point U+1F4A9 requires four 8-bit code units in the UTF-8 encoding.
///
/// @attention The NUL code point (U+0000) is not allowed anywhere in the string.
/// @note Currently only Unicode code points less than 128 (U+0080) are supported.
///       This restricts the valid contents of a string to those UTF8 strings
///       that are also valid 7-bit ASCII strings. Full Unicode support will get added later.
/// @tparam N Maximum number of UTF-8 code units that the string can store, excluding the terminating NUL character.
template <uint64_t N>
class StaticString {
  public:
    using ValueType = char;
    using CodeUnitValueType = char;
    using CodePointValueType = char32_t;
    using SizeType = size_t;
    using DifferenceType = ptrdiff_t;
    using Reference = char&;
    using ConstReference = char const&;
    using Pointer = char*;
    using ConstPointer = char const*;
    using Iterator = Pointer;
    using ConstIterator = ConstPointer;
    using OptionalReference = bb::Optional<std::reference_wrapper<char>>;
    using OptionalConstReference = bb::Optional<std::reference_wrapper<char const>>;
    using OptionalCodeUnitReference = bb::Optional<std::reference_wrapper<CodeUnitValueType>>;
    using OptionalConstCodeUnitReference = bb::Optional<std::reference_wrapper<CodeUnitValueType const>>;

    /// The unchecked API provided by this class allows for uncontrolled memory access.
    /// Users of this class must ensure that all memory accesses stay within bounds of the accessed string memory.
    class UncheckedConstAccessor {
        friend class StaticString;

      private:
        StaticString const* m_parent;

        constexpr explicit UncheckedConstAccessor(StaticString const& parent)
            : m_parent(&parent) {
        }

      public:
        ~UncheckedConstAccessor() = default;
        UncheckedConstAccessor(UncheckedConstAccessor const&) = delete;
        // NOTE: can be changed to '= delete' when C++17 becomes mandatory and we can rely on RVO
        UncheckedConstAccessor(UncheckedConstAccessor&&) = default;
        auto operator=(UncheckedConstAccessor const&) -> UncheckedConstAccessor& = delete;
        auto operator=(UncheckedConstAccessor&&) -> UncheckedConstAccessor& = delete;

        constexpr auto operator[](SizeType index) const -> ConstReference {
            return m_parent->m_string[index];
        }

        constexpr auto begin() const noexcept -> ConstIterator {
            return &(m_parent->m_string[0]);
        }

        constexpr auto end() const noexcept -> ConstIterator {
            return &(m_parent->m_string[m_parent->m_size]);
        }

        constexpr auto data() const noexcept -> ConstPointer {
            return &(m_parent->m_string[0]);
        }

        constexpr auto c_str() const noexcept -> char const* {
            return data();
        }
    };

    /// The unchecked API provided by this class allows for uncontrolled memory access and encoding violations.
    /// Users of this class must ensure that all memory accesses stay within bounds of the accessed string memory.
    /// Users of this class must ensure that writes to the string do not result in a sequence of bytes that is no longer
    /// a valid UTF-8 encoded string. This includes not setting any of the string characters to NUL (U+0000).
    class UncheckedAccessor {
        friend class StaticString;

      private:
        StaticString* m_parent;

        constexpr explicit UncheckedAccessor(StaticString& parent)
            : m_parent(&parent) {
        }

      public:
        ~UncheckedAccessor() = default;
        UncheckedAccessor(UncheckedAccessor const&) = delete;
        // NOTE: can be changed to '= delete' when C++17 becomes mandatory and we can rely on RVO
        UncheckedAccessor(UncheckedAccessor&&) = default;
        auto operator=(UncheckedAccessor const&) -> UncheckedAccessor& = delete;
        auto operator=(UncheckedAccessor&&) -> UncheckedAccessor& = delete;

        constexpr auto operator[](SizeType index) -> Reference {
            return m_parent->m_string[index];
        }

        constexpr auto begin() noexcept -> Iterator {
            return &(m_parent->m_string[0]);
        }

        constexpr auto end() noexcept -> Iterator {
            return &(m_parent->m_string[m_parent->m_size]);
        }

        constexpr auto data() noexcept -> Pointer {
            return &(m_parent->m_string[0]);
        }

        constexpr auto c_str() noexcept -> char const* {
            return data();
        }
    };

    /// The unchecked API provided by this class allows for encoding violations.
    /// Users of this class must ensure that writes to the string do not result in a sequence of bytes that is no longer
    /// a valid UTF-8 encoded string. This includes not setting any of the string characters to NUL (U+0000).
    class UncheckedAccessorCodeUnits {
        friend class StaticString;
        template <typename,
                  uint64_t Capacity,
                  bb::DoesContainInvalidContent<Capacity>,
                  bb::DoesContainInvalidCharacter<Capacity>>
        friend class bb::SemanticString;

      private:
        StaticString* m_parent;

        constexpr explicit UncheckedAccessorCodeUnits(StaticString& parent)
            : m_parent(&parent) {
        }

        /// Inserts a StaticString, obtained by str.substr(s_index, count), into the StaticString at position index. The
        /// insertion fails if the capacity would be exceeded or the provided indices are larger than the respective
        /// string lengths.
        ///
        /// @return true if the insertion was successful, otherwise false
        template <typename T>
        auto insert(SizeType index, T const& str, SizeType s_index, SizeType count = T::capacity()) ->
            typename std::enable_if_t<IsStaticString<T>::value, bool> {
            auto sub_str = str.code_unit_based_substr(s_index, count);
            if (!sub_str.has_value()) {
                return false;
            }

            auto const sub_str_size = sub_str->size();
            auto const new_size = m_parent->m_size + sub_str_size;
            // check if the new size would exceed capacity or a size overflow occured
            if (new_size > N || new_size < m_parent->m_size) {
                return false;
            }

            if (index > m_parent->m_size) {
                return false;
            }
            std::copy_backward(
                &m_parent->m_string[index], &m_parent->m_string[m_parent->m_size], &m_parent->m_string[new_size]);
            std::copy(&sub_str->m_string[0], &sub_str->m_string[sub_str_size], &m_parent->m_string[index]);

            m_parent->m_string[new_size] = '\0';
            m_parent->m_size = new_size;

            return true;
        }

      public:
        ~UncheckedAccessorCodeUnits() = default;
        UncheckedAccessorCodeUnits(UncheckedAccessorCodeUnits const&) = delete;
        // NOTE: can be changed to '= delete' when C++17 becomes mandatory and we can rely on RVO
        UncheckedAccessorCodeUnits(UncheckedAccessorCodeUnits&&) = default;
        auto operator=(UncheckedAccessorCodeUnits const&) -> UncheckedAccessorCodeUnits& = delete;
        auto operator=(UncheckedAccessorCodeUnits&&) -> UncheckedAccessorCodeUnits& = delete;

        /// Retrieve a reference to the single code unit at `index`.
        /// @return A reference to the code unit or `NULLOPT` if the index is out of bounds.
        auto element_at(SizeType index) noexcept -> OptionalCodeUnitReference {
            if (index < m_parent->m_size) {
                return m_parent->m_string[index];
            } else {
                return bb::NULLOPT;
            }
        }

        /// Retrieve a reference to the first code unit at the beginning of the string.
        /// @return A reference to the front code unit or `NULLOPT` if the string is empty.
        auto front_element() noexcept -> OptionalCodeUnitReference {
            if (!m_parent->empty()) {
                return m_parent->m_string[0];
            } else {
                return bb::NULLOPT;
            }
        }

        /// Retrieve a reference to the last code unit at the end of the string.
        /// @return A reference to the back code unit or `NULLOPT` if the string is empty.
        auto back_element() noexcept -> OptionalCodeUnitReference {
            if (!m_parent->empty()) {
                return m_parent->m_string[m_parent->size() - 1];
            } else {
                return bb::NULLOPT;
            }
        }

        /// Removes a single code unit at `index`.
        auto try_erase_at(SizeType index) noexcept -> bool {
            return try_erase_at(index, index + 1);
        }

        /// Removes the range of code units at [`begin_index`, `end_index`).
        auto try_erase_at(SizeType begin_index, SizeType end_index) noexcept -> bool {
            if ((begin_index <= end_index) && (end_index <= m_parent->m_size)) {
                auto const range_size = end_index - begin_index;
                char* const string_end = std::end(m_parent->m_string);
                std::move(&m_parent->m_string[end_index], string_end, &m_parent->m_string[begin_index]);
                std::fill(&m_parent->m_string[m_parent->m_size - range_size], string_end, '\0');
                m_parent->m_size -= range_size;
                return true;
            } else {
                return false;
            }
        }
    };

    /// This class provides the interface for accessing individual code units of the string.
    class ConstAccessorCodeUnits {
        friend class StaticString;
        friend auto bb::detail::is_valid_path_to_file<N>(const bb::StaticString<N>& name) noexcept -> bool;
        friend auto bb::detail::is_valid_path_to_directory<N>(const bb::StaticString<N>& name) noexcept -> bool;

      private:
        StaticString const* m_parent;

        constexpr explicit ConstAccessorCodeUnits(StaticString const& parent)
            : m_parent(&parent) {
        }

        /// Creates a substring containing the characters from pos until count; if pos+count is greater than the
        /// size of the original string the returned substring only contains the characters from pos until size().
        ///
        /// @return an Optional containing the substring
        ///         NULLOPT if pos is greater than the size of the original string
        auto substr(SizeType pos, SizeType count) const -> bb::Optional<StaticString> {
            return m_parent->code_unit_based_substr(pos, count);
        }

        /// Finds the first occurence of a character equal to one of the characters of the given character sequence
        /// and returns its position.
        ///
        /// @return an Optional containing the position of the first character equal to one of the characters of the
        ///         given character sequence
        ///         NULLOPT if no character is found or if pos is greater than size()
        template <typename T>
        auto find_first_of(T const& str, SizeType pos = 0U) const
            -> RequireStaticStringOrCharArray<T, bb::Optional<SizeType>> {
            if (pos > m_parent->m_size) {
                return bb::NULLOPT;
            }

            auto str_data = detail::get_data(str);
            auto str_size = detail::get_size(str);
            for (auto position = pos; position < m_parent->m_size; ++position) {
                auto found = memchr(str_data, m_parent->m_string[position], static_cast<size_t>(str_size));
                if (found != nullptr) {
                    return position;
                }
            }
            return bb::NULLOPT;
        }

        /// Finds the last occurence of a character equal to one of the characters of the given character sequence
        /// and returns its position.
        ///
        /// @return an Optional containing the position of the last character equal to one of the characters of the
        ///         given character sequence
        ///         NULLOPT if no character is found
        template <typename T>
        auto find_last_of(T const& str, SizeType pos = N) const
            -> RequireStaticStringOrCharArray<T, bb::Optional<SizeType>> {
            if (m_parent->m_size == 0) {
                return bb::NULLOPT;
            }

            auto position = std::min(static_cast<uint64_t>(pos), m_parent->m_size - 1);
            auto str_data = detail::get_data(str);
            auto str_size = detail::get_size(str);
            for (; position > 0; --position) {
                auto found = memchr(str_data, m_parent->m_string[position], str_size);
                if (found != nullptr) {
                    return position;
                }
            }
            auto found = memchr(str_data, m_parent->m_string[0], static_cast<size_t>(str_size));
            if (found != nullptr) {
                return 0U;
            }
            return bb::NULLOPT;
        }

      public:
        ~ConstAccessorCodeUnits() = default;
        ConstAccessorCodeUnits(ConstAccessorCodeUnits const&) = delete;
        // NOTE: can be changed to '= delete' when C++17 becomes mandatory and we can rely on RVO
        ConstAccessorCodeUnits(ConstAccessorCodeUnits&&) = default;
        auto operator=(ConstAccessorCodeUnits const&) -> ConstAccessorCodeUnits& = delete;
        auto operator=(ConstAccessorCodeUnits&&) -> ConstAccessorCodeUnits& = delete;

        /// Retrieve the single code unit at `index`.
        /// @return A reference to the code unit or `NULLOPT` if the index is out of bounds.
        auto element_at(SizeType index) const noexcept -> OptionalConstCodeUnitReference {
            if (index < m_parent->m_size) {
                return m_parent->m_string[index];
            } else {
                return bb::NULLOPT;
            }
        }

        /// Retrieve the first code unit at the beginning of the string.
        /// @return A reference to the front code unit or `NULLOPT` if the string is empty.
        auto front_element() const noexcept -> OptionalConstCodeUnitReference {
            if (!m_parent->empty()) {
                return m_parent->m_string[0];
            } else {
                return bb::NULLOPT;
            }
        }

        /// Retrieve the last code unit at the end of the string.
        /// @return A reference to the back code unit or `NULLOPT` if the string is empty.
        auto back_element() const noexcept -> OptionalConstCodeUnitReference {
            if (!m_parent->empty()) {
                return m_parent->m_string[m_parent->size() - 1];
            } else {
                return bb::NULLOPT;
            }
        }
    };

  private:
    template <uint64_t>
    friend class StaticString;

    // NOLINTNEXTLINE(cppcoreguidelines-avoid-c-arrays,hicpp-avoid-c-arrays,modernize-avoid-c-arrays) encapsulated storage
    char m_string[N + 1] = {};
    uint64_t m_size = 0;

  public:
    // constructors
    constexpr StaticString() noexcept = default;
    constexpr StaticString(StaticString const&) noexcept = default;
    constexpr StaticString(StaticString&&) noexcept = default;

    template <uint64_t M, std::enable_if_t<(N > M), bool> = true>
    // NOLINTNEXTLINE(hicpp-explicit-conversions), conceptually a copy constructor
    constexpr StaticString(StaticString<M> const& rhs)
        : m_size(rhs.m_size) {
        for (size_t i = 0; i < m_size; ++i) {
            m_string[i] = rhs.m_string[i];
        }
    }

    // destructor
#if __cplusplus >= 202002L
    constexpr
#endif
        ~StaticString() = default;

    // assignment
    constexpr auto operator=(StaticString const&) noexcept -> StaticString& = default;
    constexpr auto operator=(StaticString&&) noexcept -> StaticString& = default;

    template <uint64_t M, std::enable_if_t<(N > M), bool> = true>
    constexpr auto operator=(StaticString<M> const& rhs) noexcept -> StaticString& {
        m_size = rhs.m_size;
        for (size_t i = 0; i < m_size; ++i) {
            m_string[i] = rhs.m_string[i];
        }
        for (size_t i = m_size; i < N; ++i) {
            m_string[i] = '\0';
        }
        return *this;
    }

    /// Constructs a StaticString from a C-string literal.
    /// @return Nullopt if the input string does not represent a valid UTF-8 encoding.
    ///         Otherwise a StaticString that contains a copy of the input string.
    template <uint64_t M, std::enable_if_t<(N >= (M - 1)), bool> = true>
    // NOLINTNEXTLINE(cppcoreguidelines-avoid-c-arrays,hicpp-avoid-c-arrays,modernize-avoid-c-arrays) statically bounds checked
    static auto from_utf8(char const (&utf8_str)[M]) noexcept -> bb::Optional<StaticString> {
        if (utf8_str[M - 1] != '\0') {
            return bb::NULLOPT;
        }
        StaticString ret;
        for (uint64_t i = 0; i < M - 1; ++i) {
            char const character = utf8_str[i];
            if (!ret.try_push_back(character)) {
                return bb::NULLOPT;
            }
        }
        return ret;
    }

    /// Constructs a StaticString from a null terminated C-style string.
    /// This unchecked function allows for uncontrolled memory access. Users of this must ensure that the input
    /// string is properly null terminated.
    /// @return Nullopt if the input string does not represent a valid UTF-8 encoding.
    ///         Otherwise a StaticString that contains a copy of the input string.
    static auto from_utf8_null_terminated_unchecked(char const* utf8_str) -> bb::Optional<StaticString> {
        StaticString ret;
        while (*utf8_str != '\0') {
            if (!ret.try_push_back(*utf8_str)) {
                return bb::NULLOPT;
            }
            // NOLINTNEXTLINE(cppcoreguidelines-pro-bounds-pointer-arithmetic), unchecked access into c-style string
            ++utf8_str;
        }
        return ret;
    }

    /// Constructs a StaticString from a C-string literal. Users must ensure that the input string represents a
    /// valid UTF-8 encoding.
    template <uint64_t M, std::enable_if_t<(N >= (M - 1)), bool> = true>
    // NOLINTNEXTLINE(cppcoreguidelines-avoid-c-arrays,hicpp-avoid-c-arrays,modernize-avoid-c-arrays) statically bounds checked
    static auto from_utf8_unchecked(char const (&utf8_str)[M]) noexcept -> StaticString {
        StaticString ret;
        for (uint64_t i = 0; i < M - 1; ++i) {
            char const character = utf8_str[i];
            if (character == '\0') {
                break;
            }
            ret.push_back(character);
        }
        return ret;
    }

    /// Constructs a StaticString from up to `count` characters of a C-style string. If the capacity of the
    /// StaticString would be exceeded, the input string is truncated.
    /// This unchecked function allows for uncontrolled memory access. User of this must ensure that the input
    /// string is null-terminated if count exceeds the string length and the truncated string represents a valid
    /// UTF-8 encoding.
    static auto from_utf8_null_terminated_unchecked_truncated(char const* utf8_str, SizeType count) -> StaticString {
        StaticString ret;
        auto index = std::min(static_cast<uint64_t>(count), N);
        while (*utf8_str != '\0' && index > 0) {
            ret.push_back(*utf8_str);
            // NOLINTNEXTLINE(cppcoreguidelines-pro-bounds-pointer-arithmetic), unchecked access into c-style string
            ++utf8_str;
            --index;
        }
        return ret;
    }

    /// Attempt to append a single code unit to the back of the string.
    /// @return true on success.
    ///         false if the action would exceed the string's capacity or put the string content into a state that
    ///         is not a valid UTF-8 encoded string.
    constexpr auto try_push_back(CodeUnitValueType character) noexcept -> bool {
        if ((m_size < N) && (is_valid_next(character))) {
            m_string[m_size] = character;
            ++m_size;
            // we explicitly write the terminator here, as the rust string
            // may contain non-null characters after the end
            m_string[m_size] = '\0';
            return true;
        } else {
            return false;
        }
    }

    /// Attempt to pop a single code unit from the back of the string.
    /// @return true on success.
    ///         false if the string is already empty or if the action would put the string content into a state that
    ///         is not a valid UTF-8 encoded string.
    constexpr auto try_pop_back() noexcept -> bool {
        if (m_size > 0) {
            m_string[m_size - 1] = '\0';
            --m_size;
            return true;
        } else {
            return false;
        }
    }

    /// Attempt to append `count` instances of `character` to the back of the string.
    /// @return true on success.
    ///         false if the action would exceed the string's capacity or put the string content into a state that
    ///         is not a valid UTF-8 encoded string.
    constexpr auto try_append(SizeType count, CodeUnitValueType character) noexcept -> bool {
        if ((m_size + count <= N) && (is_valid_next(character))) {
            std::fill(&(m_string[m_size]), &(m_string[m_size + count]), character);
            m_size += count;
            // we explicitly write the terminator here, as the rust string
            // may contain non-null characters after the end
            m_string[m_size] = '\0';
            return true;
        } else {
            return false;
        }
    }

    /// Appends a null terminated C-style string.
    /// This unchecked function allows for uncontrolled memory access. Users of this must ensure that the input
    /// string is properly null terminated.
    /// @return true on success.
    ///         false if the input string does not represent a valid UTF-8 encoding.
    constexpr auto try_append_utf8_null_terminated_unchecked(char const* utf8_str) -> bool {
        auto const old_size = size();
        while (*utf8_str != '\0') {
            if (!try_push_back(*utf8_str)) {
                std::fill(&m_string[old_size], &m_string[m_size], '\0');
                m_size = old_size;
                return false;
            }
            // NOLINTNEXTLINE(cppcoreguidelines-pro-bounds-pointer-arithmetic), unchecked access into c-style string
            ++utf8_str;
        }
        return true;
    }

    static constexpr auto capacity() noexcept -> SizeType {
        return N;
    }

    constexpr auto size() const noexcept -> SizeType {
        return m_size;
    }

    constexpr auto empty() const -> bool {
        return size() == 0;
    }

    /// Unchecked mutable access to the string contents on a per-code-unit basis.
    auto unchecked_code_units() -> UncheckedAccessorCodeUnits {
        return UncheckedAccessorCodeUnits { *this };
    }

    /// Immutable access to the string contents on a per-code-unit basis.
    auto code_units() const -> ConstAccessorCodeUnits {
        return ConstAccessorCodeUnits { *this };
    }

    /// Unchecked mutable access to the string contents.
    auto unchecked_access() -> UncheckedAccessor {
        return UncheckedAccessor { *this };
    }

    /// Unchecked immutable access to the string contents.
    auto unchecked_access() const -> UncheckedConstAccessor {
        return UncheckedConstAccessor { *this };
    }

    // comparison operators
    friend auto operator==(StaticString const& lhs, StaticString const& rhs) -> bool {
        return std::equal(lhs.unchecked_access().begin(),
                          lhs.unchecked_access().end(),
                          rhs.unchecked_access().begin(),
                          rhs.unchecked_access().end());
    }

    friend auto operator!=(StaticString const& lhs, StaticString const& rhs) -> bool {
        return !(lhs == rhs);
    }

    friend auto operator<(StaticString const& lhs, StaticString const& rhs) -> bool {
        return lhs.compare(rhs) < 0;
    }

    friend auto operator<=(StaticString const& lhs, StaticString const& rhs) -> bool {
        return lhs.compare(rhs) <= 0;
    }

    friend auto operator>(StaticString const& lhs, StaticString const& rhs) -> bool {
        return lhs.compare(rhs) > 0;
    }

    friend auto operator>=(StaticString const& lhs, StaticString const& rhs) -> bool {
        return lhs.compare(rhs) >= 0;
    }

    /// Obtains metrics about the internal memory layout of the vector.
    /// This function is intended for internal use only.
    constexpr auto static_memory_layout_metrics() noexcept {
        struct StringMemoryLayoutMetrics {
            size_t string_alignment;
            size_t string_size;
            size_t sizeof_data;
            size_t offset_data;
            size_t sizeof_size;
            size_t offset_size;
            bool size_is_unsigned;
        } ret;
        using Self = std::remove_reference_t<decltype(*this)>;
        ret.string_size = sizeof(Self);
        ret.string_alignment = alignof(Self);
        ret.sizeof_data = sizeof(m_string);
        ret.offset_data = offsetof(Self, m_string);
        ret.sizeof_size = sizeof(m_size);
        ret.offset_size = offsetof(Self, m_size);
        ret.size_is_unsigned = std::is_unsigned<decltype(m_size)>::value;
        return ret;
    }

  private:
    auto is_valid_next(char character) const noexcept -> bool {
        constexpr char const CODE_UNIT_UPPER_BOUND = 127;
        return (character > 0) && (character <= CODE_UNIT_UPPER_BOUND);
    }

    auto compare(StaticString const& other) const -> int64_t {
        auto const other_size = other.size();
        auto const res = memcmp(&m_string[0], &other.m_string[0], std::min(m_size, static_cast<uint64_t>(other_size)));
        if (res == 0) {
            if (m_size < other_size) {
                return -1;
            }
            return ((m_size > other_size) ? 1 : 0);
        }
        return res;
    }

    constexpr void push_back(CodeUnitValueType character) noexcept {
        m_string[m_size] = character;
        ++m_size;
        m_string[m_size] = '\0';
    }

    auto code_unit_based_substr(SizeType pos, SizeType count) const -> bb::Optional<StaticString> {
        if (pos > m_size) {
            return bb::NULLOPT;
        }

        auto const length = std::min(static_cast<uint64_t>(count), m_size - pos);
        StaticString sub_str;
        std::copy(&m_string[pos], &m_string[pos + length], &sub_str.m_string[0]);
        sub_str.m_string[length] = '\0';
        sub_str.m_size = length;
        return sub_str;
    }
};

} // namespace bb
} // namespace iox2

template <uint64_t N>
auto operator<<(std::ostream& stream, const iox2::bb::StaticString<N>& value) -> std::ostream& {
    stream << "StaticString::<" << N << "> { m_size: " << value.size() << ", m_string: \""
           << value.unchecked_access().c_str() << "\" }";
    return stream;
}

#endif // IOX2_INCLUDE_GUARD_BB_STATIC_STRING_HPP
