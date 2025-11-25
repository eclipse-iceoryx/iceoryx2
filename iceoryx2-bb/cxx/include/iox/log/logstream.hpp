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

#ifndef IOX_HOOFS_REPORTING_LOG_LOGSTREAM_HPP
#define IOX_HOOFS_REPORTING_LOG_LOGSTREAM_HPP

#include "iox/log/logger.hpp"
#include "iox/type_traits.hpp"

#include <string>

namespace iox
{
namespace log
{
class LogStream;

/// @brief Helper struct to log in hexadecimal format
template <typename T>
class LogHex
{
  public:
    friend class LogStream;

    template <typename TT, typename>
    friend constexpr LogHex<TT> hex(const TT) noexcept;
    friend constexpr LogHex<const void* const> hex(const void* const) noexcept;

    ~LogHex() = default;

    LogHex(const LogHex&) = delete;
    LogHex& operator=(const LogHex&) = delete;
    LogHex& operator=(LogHex&&) noexcept = delete;

  private:
    template <typename = std::enable_if_t<std::is_arithmetic<T>::value || std::is_pointer<T>::value>>
    explicit constexpr LogHex(const T value) noexcept;

    LogHex(LogHex&&) noexcept = default;

    // NOLINTJUSTIFICATION intentionally used since the class does not need to be assignable and the data is not supposed to be changed
    // NOLINTNEXTLINE(cppcoreguidelines-avoid-const-or-ref-data-members)
    T m_value;
};

/// @brief Log a number in hexadecimal format
/// @tparam[in] T the arithmetic data type of the value to log
/// @param[in] value to be logged
/// @return a helper struct which will be used by the LogStream
// AXIVION Next Construct AutosarC++19_03-M17.0.3 : The function is in the iox::log namespace which prevents easy misuse
template <typename T, typename = std::enable_if_t<std::is_arithmetic<T>::value && !std::is_same<T, bool>::value>>
constexpr LogHex<T> hex(const T value) noexcept;

/// @brief Log a pointer in hexadecimal format
/// @param[in] ptr is the pointer to be logged
/// @return a helper struct which will be used by the LogStream
// AXIVION Next Construct AutosarC++19_03-M17.0.3 : The function is in the iox::log namespace which prevents easy misuse
constexpr LogHex<const void* const> hex(const void* const ptr) noexcept;

/// @brief Helper struct to log in octal format
template <typename T>
class LogOct
{
  public:
    friend class LogStream;

    template <typename TT, typename>
    friend constexpr LogOct<TT> oct(const TT) noexcept;

    ~LogOct() = default;

    LogOct(const LogOct&) = delete;
    LogOct& operator=(const LogOct&) = delete;
    LogOct& operator=(LogOct&&) noexcept = delete;

  private:
    template <typename = std::enable_if_t<std::is_integral<T>::value>>
    explicit constexpr LogOct(const T value) noexcept;

    LogOct(LogOct&&) noexcept = default;

    T m_value;
};

/// @brief Log a number in octal format
/// @tparam[in] T the integral data type of the value to log
/// @param[in] value to be logged
/// @return a helper struct which will be used by the LogStream
// AXIVION Next Construct AutosarC++19_03-M17.0.3 : The function is in the iox::log namespace which prevents easy misuse
template <typename T, typename = std::enable_if_t<std::is_integral<T>::value && !std::is_same<T, bool>::value>>
constexpr LogOct<T> oct(const T value) noexcept;

/// @brief Helper struct to log in binary format
template <typename T>
class LogBin
{
  public:
    friend class LogStream;

    template <typename TT, typename>
    friend constexpr LogBin<TT> bin(const TT) noexcept;

    ~LogBin() = default;

    LogBin(const LogBin&) = delete;
    LogBin& operator=(const LogBin&) = delete;
    LogBin& operator=(LogBin&&) noexcept = delete;

  private:
    template <typename = std::enable_if_t<std::is_integral<T>::value>>
    explicit constexpr LogBin(const T value) noexcept;

    LogBin(LogBin&&) noexcept = default;

