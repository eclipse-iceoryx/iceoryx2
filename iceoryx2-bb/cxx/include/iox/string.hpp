// Copyright (c) 2019 by Robert Bosch GmbH. All rights reserved.
// Copyright (c) 2021 - 2023 by Apex.AI Inc. All rights reserved.
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
#ifndef IOX_HOOFS_VOCABULARY_STRING_HPP
#define IOX_HOOFS_VOCABULARY_STRING_HPP

#include "iox/assertions.hpp"
#include "iox/buffer_info.hpp"
#include "iox/detail/string_internal.hpp"
#include "iox/detail/string_type_traits.hpp"
#include "iox/function.hpp"
#include "iox/log/logstream.hpp"
#include "iox/optional.hpp"
#include "iox/type_traits.hpp"

#include <algorithm>
#include <cstdint>
#include <cstring>

namespace iox
{
namespace log
{
class LogStream;
}

/// @brief This helper struct can be used to enable the comparison methods for a custom string type by specializing for
/// a custom 'T'
/// @note The custom string needs a specialisation of 'GetData' and 'GetSize' as well
template <typename T>
struct is_custom_string : public std::false_type
{
};

// AXIVION DISABLE STYLE AutosarC++19_03-A18.1.1 : C-array type usage is intentional

template <typename T, typename ReturnType>
using IsStringOrCharArrayOrChar =
    typename std::enable_if<((is_iox_string<T>::value || is_char_array<T>::value)
                             || (std::is_same<T, char>::value || is_custom_string<T>::value)),
                            ReturnType>::type;

template <typename T, typename ReturnType>
using IsStringOrCharArray =
    typename std::enable_if<((is_iox_string<T>::value || is_char_array<T>::value) || is_custom_string<T>::value),
                            ReturnType>::type;

template <typename T, typename ReturnType>
using IsCustomStringOrCharArrayOrChar =
    typename std::enable_if<((is_char_array<T>::value || std::is_same<T, char>::value) || is_custom_string<T>::value),
                            ReturnType>::type;

template <typename T, typename ReturnType>
using IsIoxStringOrCharArray =
    typename std::enable_if<(is_iox_string<T>::value || is_char_array<T>::value), ReturnType>::type;

template <typename T1, typename T2, typename ReturnType>
using IsIoxStringOrCharArrayOrChar =
    typename std::enable_if<((is_char_array<T1>::value || is_iox_string<T1>::value) || std::is_same<T1, char>::value)
                                && ((is_char_array<T2>::value || is_iox_string<T2>::value)
                                    || std::is_same<T2, char>::value),
                            ReturnType>::type;

template <typename T1, typename T2, typename ReturnType>
using IsIoxStringAndIoxStringOrCharArrayOrChar =
    typename std::enable_if<((is_char_array<T1>::value || std::is_same<T1, char>::value) && is_iox_string<T2>::value)
                                || (is_iox_string<T1>::value
                                    && ((is_char_array<T2>::value || std::is_same<T2, char>::value)
                                        || (is_iox_string<T1>::value && is_iox_string<T2>::value))),
                            ReturnType>::type;

/// @brief concatenates two iox::strings/string literals/chars
///
/// @param [in] iox::strings/string literals/chars to concatenate
///
/// @return a new iox::string with capacity equal to the sum of the capacities of the concatenated strings/chars
///
/// @code
///     string<5> fuu("cdefg");
///     auto bar = iox::concatenate(fuu, "ahc");
/// @endcode
template <typename T1, typename T2>
IsIoxStringOrCharArrayOrChar<T1, T2, string<internal::SumCapa<T1, T2>::value>> concatenate(const T1& str1,
                                                                                           const T2& str2) noexcept;

/// @brief concatenates an arbitrary number of iox::strings, string literals or chars
///
/// @param [in] iox::strings/string literals/chars to concatenate
///
/// @return a new iox::string with capacity equal to the sum of the capacities of the concatenated strings/chars
///
/// @code
///     string<4> fuu("cdef");
///     auto bar = iox::concatenate(fuu, "g", "ah", fuu);
/// @endcode
template <typename T1, typename T2, typename... Targs>
IsIoxStringOrCharArrayOrChar<T1, T2, string<internal::SumCapa<T1, T2, Targs...>::value>>
concatenate(const T1& str1, const T2& str2, const Targs&... targs) noexcept;

// AXIVION Next Construct AutosarC++19_03-M17.0.3 : operator+ is defined within iox namespace which prevents easy
// misuse
/// @brief concatenates two iox::strings or one iox::string and one string literal/char; concatenation of two
/// string literals/chars is not possible
///
/// @param [in] iox::strings/string literal/char to concatenate
///
/// @return a new iox::string with capacity equal to the sum of the capacities of the concatenated strings/chars
template <typename T1, typename T2>
IsIoxStringAndIoxStringOrCharArrayOrChar<T1, T2, string<internal::SumCapa<T1, T2>::value>>
operator+(const T1& str1, const T2& str2) noexcept;

/// @brief struct used to define a compile time variable which is used to distinguish between
/// constructors with certain behavior
struct TruncateToCapacity_t
{
    explicit TruncateToCapacity_t() = default;
};
constexpr TruncateToCapacity_t TruncateToCapacity{};

/// @brief string implementation with some adjustments in the API, because we are not allowed to throw exceptions or use
/// heap.
template <uint64_t Capacity>
class string final
{
    static_assert(Capacity > 0U, "The capacity of the fixed string must be greater than 0!");

