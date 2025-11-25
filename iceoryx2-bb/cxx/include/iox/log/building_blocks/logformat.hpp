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

#ifndef IOX_HOOFS_REPORTING_LOG_BUILDING_BLOCKS_LOGFORMAT_HPP
#define IOX_HOOFS_REPORTING_LOG_BUILDING_BLOCKS_LOGFORMAT_HPP

#include "iox/iceoryx_hoofs_types.hpp"

#include <cstdint>

namespace iox
{
namespace log
{
// AXIVION DISABLE STYLE AutosarC++19_03-A3.9.1 : Basic numeric types are used in order to get the correct format string
// on all platforms. Fixed width types are used on a higher abstraction layer. Furthermore are 'const char*' used for
// string literals with static lifetime and as a pointer to a buffer of character

/// @brief converts LogLevel into a string literal color code
/// @param[in] value the LogLevel to convert
/// @return string literal of the corresponding color code
constexpr const char* logLevelDisplayColor(const LogLevel value) noexcept;

/// @brief converts LogLevel into a string literal display text
/// @param[in] value the LogLevel to convert
/// @return string literal of the display text
constexpr const char* logLevelDisplayText(const LogLevel value) noexcept;

/// @brief Provides access to the log buffer if direct access is required
struct LogBuffer
{
    const char* buffer{nullptr};
    uint64_t writeIndex{0};
};

namespace internal
{
template <typename T>
constexpr const char* logFormatDec() noexcept;
template <>
constexpr const char* logFormatDec<char>() noexcept;
template <>
constexpr const char* logFormatDec<signed char>() noexcept;
template <>
constexpr const char* logFormatDec<unsigned char>() noexcept;
template <>
constexpr const char* logFormatDec<short>() noexcept;
template <>
constexpr const char* logFormatDec<unsigned short>() noexcept;
template <>
constexpr const char* logFormatDec<int>() noexcept;
template <>
constexpr const char* logFormatDec<unsigned int>() noexcept;
template <>
constexpr const char* logFormatDec<long>() noexcept;
template <>
constexpr const char* logFormatDec<unsigned long>() noexcept;
template <>
constexpr const char* logFormatDec<long long>() noexcept;
template <>
constexpr const char* logFormatDec<unsigned long long>() noexcept;
template <>
constexpr const char* logFormatDec<float>() noexcept;
template <>
constexpr const char* logFormatDec<double>() noexcept;
template <>
constexpr const char* logFormatDec<long double>() noexcept;

template <typename T>
constexpr const char* logFormatHex() noexcept;
template <>
constexpr const char* logFormatHex<unsigned char>() noexcept;
template <>
constexpr const char* logFormatHex<unsigned short>() noexcept;
template <>
constexpr const char* logFormatHex<unsigned int>() noexcept;
template <>
constexpr const char* logFormatHex<unsigned long>() noexcept;
template <>
constexpr const char* logFormatHex<unsigned long long>() noexcept;
template <>
constexpr const char* logFormatHex<float>() noexcept;
template <>
constexpr const char* logFormatHex<double>() noexcept;
template <>
constexpr const char* logFormatHex<long double>() noexcept;
template <>
constexpr const char* logFormatHex<const void*>() noexcept;

template <typename T>
constexpr const char* logFormatOct() noexcept;
template <>
constexpr const char* logFormatOct<unsigned char>() noexcept;
template <>
constexpr const char* logFormatOct<unsigned short>() noexcept;
template <>
constexpr const char* logFormatOct<unsigned int>() noexcept;
template <>
constexpr const char* logFormatOct<unsigned long>() noexcept;
template <>
constexpr const char* logFormatOct<unsigned long long>() noexcept;
} // namespace internal

/// @brief printf-like format string for decimal formatting of numbers
template <typename T>
static constexpr const char* LOG_FORMAT_DEC{internal::logFormatDec<T>()};

/// @brief printf-like format string for hexadecimal formatting of numbers
template <typename T>
static constexpr const char* LOG_FORMAT_HEX{internal::logFormatHex<T>()};

/// @brief printf-like format string for octal formatting of numbers
template <typename T>
static constexpr const char* LOG_FORMAT_OCT{internal::logFormatOct<T>()};

// AXIVION ENABLE STYLE AutosarC++19_03-A3.9.1

} // namespace log
} // namespace iox

#include "iox/detail/log/building_blocks/logformat.inl"

#endif // IOX_HOOFS_REPORTING_LOG_BUILDING_BLOCKS_LOGFORMAT_HPP
