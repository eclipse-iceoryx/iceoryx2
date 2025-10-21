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

#ifndef IOX2_INCLUDE_GUARD_CONTAINER_STATIC_STRING_HPP
#define IOX2_INCLUDE_GUARD_CONTAINER_STATIC_STRING_HPP

#include "iox2/container/optional.hpp"

#include <algorithm>
#include <cstddef>
#include <cstdint>
#include <cstring>
#include <functional>
#include <ostream>
#include <type_traits>

namespace iox2 {
namespace container {

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
    using OptionalReference = Optional<std::reference_wrapper<char>>;
    using OptionalConstReference = Optional<std::reference_wrapper<char const>>;
    using OptionalCodeUnitReference = Optional<std::reference_wrapper<CodeUnitValueType>>;
    using OptionalConstCodeUnitReference = Optional<std::reference_wrapper<CodeUnitValueType const>>;

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
        UncheckedConstAccessor(UncheckedConstAccessor&&) = delete;
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
        UncheckedAccessor(UncheckedAccessor&&) = delete;
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

      private:
        StaticString* m_parent;

        constexpr explicit UncheckedAccessorCodeUnits(StaticString& parent)
            : m_parent(&parent) {
        }

      public:
        ~UncheckedAccessorCodeUnits() = default;
        UncheckedAccessorCodeUnits(UncheckedAccessorCodeUnits const&) = delete;
        UncheckedAccessorCodeUnits(UncheckedAccessorCodeUnits&&) = delete;
        auto operator=(UncheckedAccessorCodeUnits const&) -> UncheckedAccessorCodeUnits& = delete;
        auto operator=(UncheckedAccessorCodeUnits&&) -> UncheckedAccessorCodeUnits& = delete;

        /// Retrieve a reference to the single code unit at `index`.
        /// @return A reference to the code unit or `nullopt` if the index is out of bounds.
        auto element_at(SizeType index) noexcept -> OptionalCodeUnitReference {
            if (index < m_parent->m_size) {
                return m_parent->m_string[index];
            } else {
                return nullopt;
            }
        }

        /// Retrieve a reference to the first code unit at the beginning of the string.
        /// @return A reference to the front code unit or `nullopt` if the string is empty.
        auto front_element() noexcept -> OptionalCodeUnitReference {
            if (!m_parent->empty()) {
                return m_parent->m_string[0];
            } else {
                return nullopt;
            }
        }

        /// Retrieve a reference to the last code unit at the end of the string.
        /// @return A reference to the back code unit or `nullopt` if the string is empty.
        auto back_element() noexcept -> OptionalCodeUnitReference {
            if (!m_parent->empty()) {
                return m_parent->m_string[m_parent->size() - 1];
            } else {
                return nullopt;
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

      private:
        StaticString const* m_parent;

        constexpr explicit ConstAccessorCodeUnits(StaticString const& parent)
            : m_parent(&parent) {
        }

      public:
        ~ConstAccessorCodeUnits() = default;
        ConstAccessorCodeUnits(ConstAccessorCodeUnits const&) = delete;
        ConstAccessorCodeUnits(ConstAccessorCodeUnits&&) = delete;
        auto operator=(ConstAccessorCodeUnits const&) -> ConstAccessorCodeUnits& = delete;
        auto operator=(ConstAccessorCodeUnits&&) -> ConstAccessorCodeUnits& = delete;

        /// Retrieve the single code unit at `index`.
        /// @return A reference to the code unit or `nullopt` if the index is out of bounds.
        auto element_at(SizeType index) const noexcept -> OptionalConstCodeUnitReference {
            if (index < m_parent->m_size) {
                return m_parent->m_string[index];
            } else {
                return nullopt;
            }
        }

        /// Retrieve the first code unit at the beginning of the string.
        /// @return A reference to the front code unit or `nullopt` if the string is empty.
        auto front_element() const noexcept -> OptionalConstCodeUnitReference {
            if (!m_parent->empty()) {
                return m_parent->m_string[0];
            } else {
                return nullopt;
            }
        }

        /// Retrieve the last code unit at the end of the string.
        /// @return A reference to the back code unit or `nullopt` if the string is empty.
        auto back_element() const noexcept -> OptionalConstCodeUnitReference {
            if (!m_parent->empty()) {
                return m_parent->m_string[m_parent->size() - 1];
            } else {
                return nullopt;
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
    static auto from_utf8(char const (&utf8_str)[M]) noexcept -> Optional<StaticString> {
        if (utf8_str[M - 1] != '\0') {
            return nullopt;
        }
        StaticString ret;
        for (uint64_t i = 0; i < M - 1; ++i) {
            char const character = utf8_str[i];
            if (!ret.try_push_back(character)) {
                return nullopt;
            }
        }
        return ret;
    }

    /// Constructs a StaticString from a null terminated C-style string.
    /// This unchecked function allows for uncontrolled memory access. Users of this must ensure that the input string
    /// is properly null terminated.
    /// @return Nullopt if the input string does not represent a valid UTF-8 encoding.
    ///         Otherwise a StaticString that contains a copy of the input string.
    static auto from_utf8_null_terminated_unchecked(char const* utf8_str) -> Optional<StaticString> {
        StaticString ret;
        while (*utf8_str != '\0') {
            if (!ret.try_push_back(*utf8_str)) {
                return nullopt;
            }
            // NOLINTNEXTLINE(cppcoreguidelines-pro-bounds-pointer-arithmetic), unchecked access into c-style string
            ++utf8_str;
        }
        return ret;
    }

    /// Attempt to append a single code unit to the back of the string.
    /// @return true on success.
    ///         false if the action would exceed the string's capacity or put the string content into a state that is
    ///         not a valid UTF-8 encoded string.
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
    ///         false if the string is already empty or if the action would put the string content into a state that is
    ///         not a valid UTF-8 encoded string.
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
    ///         false if the action would exceed the string's capacity or put the string content into a state that is
    ///         not a valid UTF-8 encoded string.
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
    /// This unchecked function allows for uncontrolled memory access. Users of this must ensure that the input string
    /// is properly null terminated.
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
        // NOLINTNEXTLINE(modernize-type-traits), _v requires C++17
        ret.size_is_unsigned = std::is_unsigned<decltype(m_size)>::value;
        return ret;
    }

  private:
    auto is_valid_next(char character) const noexcept -> bool {
        constexpr char const CODE_UNIT_UPPER_BOUND = 127;
        return (character > 0) && (character <= CODE_UNIT_UPPER_BOUND);
    }
};

template <typename>
struct IsStaticString : std::false_type { };

template <uint64_t N>
struct IsStaticString<StaticString<N>> : std::true_type { };

} // namespace container
} // namespace iox2

template <uint64_t N>
auto operator<<(std::ostream& stream, const iox2::container::StaticString<N>& value) -> std::ostream& {
    stream << "StaticString::<" << N << "> { m_size: " << value.size() << ", m_string: \""
           << value.unchecked_access().c_str() << "\" }";
    return stream;
}

#endif