  public:
    /// @brief creates an empty string with size 0
    constexpr string() noexcept = default;

    /// @brief copy constructor
    ///
    /// @param [in] other is the copy origin
    string(const string& other) noexcept;

    /// @brief move constructor
    ///
    /// @param [in] other is the move origin
    string(string&& other) noexcept;

    /// @brief copy assignment
    ///
    /// @param [in] rhs is the copy origin
    ///
    /// @return reference to self
    string& operator=(const string& rhs) noexcept;

    /// @brief move assignment
    ///
    /// @param [in] rhs is the move origin
    ///
    /// @return reference to self
    string& operator=(string&& rhs) noexcept;

    /// @brief destructor
    ~string() noexcept = default;

    /// @brief creates a new string of given capacity as a copy of other with compile time check whether the capacity of
    /// other is less than or equal to this' capacity
    ///
    /// @param [in] other is the copy origin
    template <uint64_t N>
    // NOLINTNEXTLINE(hicpp-explicit-conversions) copy constructor for string with different capacity
    string(const string<N>& other) noexcept;

    /// @brief moves other to this with compile time check whether the capacity of other is less than or equal to
    /// this' capacity
    ///
    /// @param [in] other is the move origin
    template <uint64_t N>
    // NOLINTNEXTLINE(hicpp-explicit-conversions) move constructor for string with different capacity
    string(string<N>&& other) noexcept;

    /// @brief assigns rhs fixed string to this with compile time check whether the capacity of rhs is less than or
    /// equal to this' capacity
    ///
    /// @param [in] rhs is the copy origin
    ///
    /// @return reference to self
    template <uint64_t N>
    string& operator=(const string<N>& rhs) noexcept;

    /// @brief moves rhs fixed string to this with compile time check whether the capacity of rhs is less than or
    /// equal to this' capacity
    ///
    /// @param [in] rhs is the move origin
    ///
    /// @return reference to self
    template <uint64_t N>
    string& operator=(string<N>&& rhs) noexcept;

    /// @brief conversion constructor for char array with compile time check if the array size is less than or equal
    /// to the string capacity
    ///
    /// @tparam N is the implicit template parameter for the char array size
    /// @param [in] other is the char array
    ///
    /// @note if the array is not zero-terminated, the last value will be overwritten with 0
    ///
    /// @code
    ///     #include "iox/string.hpp"
    ///     using namespace iox;
    ///
    ///     int main()
    ///     {
    ///         string<4> fuu("abcd");
    ///     }
    /// @endcode
    template <uint64_t N>
    // avoid-c-arrays: iox::string wraps char array
    // explicit-conversions: we want to assign string literals to the string, like string<10> str = "abc"; this is safe
    // because iox::string wraps char array
    // NOLINTNEXTLINE(hicpp-avoid-c-arrays, cppcoreguidelines-avoid-c-arrays, hicpp-explicit-conversions)
    string(const char (&other)[N]) noexcept;