    T m_value;
};

/// @brief Log a number in binary format
/// @tparam[in] T the integral data type of the value to log
/// @param[in] value to be logged
/// @return a helper struct which will be used by the LogStream
// AXIVION Next Construct AutosarC++19_03-M17.0.3 : The function is in the iox::log namespace which prevents easy misuse
template <typename T, typename = std::enable_if_t<std::is_integral<T>::value && !std::is_same<T, bool>::value>>
constexpr LogBin<T> bin(const T value) noexcept;

/// @brief Helper struct to log in raw bytes
class LogRaw
{
  public:
    friend class LogStream;

    // for some reasons the default template argument SFINEA results in the error
    // 'redeclaration of friend ... may not have default template arguments' on GCC but return type SFINEA seams to work
    template <typename T>
    friend constexpr typename std::enable_if<!std::is_pointer<T>::value, LogRaw>::type raw(const T&) noexcept;
    friend constexpr LogRaw raw(const void* const, const uint64_t) noexcept;

    ~LogRaw() = default;

    LogRaw(const LogRaw&) = delete;
    LogRaw& operator=(const LogRaw&) = delete;
    LogRaw& operator=(LogRaw&&) noexcept = delete;

  private:
    explicit constexpr LogRaw(const void* const data, uint64_t size) noexcept;

    LogRaw(LogRaw&&) noexcept = default;

    // NOLINTJUSTIFICATION intentionally used since the class does not need to be assignable and the data is not supposed to be changed
    // NOLINTNEXTLINE(cppcoreguidelines-avoid-const-or-ref-data-members)
    const void* const m_data;
    uint64_t m_size;
};

/// @brief Log an object in raw bytes
/// @tparam[in] T the data type of the object to log
/// @param[in] object to be logged
/// @return a helper struct which will be used by the LogStream
// AXIVION Next Construct AutosarC++19_03-M17.0.3 : The function is in the iox::log namespace which prevents easy misuse
template <typename T>
constexpr typename std::enable_if<!std::is_pointer<T>::value, LogRaw>::type raw(const T& object) noexcept;

/// @brief Log data in raw bytes
/// @param[in] data pointer to the data to be logged
/// @param[in] size of the data to be logged
/// @return a helper struct which will be used by the LogStream
// AXIVION Next Construct AutosarC++19_03-M17.0.3 : The function is in the iox::log namespace which prevents easy misuse
constexpr LogRaw raw(const void* const data, const uint64_t size) noexcept;

/// @brief This class provides the public interface to the logger and is used with the 'IOX_LOG' macro. In order to add
/// support for custom data types 'operator<<' needs to be implement for the custom type.
/// @code
/// iox::log::LogStream& operator<<(iox::log::LogStream& stream, const MyType& myType) noexcept
/// {
///     stream << "MyType { member = " << myType.member << " }";
///     return stream;
/// }
/// @endcode
class LogStream
{
  public:
    /// @brief Constructor for a LogStream object with an externally provided logger
    /// @note This is not intended for public use! Use the 'IOX_LOG' macro instead
    /// @param[in] logger to be used by the LogStream instance
    /// @param[in] file the file of the log message. Please use the '__FILE__' compiler intrinsic
    /// @param[in] line the line of the log message. Please use the '__LINE__' compiler intrinsic
    /// @param[in] function the function of the log message. Please use the '__FUNCTION__' compiler intrinsic
    /// @param[in] logLevel is the log level for the log message
    // AXIVION Next Construct AutosarC++19_03-A3.9.1 : file, line and function are used in conjunction with '__FILE__',
    // '__LINE__' and '__FUNCTION__'; these are compiler intrinsic and cannot be changed to fixed width types in a
    // platform agnostic way
    LogStream(Logger& logger, const char* file, const int line, const char* function, LogLevel logLevel) noexcept;

