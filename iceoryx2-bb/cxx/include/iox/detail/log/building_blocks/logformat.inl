// Copyright (c) 2022 by Apex.AI Inc. All rights reserved.
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

#ifndef IOX_HOOFS_REPORTING_LOG_BUILDING_BLOCKS_LOGFORMAT_INL
#define IOX_HOOFS_REPORTING_LOG_BUILDING_BLOCKS_LOGFORMAT_INL

#include "iox/log/building_blocks/logformat.hpp"

namespace iox
{
namespace log
{
// AXIVION DISABLE STYLE AutosarC++19_03-A3.9.1 : See at declaration in header
// AXIVION DISABLE STYLE AutosarC++19_03-M2.13.2 : Octal numbers are required for the terminal color codes and it is
// checked that only valid digits are used

inline constexpr const char* logLevelDisplayColor(const LogLevel value) noexcept
{
    switch (value)
    {
    case LogLevel::Off:
        return ""; // nothing
    case LogLevel::Fatal:
        return "\033[0;1;97;41m"; // bold bright white on red
    case LogLevel::Error:
        return "\033[0;1;31;103m"; // bold red on light yellow
    case LogLevel::Warn:
        return "\033[0;1;93m"; // bold bright yellow
    case LogLevel::Info:
        return "\033[0;1;92m"; // bold bright green
    case LogLevel::Debug:
        return "\033[0;1;96m"; // bold bright cyan
    case LogLevel::Trace:
        return "\033[0;1;36m"; // bold cyan
    }

    return ""; // nothing
}

inline constexpr const char* logLevelDisplayText(const LogLevel value) noexcept
{
    switch (value)
    {
    case LogLevel::Off:
        return "[ Off ]";
    case LogLevel::Fatal:
        return "[Fatal]";
    case LogLevel::Error:
        return "[Error]";
    case LogLevel::Warn:
        return "[Warn ]";
    case LogLevel::Info:
        return "[Info ]";
    case LogLevel::Debug:
        return "[Debug]";
    case LogLevel::Trace:
        return "[Trace]";
    }

    return "[UNDEF]";
}

// AXIVION ENABLE STYLE AutosarC++19_03-M2.13.2

namespace internal
{
template <typename>
constexpr bool always_false_v{false};

template <typename T>
inline constexpr const char* logFormatDec() noexcept
{
    static_assert(always_false_v<T>, "This type is not supported for decimal output!");
    return nullptr;
}
template <>
inline constexpr const char* logFormatDec<char>() noexcept
{
    return "%c";
}
template <>
inline constexpr const char* logFormatDec<signed char>() noexcept
{
    return "%hhi";
}
template <>
inline constexpr const char* logFormatDec<unsigned char>() noexcept
{
    return "%hhu";
}
template <>
inline constexpr const char* logFormatDec<short>() noexcept
{
    return "%hi";
}
template <>
inline constexpr const char* logFormatDec<unsigned short>() noexcept
{
    return "%hu";
}
template <>
inline constexpr const char* logFormatDec<int>() noexcept
{
    return "%i";
}
template <>
inline constexpr const char* logFormatDec<unsigned int>() noexcept
{
    return "%u";
}
template <>
inline constexpr const char* logFormatDec<long>() noexcept
{
    return "%li";
}
template <>
inline constexpr const char* logFormatDec<unsigned long>() noexcept
{
    return "%lu";
}
template <>
inline constexpr const char* logFormatDec<long long>() noexcept
{
    return "%lli";
}
template <>
inline constexpr const char* logFormatDec<unsigned long long>() noexcept
{
    return "%llu";
}
template <>
inline constexpr const char* logFormatDec<float>() noexcept
{
    return "%.5e";
}
template <>
inline constexpr const char* logFormatDec<double>() noexcept
{
    return "%.5le";
}
template <>
inline constexpr const char* logFormatDec<long double>() noexcept
{
    return "%.5Le";
}

template <typename T>
inline constexpr const char* logFormatHex() noexcept
{
    static_assert(always_false_v<T>, "This type is not supported for hexadecimal output!");
    return nullptr;
}
template <>
inline constexpr const char* logFormatHex<unsigned char>() noexcept
{
    return "%hhx";
}
template <>
inline constexpr const char* logFormatHex<unsigned short>() noexcept
{
    return "%hx";
}
template <>
inline constexpr const char* logFormatHex<unsigned int>() noexcept
{
    return "%x";
}
template <>
inline constexpr const char* logFormatHex<unsigned long>() noexcept
{
    return "%lx";
}
template <>
inline constexpr const char* logFormatHex<unsigned long long>() noexcept
{
    return "%llx";
}
template <>
inline constexpr const char* logFormatHex<float>() noexcept
{
    return "%a";
}
template <>
inline constexpr const char* logFormatHex<double>() noexcept
{
    return "%la";
}
template <>
inline constexpr const char* logFormatHex<long double>() noexcept
{
    return "%La";
}
template <>
inline constexpr const char* logFormatHex<const void*>() noexcept
{
    return "%p";
}

template <typename T>
inline constexpr const char* logFormatOct() noexcept
{
    static_assert(always_false_v<T>, "This type is not supported for octal output!");
    return nullptr;
}
template <>
inline constexpr const char* logFormatOct<unsigned char>() noexcept
{
    return "%hho";
}
template <>
inline constexpr const char* logFormatOct<unsigned short>() noexcept
{
    return "%ho";
}
template <>
inline constexpr const char* logFormatOct<unsigned int>() noexcept
{
    return "%o";
}
template <>
inline constexpr const char* logFormatOct<unsigned long>() noexcept
{
    return "%lo";
}
template <>
inline constexpr const char* logFormatOct<unsigned long long>() noexcept
{
    return "%llo";
}
} // namespace internal

// AXIVION ENABLE STYLE AutosarC++19_03-A3.9.1

} // namespace log
} // namespace iox

#endif // IOX_HOOFS_REPORTING_LOG_BUILDING_BLOCKS_LOGFORMAT_INL