    /// @brief conversion constructor for cstring to string which truncates characters if the size is greater than
    /// the string capacity
    ///
    /// @param [in] TruncateToCapacity_t is a compile time variable which is used to distinguish between
    /// constructors with certain behavior
    /// @param [in] other is the cstring to convert
    /// @attention Passed strings need to be null-terminated. Truncates characters if the size is greater than the
    /// string capacity.
    ///
    /// @code
    ///     #include "iox/string.hpp"
    ///     using namespace iox;
    ///
    ///     int main()
    ///     {
    ///         string<4> fuu(TruncateToCapacity, "abcd");
    ///     }
    /// @endcode
    // TruncateToCapacity_t is a compile time variable to distinguish between constructors
    // NOLINTNEXTLINE(hicpp-named-parameter, readability-named-parameter)
    string(TruncateToCapacity_t, const char* const other) noexcept;

    /// @brief constructor from cstring to string. Constructs the string with the first count characters of the cstring
    /// including null characters. If count is greater than the string capacity the remainder of the characters are
    /// truncated.
    ///
    /// @param [in] TruncateToCapacity_t is a compile time variable which is used to distinguish between
    /// constructors with certain behavior
    /// @param [in] other is the cstring to convert
    /// @param [in] count is the number of characters for constructing the string
    ///
    /// @code
    ///     #include "iox/string.hpp"
    ///     using namespace iox;
    ///
    ///     int main()
    ///     {
    ///         string<4> fuu(TruncateToCapacity, "abcd", 2);
    ///     }
    /// @endcode
    // TruncateToCapacity_t is a compile time variable to distinguish between constructors
    // NOLINTNEXTLINE(hicpp-named-parameter, readability-named-parameter)
    string(TruncateToCapacity_t, const char* const other, const uint64_t count) noexcept;

    /// @brief assigns a char array to string with compile time check if the array size is less than or equal
    /// to the string capacity
    ///
    /// @tparam [in] N is the implicit template parameter for the char array size
    /// @param [in] rhs is the char array
    ///
    /// @return reference to self
    ///
    /// @note if the array is not zero-terminated, the last value will be overwritten with 0
    ///
    /// @code
    ///     #include "iox/string.hpp"
    ///     using namespace iox;
    ///
    ///     int main()
    ///     {
    ///         string<4> fuu = "abcd";
    ///     }
    /// @endcode
    template <uint64_t N>
    // We want to assign string literals to the iox::string, like myString = "abc";
    // NOLINTNEXTLINE(hicpp-avoid-c-arrays, cppcoreguidelines-avoid-c-arrays)
    string& operator=(const char (&rhs)[N]) noexcept;

    /// @brief fixed string assignment with compile time check if capacity of str is less than or equal to this'
    /// capacity
    ///
    /// @param [in] str is the fixed string object to assign
    ///
    /// @return reference to self
    template <uint64_t N>
    string& assign(const string<N>& str) noexcept;

    /// @brief assigns a char array to string with compile time check if the array size is less than or equal to the
    /// string capacity
    ///
    /// @tparam [in] N is the implicit template parameter for the char array size
    /// @param [in] str is the char array
    ///
    /// @return reference to self
    ///
    /// @note if the array is not zero-terminated, the last value will be overwritten with 0
    ///
    /// @code
    ///
    ///     #include "iox/string.hpp"
    ///     using namespace iox;
    ///
    ///     int main()
    ///     {
    ///         string<4> fuu;
    ///         char bar[] = "abcd";
    ///         fuu.assign(bar);
    ///     }
    /// @endcode
    template <uint64_t N>
    // We want to assign string literals to the iox::string, like myString.assign("abc");
    // NOLINTNEXTLINE(hicpp-avoid-c-arrays, cppcoreguidelines-avoid-c-arrays)
    string& assign(const char (&str)[N]) noexcept;

