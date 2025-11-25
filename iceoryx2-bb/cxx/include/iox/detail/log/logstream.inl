// Copyright (c) 2019 - 2021 by Robert Bosch GmbH. All rights reserved.
// Copyright (c) 2021 - 2022 by Apex.AI Inc. All rights reserved.
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

#ifndef IOX_HOOFS_REPORTING_LOG_LOGSTREAM_INL
#define IOX_HOOFS_REPORTING_LOG_LOGSTREAM_INL

#include "iox/log/logstream.hpp"

#include <string>

namespace iox
{
// AXIVION Next Construct AutosarC++19_03-M2.10.1 : log is a sensible namespace for a logger; furthermore it is in the
// iox namespace and when used as function the compiler will complain
namespace log
{
template <typename T>
template <typename>
inline constexpr LogHex<T>::LogHex(const T value) noexcept
    : m_value(value)
{
}

// AXIVION Next Construct AutosarC++19_03-M17.0.3 : See at declaration in header
template <typename T, typename>
inline constexpr LogHex<T> hex(const T value) noexcept
{
    return LogHex<T>(value);
}

// AXIVION Next Construct AutosarC++19_03-M17.0.3 : See at declaration in header
inline constexpr LogHex<const void* const> hex(const void* const ptr) noexcept
{
    return LogHex<const void* const>(ptr);
}

template <typename T>
template <typename>
inline constexpr LogOct<T>::LogOct(const T value) noexcept
    : m_value(value)
{
}

// AXIVION Next Construct AutosarC++19_03-M17.0.3 : See at declaration in header
template <typename T, typename>
inline constexpr LogOct<T> oct(const T value) noexcept
{
    return LogOct<T>(value);
}

template <typename T>
template <typename>
inline constexpr LogBin<T>::LogBin(const T value) noexcept
    : m_value(value)
{
}

// AXIVION Next Construct AutosarC++19_03-M17.0.3 : See at declaration in header
template <typename T, typename>
inline constexpr LogBin<T> bin(const T value) noexcept
{
    return LogBin<T>(value);
}

inline constexpr LogRaw::LogRaw(const void* const data, uint64_t size) noexcept
    : m_data(data)
    , m_size(size)
{
}

// AXIVION Next Construct AutosarC++19_03-M17.0.3 : See at declaration in header
template <typename T>
inline constexpr typename std::enable_if<!std::is_pointer<T>::value, LogRaw>::type raw(const T& object) noexcept
{
    return LogRaw(&object, sizeof(T));
}

// AXIVION Next Construct AutosarC++19_03-M17.0.3 : See at declaration in header
inline constexpr LogRaw raw(const void* const data, const uint64_t size) noexcept
{
    return LogRaw(data, size);
}

/// @todo iox-#1755 use something like 'source_location'
// AXIVION Next Construct AutosarC++19_03-A3.9.1 : See at declaration in header
// NOLINTNEXTLINE(readability-function-size)
inline LogStream::LogStream(
    Logger& logger, const char* file, const int line, const char* function, LogLevel logLevel) noexcept
    : m_logger(logger)
{
    m_logger.createLogMessageHeader(file, line, function, logLevel);
}

// AXIVION Next Construct AutosarC++19_03-A3.9.1 : See at declaration in header
inline LogStream::LogStream(const char* file, const int line, const char* function, LogLevel logLevel) noexcept
    : LogStream(Logger::get(), file, line, function, logLevel)
{
}

inline LogStream::~LogStream() noexcept
{
    flush();
}

inline void LogStream::flush() noexcept
{
    if (!m_isFlushed && m_doFlush)
    {
        m_logger.flush();
        m_isFlushed = true;
    }
}

inline LogStream& LogStream::self() noexcept
{
    return *this;
}

// AXIVION DISABLE STYLE AutosarC++19_03-M5.17.1 : This is not used as shift operator but as stream operator and does
// not require to implement '<<='

// AXIVION Next Construct AutosarC++19_03-A3.9.1 : See at declaration in header
inline LogStream& LogStream::operator<<(const char* cstr) noexcept
{
    m_logger.logString(cstr);
    m_isFlushed = false;
    return *this;
}

inline LogStream& LogStream::operator<<(const std::string& str) noexcept
{
    m_logger.logString(str.c_str());
    m_isFlushed = false;
    return *this;
}

inline LogStream& LogStream::operator<<(const bool val) noexcept
{
    m_logger.logBool(val);
    return *this;
}

// AXIVION DISABLE STYLE AutosarC++19_03-A3.9.1 : See at declaration in header

inline LogStream& LogStream::operator<<(const char val) noexcept
{
    m_logger.logDec(val);
    m_isFlushed = false;
    return *this;
}

inline LogStream& LogStream::operator<<(const signed char val) noexcept
{
    m_logger.logDec(val);
    m_isFlushed = false;
    return *this;
}

inline LogStream& LogStream::operator<<(const unsigned char val) noexcept
{
    m_logger.logDec(val);
    m_isFlushed = false;
    return *this;
}

inline LogStream& LogStream::operator<<(const short val) noexcept
{
    m_logger.logDec(val);
    m_isFlushed = false;
    return *this;
}

inline LogStream& LogStream::operator<<(const unsigned short val) noexcept
{
    m_logger.logDec(val);
    m_isFlushed = false;
    return *this;
}

inline LogStream& LogStream::operator<<(const int val) noexcept
{
    m_logger.logDec(val);
    m_isFlushed = false;
    return *this;
}

inline LogStream& LogStream::operator<<(const unsigned int val) noexcept
{
    m_logger.logDec(val);
    m_isFlushed = false;
    return *this;
}

inline LogStream& LogStream::operator<<(const long val) noexcept
{
    m_logger.logDec(val);
    m_isFlushed = false;
    return *this;
}

inline LogStream& LogStream::operator<<(const unsigned long val) noexcept
{
    m_logger.logDec(val);
    m_isFlushed = false;
    return *this;
}

inline LogStream& LogStream::operator<<(const long long val) noexcept
{
    m_logger.logDec(val);
    m_isFlushed = false;
    return *this;
}

inline LogStream& LogStream::operator<<(const unsigned long long val) noexcept
{
    m_logger.logDec(val);
    m_isFlushed = false;
    return *this;
}

inline LogStream& LogStream::operator<<(const float val) noexcept
{
    m_logger.logDec(val);
    m_isFlushed = false;
    return *this;
}

inline LogStream& LogStream::operator<<(const double val) noexcept
{
    m_logger.logDec(val);
    m_isFlushed = false;
    return *this;
}

inline LogStream& LogStream::operator<<(const long double val) noexcept
{
    m_logger.logDec(val);
    m_isFlushed = false;
    return *this;
}

// AXIVION ENABLE STYLE AutosarC++19_03-A3.9.1

template <typename T, typename std::enable_if_t<std::is_integral<T>::value, bool>>
inline LogStream& LogStream::operator<<(const LogHex<T>&& val) noexcept
{
    m_logger.logString("0x");
    m_logger.logHex(static_cast<typename std::make_unsigned<T>::type>(val.m_value));
    m_isFlushed = false;
    return *this;
}

template <typename T, typename std::enable_if_t<std::is_floating_point<T>::value, bool>>
inline LogStream& LogStream::operator<<(const LogHex<T>&& val) noexcept
{
    m_logger.logHex(val.m_value);
    m_isFlushed = false;
    return *this;
}

inline LogStream& LogStream::operator<<(const LogHex<const void* const>&& val) noexcept
{
    m_logger.logHex(val.m_value);
    m_isFlushed = false;
    return *this;
}

template <typename T, typename std::enable_if_t<std::is_integral<T>::value, bool>>
inline LogStream& LogStream::operator<<(const LogOct<T>&& val) noexcept
{
    m_logger.logString("0o");
    m_logger.logOct(static_cast<typename std::make_unsigned<T>::type>(val.m_value));
    m_isFlushed = false;
    return *this;
}

template <typename T, typename std::enable_if_t<std::is_integral<T>::value, bool>>
inline LogStream& LogStream::operator<<(const LogBin<T>&& val) noexcept
{
    m_logger.logString("0b");
    m_logger.logBin(static_cast<typename std::make_unsigned<T>::type>(val.m_value));
    m_isFlushed = false;
    return *this;
}

inline LogStream& LogStream::operator<<(const LogRaw&& val) noexcept
{
    m_logger.logRaw(val.m_data, val.m_size);
    m_isFlushed = false;
    return *this;
}

template <typename Callable, typename>
inline LogStream& LogStream::operator<<(const Callable& c) noexcept
{
    return c(*this);
}

inline LogStream& LogStream::operator<<(const LogLevel value) noexcept
{
    m_logger.logString(asStringLiteral(value));
    return *this;
}

// AXIVION ENABLE STYLE AutosarC++19_03-M5.17.1

} // namespace log
} // namespace iox

#endif // IOX_HOOFS_REPORTING_LOG_LOGSTREAM_INL