    /// @brief Constructor for a LogStream object with the logger from iox::log::Logger::get
    /// @note This is not intended for public use! Use the 'IOX_LOG' macro instead
    /// @param[in] file the file of the log message. Please use the '__FILE__' compiler intrinsic
    /// @param[in] line the line of the log message. Please use the '__LINE__' compiler intrinsic
    /// @param[in] function the function of the log message. Please use the '__FUNCTION__' compiler intrinsic
    /// @param[in] logLevel is the log level for the log message
    // AXIVION Next Construct AutosarC++19_03-A3.9.1 : file, line and function are used in conjunction with '__FILE__',
    // '__LINE__' and '__FUNCTION__'; these are compiler intrinsic and cannot be changed to fixed width types in a
    // platform agnostic way
    LogStream(const char* file, const int line, const char* function, LogLevel logLevel) noexcept;

    virtual ~LogStream() noexcept;

    LogStream(const LogStream&) = delete;
    LogStream(LogStream&&) = delete;

    LogStream& operator=(const LogStream&) = delete;
    LogStream& operator=(LogStream&&) = delete;

    /// @brief Helper function to convert a r-value reference into a l-value reference in order to be able to extend the
    /// data types which can be logged with LogStream object. Without this, one would need to always log a builtin type
    /// before a custom type could be logged
    /// @return a reference to the LogStream instance
    LogStream& self() noexcept;

    /// @brief Logging support for C-style strings
    /// @param[in] cstr is the C-style string to log
    /// @return a reference to the LogStream instance
    // AXIVION Next Construct AutosarC++19_03-A3.9.1 : Logging support for C-style strings
    LogStream& operator<<(const char* cstr) noexcept;

    /// @brief Logging support for std::string
    /// @param[in] str is the std::string to log
    /// @return a reference to the LogStream instance
    /// @todo iox-#1755 instead of using std::string we could also accept everything with a c_str() method
    /// and avoid the std::string dependency; alternatively this could be implemented as free function
    LogStream& operator<<(const std::string& str) noexcept;

    /// @brief Logging support for 'boolean'
    /// @param[in] val is the 'boolean' to log
    /// @return a reference to the LogStream instance
    LogStream& operator<<(const bool val) noexcept;

    // AXIVION DISABLE STYLE AutosarC++19_03-A3.9.1 : Basic numeric types are used in order to cover als basic numeric types, independent of the type alias

    /// @brief Logging support for 'char'
    /// @param[in] val is the 'char' to log
    /// @return a reference to the LogStream instance
    LogStream& operator<<(const char val) noexcept;

    /// @brief Logging support for 'signed char'
    /// @param[in] val is the 'signed char' to log
    /// @return a reference to the LogStream instance
    LogStream& operator<<(const signed char val) noexcept;

    /// @brief Logging support for 'unsigned char'
    /// @param[in] val is the 'unsigned char' to log
    /// @return a reference to the LogStream instance
    LogStream& operator<<(const unsigned char val) noexcept;

    /// @brief Logging support for 'short'
    /// @param[in] val is the 'short' to log
    /// @return a reference to the LogStream instance
    LogStream& operator<<(const short val) noexcept;

    /// @brief Logging support for 'unsigned short'
    /// @param[in] val is the 'unsigned short' to log
    /// @return a reference to the LogStream instance
    LogStream& operator<<(const unsigned short val) noexcept;

    /// @brief Logging support for 'int'
    /// @param[in] val is the 'int' to log
    /// @return a reference to the LogStream instance
    LogStream& operator<<(const int val) noexcept;

    /// @brief Logging support for 'unsigned int'
    /// @param[in] val is the 'unsigned int' to log
    /// @return a reference to the LogStream instance
    LogStream& operator<<(const unsigned int val) noexcept;

    /// @brief Logging support for 'long'
    /// @param[in] val is the 'long' to log
    /// @return a reference to the LogStream instance
    LogStream& operator<<(const long val) noexcept;

    /// @brief Logging support for 'unsigned long'
    /// @param[in] val is the 'unsigned long' to log
    /// @return a reference to the LogStream instance
    LogStream& operator<<(const unsigned long val) noexcept;

    /// @brief Logging support for 'long long'
    /// @param[in] val is the 'long long' to log
    /// @return a reference to the LogStream instance
    LogStream& operator<<(const long long val) noexcept;

    /// @brief Logging support for 'unsigned long long'
    /// @param[in] val is the 'unsigned long long' to log
    /// @return a reference to the LogStream instance
    LogStream& operator<<(const unsigned long long val) noexcept;