    /// @brief assigns a cstring to string. The assignment fails if the cstring size is greater than the string
    /// capacity.
    /// @attention Passed strings need to be null-terminated.
    ///
    /// @param [in] str is the cstring to assign
    ///
    /// @return true if the assignment succeeds, otherwise false
    bool unsafe_assign(const char* const str) noexcept;

    /// @brief compares self and an iox::string, custom string or char array
    ///
    /// @param [in] other is the string to compare with self
    ///
    /// @return an integer < 0 if the first character that does not match has a lower value in self than in other, 0 if
    /// the contents of both strings are equal, an integer > 0 if the first character that does not match has a greater
    /// value in self than in other
    template <typename T>
    IsStringOrCharArray<T, int64_t> compare(const T& other) const noexcept;

    /// @brief compares self and a char
    ///
    /// @param [in] other is the char to compare with self
    ///
    /// @return an integer < 0 if the first character that does not match has a lower value in self than in other, 0 if
    /// the contents of both strings are equal, an integer > 0 if the first character that does not match has a greater
    /// value in self than in other
    ///
    /// @note the logic is the same as in the other compare method with other treated as a string with size 1
    int64_t compare(char other) const noexcept;

    /// @brief returns a pointer to the char array of self
    ///
    /// @return a pointer to the char array of self
    const char* c_str() const noexcept;

    /// @brief returns the number of characters stored in the string
    ///
    /// @return the number of characters stored in the string
    constexpr uint64_t size() const noexcept;

    /// @brief returns the maximum number of characters that can be stored in the string
    ///
    /// @return the maximum number of characters that can be stored in the string
    static constexpr uint64_t capacity() noexcept;

    /// @brief returns if the string is empty or not
    ///
    /// @return true if size() == 0 otherwise false
    constexpr bool empty() const noexcept;

    /// @brief clears the content of the string
    constexpr void clear() noexcept;

    /// @brief since there are two valid options for what should happen when appending a string larger than this'
    /// capacity (failing or truncating), the fixed string does not support operator+=; use append for truncating or
    /// unsafe_append for failing in that case
    template <typename T>
    // NOLINTNEXTLINE(hicpp-named-parameter, readability-named-parameter) method is disabled via static_assert
    string& operator+=(const T&) noexcept;

    /// @brief appends a iox::string/string literal/custom string to the end of this. If this' capacity is too
    /// small for appending the whole string (literal), the remainder of the characters are truncated.
    ///
    /// @param [in] TruncateToCapacity_t is a compile time variable which is used to make the user aware of the possible
    /// truncation
    /// @param [in] str is the iox::string/string literal/custom string to append
    ///
    /// @return reference to self
    ///
    /// @code
    ///     string<5> fuu("cde");
    ///     fuu.append(TruncateToCapacity, "fgahc");
    /// @endcode
    template <typename T>
    // TruncateToCapacity_t informs how the method behaves when str does not fit into the string
    // NOLINTNEXTLINE(hicpp-named-parameter, readability-named-parameter)
    IsStringOrCharArrayOrChar<T, string&> append(TruncateToCapacity_t, const T& str) noexcept;

    /// @brief appends a char to the end of this if this' capacity is large enough.
    ///
    /// @param [in] cstr is the char to append
    ///
    /// @return reference to self
    // TruncateToCapacity_t informs how the method behaves when cstr does not fit into the string
    // NOLINTNEXTLINE(hicpp-named-parameter, readability-named-parameter)
    string& append(TruncateToCapacity_t, char cstr) noexcept;

    /// @brief appends a iox::string/string literal/char/custom string to the end of this. The appending fails if the
    /// sum of both sizes is greater than this' capacity.
    ///
    /// @param [in] iox::string/string literal/char/custom string to append
    ///
    /// @return true if the appending succeeds, otherwise false
    template <typename T>
    IsStringOrCharArrayOrChar<T, bool> unsafe_append(const T& str) noexcept;

    /// @brief direct access to the string's raw pointer. The access resizes the data with the value returned by the
    /// passed function. If the data written has not the terminator at the returned size, a FATAL error occurs.
    ///
    /// @param [in] function func is a function composed by the raw data pointer and the BufferInfo with current size
    /// and total size, including the space for the zero termination. The return value is the string length.
    ///
    /// @code
    ///     iox::string<100> s;
    ///     s.unsafe_raw_access([] (auto* str, const auto info) {
    ///         strncpy(str, "Hello World", info.total_size);
    ///         return strlen("Hello World");
    ///     });
    /// @endcode
    void unsafe_raw_access(const iox::function_ref<uint64_t(char*, const iox::BufferInfo info)>& func) noexcept;

    /// @brief inserts a iox::string or char array in the range [str[0], str[count]) at position pos. The insertion
    /// fails if the string capacity would be exceeded or pos is greater than the string size or count is greater than
    /// the string to be inserted.
    ///
    /// @param [in] pos position at which the string shall be inserted
    /// @param [in] str the iox::string or char array to be inserted
    /// @param [in] count number of characters to be inserted
    ///
    /// @return true if the insertion was successful, otherwise false
    template <typename T>
    IsIoxStringOrCharArray<T, bool> insert(const uint64_t pos, const T& str, const uint64_t count) noexcept;

    /// @brief creates a substring containing the characters from pos until count; if pos+count is greater than the size
    /// of the original string the returned substring only contains the characters from pos until size();
    /// iox::nullopt is returned if pos is greater than the size of the original string;
    ///
    /// @param [in] pos is the position of the first character used for the substring
    /// @param [in] count is the requested length of the substring
    ///
    /// @return an optional containing the substring, iox::nullopt if pos is greater than the size of the original
    /// string
    optional<string<Capacity>> substr(const uint64_t pos, const uint64_t count) const noexcept;

    /// @brief creates a substring containing the characters from pos until size(); iox::nullopt is returned if pos
    /// is greater than the size of the original string
    ///
    /// @param [in] pos is the position of the first character used for the substring
    ///
    /// @return an optional containing the substring, iox::nullopt if pos is greater than the size of the original
    /// string
    optional<string<Capacity>> substr(const uint64_t pos = 0U) const noexcept;

    /// @brief finds the first occurence of the given character sequence; returns the position of the first character of
    /// the found substring, returns iox::nullopt if no substring is found or if pos is greater than this' size
    ///
    /// @param [in] str is the character sequence to search for; must be a iox::string, string literal or custom string
    /// @param [in] pos is the position at which to start the search
    ///
    /// @return an optional containing the position of the first character of the found substring, iox::nullopt if
    /// no substring is found
    template <typename T>
    IsStringOrCharArray<T, optional<uint64_t>> find(const T& str, const uint64_t pos = 0U) const noexcept;

    /// @brief finds the first occurence of a character equal to one of the characters of the given character sequence
    /// and returns its position; returns iox::nullopt if no character is found or if pos is greater than this'
    /// size
    ///
    /// @param [in] str is the character sequence to search for; must be a iox::string, string literal or custom string
    /// @param [in] pos is the position at which to start the search
    ///
    /// @return an optional containing the position of the first character equal to one of the characters of the given
    /// character sequence, iox::nullopt if no character is found
    template <typename T>
    IsStringOrCharArray<T, optional<uint64_t>> find_first_of(const T& str, const uint64_t pos = 0U) const noexcept;

    /// @brief finds the last occurence of a character equal to one of the characters of the given character sequence
    /// and returns its position; returns iox::nullopt if no character is found
    ///
    /// @param [in] str is the character sequence to search for; must be a iox::string, string literal or custom string
    /// @param [in] pos is the position at which to finish the search
    ///
    /// @return an optional containing the position of the last character equal to one of the characters of the given
    /// character sequence, iox::nullopt if no character is found
    template <typename T>
    IsStringOrCharArray<T, optional<uint64_t>> find_last_of(const T& str, const uint64_t pos = Capacity) const noexcept;

    /// @brief returns a reference to the character stored at pos
    /// @param[in] pos position of character to return
    /// @return reference to the character
    /// @note out of bounds access leads to program termination
    constexpr char& at(const uint64_t pos) noexcept;