    /// @brief Logging support for 'float'
    /// @param[in] val is the 'float' to log
    /// @return a reference to the LogStream instance
    LogStream& operator<<(const float val) noexcept;

    /// @brief Logging support for 'double'
    /// @param[in] val is the 'double' to log
    /// @return a reference to the LogStream instance
    LogStream& operator<<(const double val) noexcept;

    /// @brief Logging support for 'long double'
    /// @param[in] val is the 'long double' to log
    /// @return a reference to the LogStream instance
    LogStream& operator<<(const long double val) noexcept;

    // AXIVION ENABLE STYLE AutosarC++19_03-A3.9.1

    /// @brief Logging support for integral numbers in hexadecimal format
    /// @tparam[in] T is the integral data type of the value to log
    /// @param[in] val is the value to log
    /// @return a reference to the LogStream instance
    template <typename T, typename std::enable_if_t<std::is_integral<T>::value, bool> = 0>
    LogStream& operator<<(const LogHex<T>&& val) noexcept;

    /// @brief Logging support for floating point numbers in hexadecimal format
    /// @tparam[in] T is the floating point data type of the value to log
    /// @param[in] val is the value to log
    /// @return a reference to the LogStream instance
    template <typename T, typename std::enable_if_t<std::is_floating_point<T>::value, bool> = 0>
    LogStream& operator<<(const LogHex<T>&& val) noexcept;

    /// @brief Logging support for pointer in hexadecimal format
    /// @param[in] val is the pointer to log
    /// @return a reference to the LogStream instance
    LogStream& operator<<(const LogHex<const void* const>&& val) noexcept;

    /// @brief Logging support for integral numbers in octal format
    /// @tparam[in] T is the integral data type of the value to log
    /// @param[in] val is the value to log
    /// @return a reference to the LogStream instance
    template <typename T, typename std::enable_if_t<std::is_integral<T>::value, bool> = 0>
    LogStream& operator<<(const LogOct<T>&& val) noexcept;

    /// @brief Logging support for integral numbers in binary format
    /// @tparam[in] T is the integral data type of the value to log
    /// @param[in] val is the value to log
    /// @return a reference to the LogStream instance
    template <typename T, typename std::enable_if_t<std::is_integral<T>::value, bool> = 0>
    LogStream& operator<<(const LogBin<T>&& val) noexcept;

    /// @brief Logging support for data in raw bytes
    /// @param[in] val is the value to log
    /// @return a reference to the LogStream instance
    LogStream& operator<<(const LogRaw&& val) noexcept;

    /// @brief Logging support for callable. This gives access to the LogStream instance which e.g. can be used in a
    /// loop
    /// @tparam[in] Callable with a signature 'iox::log::LogStream&(iox::log::LogStream&)'
    /// @param[in] c is the callable which receives a LogStream object for the actual logging
    /// @code
    /// IOX_LOG(Info, [] (auto& stream) -> auto& {
    ///     for(const auto& num: {13, 37, 42, 73}) {
    ///         stream << num << " ";
    ///     }
    ///     return stream;
    /// });
    /// @endcode
    template <typename Callable, typename = std::enable_if_t<is_invocable_r<LogStream&, Callable, LogStream&>::value>>
    LogStream& operator<<(const Callable& c) noexcept;

    /// @brief Logging support for LogLevel
    /// @param[in] value is the LogLevel to log
    /// @return a reference to the LogStream instance
    LogStream& operator<<(const LogLevel value) noexcept;

  private:
    void flush() noexcept;

    // JUSTIFICATION it is fine to use a reference since the LogStream object is intentionally not movable
    // NOLINTNEXTLINE(cppcoreguidelines-avoid-const-or-ref-data-members)
    Logger& m_logger;
    bool m_isFlushed{false};

    /// @todo iox-#1755 workaround due to deactivation of lazy evaluation
    bool m_doFlush{true};
};

} // namespace log
} // namespace iox

#include "iox/detail/log/logstream.inl"

#endif // IOX_HOOFS_REPORTING_LOG_LOGSTREAM_HPP