    /// @brief returns a reference to the character stored at pos
    /// @param[in] pos position of character to return
    /// @return const reference to the character
    /// @note out of bounds access leads to program termination
    constexpr const char& at(const uint64_t pos) const noexcept;

    /// @brief returns a reference to the character stored at pos
    /// @param[in] pos position of character to return
    /// @return reference to the character
    /// @note does not perform any bound checks
    constexpr char& unchecked_at(const uint64_t pos) noexcept;

    /// @brief returns a reference to the character stored at pos
    /// @param[in] pos position of character to return
    /// @return const reference to the character
    /// @note does not perform any bound checks
    constexpr const char& unchecked_at(const uint64_t pos) const noexcept;

    /// @brief returns a reference to the character stored at pos
    /// @param[in] pos position of the character to return
    /// @return reference to the character
    /// @note out of bounds access leads to program termination
    constexpr char& operator[](const uint64_t pos) noexcept;

    /// @brief returns a const reference to the character stored at pos
    /// @param[in] pos position of the character to return
    /// @return const reference to the character
    /// @note out of bounds access leads to program termination
    constexpr const char& operator[](const uint64_t pos) const noexcept;

    template <uint64_t N>
    friend class string;

    template <typename T1, typename T2>
    friend IsIoxStringOrCharArrayOrChar<T1, T2, string<internal::SumCapa<T1, T2>::value>>
    concatenate(const T1& str1, const T2& str2) noexcept;

  private:
    /// @brief copies rhs fixed string to this with compile time check whether rhs capacity is less than or equal to
    /// this' capacity
    ///
    /// @param [in] rhs is the copy origin
    ///
    /// @return reference to self
    template <uint64_t N>
    string& copy(const string<N>& rhs) noexcept;

    /// @brief moves rhs fixed string to this with compile time check whether rhs capacity is less than or equal to
    /// this' capacity
    ///
    /// @param [in] rhs is the move origin
    ///
    /// @return reference to self
    template <uint64_t N>
    string& move(string<N>&& rhs) noexcept;

    // AXIVION Next Construct FaultDetection-IndirectAssignmentOverflow : False positive. Overflow checks are done before assigning and the data will be truncated if necessary
    // NOLINTNEXTLINE(hicpp-avoid-c-arrays, cppcoreguidelines-avoid-c-arrays) safe access is guaranteed since the char array is wrapped inside the string class
    char m_rawstring[Capacity + 1]{};
    uint64_t m_rawstringSize{0U};
};

/// @brief Logging support for the fixed string
///
/// @param [in] stream is the output stream
/// @param [in] str is the fixed string
/// @return a reference to the LogStream instance
template <uint64_t Capacity>
log::LogStream& operator<<(log::LogStream& stream, const string<Capacity>& str) noexcept;

// AXIVION DISABLE STYLE AutosarC++19_03-A13.5.5: Comparison with custom string, char array or
// char is also intended
/// @brief checks if a lhs custom string, char array or char is equal to a rhs iox::string
///
/// @param [in] rhs is the iox::string
///
/// @return true if the contents of lhs and rhs are equal, otherwise false
template <typename T, uint64_t Capacity>
IsCustomStringOrCharArrayOrChar<T, bool> operator==(const T& lhs, const string<Capacity>& rhs) noexcept;

/// @brief checks if a lhs custom string, char array or char is not equal to a rhs iox::string
///
/// @param [in] lhs is the custom string, char array or char
/// @param [in] rhs is the iox::string
///
/// @return true if the contents of lhs and rhs are not equal, otherwise false
template <typename T, uint64_t Capacity>
IsCustomStringOrCharArrayOrChar<T, bool> operator!=(const T& lhs, const string<Capacity>& rhs) noexcept;

/// @brief checks if a lhs custom string, char array or char is less than a rhs iox::string
///
/// @param [in] lhs is the custom string, char array or char
/// @param [in] rhs is the iox::string
///
/// @return true if lhs is less than rhs, otherwise false
template <typename T, uint64_t Capacity>
IsCustomStringOrCharArrayOrChar<T, bool> operator<(const T& lhs, const string<Capacity>& rhs) noexcept;

/// @brief checks if a lhs custom string, char array or char is less than or equal to a rhs iox::string
///
/// @param [in] lhs is the custom string, char array or char
/// @param [in] rhs is the iox::string
///
/// @return true if lhs is less than or equal to rhs, otherwise false
template <typename T, uint64_t Capacity>
IsCustomStringOrCharArrayOrChar<T, bool> operator<=(const T& lhs, const string<Capacity>& rhs) noexcept;

/// @brief checks if a lhs custom string, char array or char is greater than a rhs iox::string
///
/// @param [in] lhs is the custom string, char array or char
/// @param [in] rhs is the iox::string
///
/// @return true if lhs is greater than rhs, otherwise false
template <typename T, uint64_t Capacity>
IsCustomStringOrCharArrayOrChar<T, bool> operator>(const T& lhs, const string<Capacity>& rhs) noexcept;

/// @brief checks if a lhs custom string, char array or char is greater than or equal to a rhs iox::string
///
/// @param [in] lhs is the custom string, char array or char
/// @param [in] rhs is the iox::string
///
/// @return true if lhs is greater than or equal to rhs, otherwise false
template <typename T, uint64_t Capacity>
IsCustomStringOrCharArrayOrChar<T, bool> operator>=(const T& lhs, const string<Capacity>& rhs) noexcept;

/// @brief checks if lhs is equal to rhs
///
/// @param [in] lhs is the iox::string
/// @param [in] rhs is the iox::string, custom string, char array or char to compare with lhs
///
/// @return true if both strings are equal, otherwise false
template <typename T, uint64_t Capacity>
IsStringOrCharArrayOrChar<T, bool> operator==(const string<Capacity>& lhs, const T& rhs) noexcept;

/// @brief checks if lhs is not equal to rhs
///
/// @param [in] lhs is the iox::string
/// @param [in] rhs is the iox::string, custom string, char array or char to compare with lhs
///
/// @return true if both strings are not equal, otherwise false
template <typename T, uint64_t Capacity>
IsStringOrCharArrayOrChar<T, bool> operator!=(const string<Capacity>& lhs, const T& rhs) noexcept;

/// @brief checks if lhs is less than rhs, in lexicographical order
///
/// @param [in] lhs is the iox::string
/// @param [in] rhs is the iox::string, custom string, char array or char to compare with lhs
///
/// @return true if lhs is less than rhs, otherwise false
template <typename T, uint64_t Capacity>
IsStringOrCharArrayOrChar<T, bool> operator<(const string<Capacity>& lhs, const T& rhs) noexcept;

/// @brief checks if lhs is less than or equal to rhs, in lexicographical order
///
/// @param [in] lhs is the iox::string
/// @param [in] rhs is the iox::string, custom string, char array or char to compare with lhs
///
/// @return true if lhs is less than or equal to rhs, otherwise false
template <typename T, uint64_t Capacity>
IsStringOrCharArrayOrChar<T, bool> operator<=(const string<Capacity>& lhs, const T& rhs) noexcept;

/// @brief checks if lhs is greater than rhs, in lexicographical order
///
/// @param [in] lhs is the iox::string
/// @param [in] rhs is the iox::string, custom string, char array or char to compare with lhs
///
/// @return true if lhs is greater than rhs, otherwise false
template <typename T, uint64_t Capacity>
IsStringOrCharArrayOrChar<T, bool> operator>(const string<Capacity>& lhs, const T& rhs) noexcept;

/// @brief checks if lhs is greater than or equal to rhs, in lexicographical order
///
/// @param [in] lhs is the iox::string
/// @param [in] rhs is the iox::string, custom string, char array or char to compare with lhs
///
/// @return true if lhs is greater than or equal to rhs, otherwise false
template <typename T, uint64_t Capacity>
IsStringOrCharArrayOrChar<T, bool> operator>=(const string<Capacity>& lhs, const T& rhs) noexcept;
// AXIVION ENABLE STYLE AutosarC++19_03-A13.5.5
// AXIVION ENABLE STYLE AutosarC++19_03-A18.1.1

} // namespace iox
#include "iox/detail/string.inl"

#endif // IOX_HOOFS_VOCABULARY_STRING_HPP
