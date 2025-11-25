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

#ifndef IOX_HOOFS_REPORTING_LOG_BUILDING_BLOCKS_CONSOLE_LOGGER_HPP
#define IOX_HOOFS_REPORTING_LOG_BUILDING_BLOCKS_CONSOLE_LOGGER_HPP

#include "iox/atomic.hpp"
#include "iox/iceoryx_hoofs_types.hpp"
#include "iox/log/building_blocks/logformat.hpp"

#include <cstdint>
#include <cstdio>
#include <mutex>

namespace iox
{
namespace log
{
/// @brief A minimal logger implementation which outputs the log messages to the console
class ConsoleLogger
{
  public:
    /// @brief Obtain the current log level
    /// @return the current log level
    /// @note In case this class is used as template for a custom logger implementation keep in mind that this method
    /// must not have any side effects
    /// @todo iox-#1755 update the design document with the requirement that this method must not have side effects
    static LogLevel getLogLevel() noexcept;

    /// @brief Sets a new log level
    /// @param[in] logLevel to be used after the call
    static void setLogLevel(const LogLevel logLevel) noexcept;

    virtual ~ConsoleLogger() = default;

    ConsoleLogger(const ConsoleLogger&) = delete;
    ConsoleLogger(ConsoleLogger&&) = delete;

    ConsoleLogger& operator=(const ConsoleLogger&) = delete;
    ConsoleLogger& operator=(ConsoleLogger&&) = delete;

  protected:
    ConsoleLogger() noexcept = default;

    virtual void initLogger(const LogLevel) noexcept;

    // AXIVION Next Construct AutosarC++19_03-A3.9.1 : file, line and function are used in conjunction with '__FILE__',
    // '__LINE__' and '__FUNCTION__'; these are compiler intrinsic and cannot be changed to fixed width types in a
    // platform agnostic way
    virtual void
    createLogMessageHeader(const char* file, const int line, const char* function, LogLevel logLevel) noexcept;

    virtual void flush() noexcept;

    LogBuffer getLogBuffer() const noexcept;

    void assumeFlushed() noexcept;

    // AXIVION Next Construct AutosarC++19_03-A3.9.1 : Not used as an integer but a low-level C-style string
    void logString(const char* message) noexcept;

    void logChar(const char value) noexcept;

    void logBool(const bool value) noexcept;

    template <typename T, typename std::enable_if_t<std::is_arithmetic<T>::value, bool> = 0>
    void logDec(const T value) noexcept;

    template <typename T,
              typename std::enable_if_t<(std::is_integral<T>::value && std::is_unsigned<T>::value)
                                            || std::is_floating_point<T>::value || std::is_pointer<T>::value,
                                        bool> = 0>
    void logHex(const T value) noexcept;

    template <typename T, typename std::enable_if_t<std::is_integral<T>::value && std::is_unsigned<T>::value, bool> = 0>
    void logOct(const T value) noexcept;

    template <typename T, typename std::enable_if_t<std::is_integral<T>::value && std::is_unsigned<T>::value, bool> = 0>
    void logBin(const T value) noexcept;

    void logRaw(const void* const data, const uint64_t size) noexcept;

  private:
    // AXIVION Next Construct AutosarC++19_03-A3.9.1 : Not used as an integer but as actual character
    // AXIVION Next Construct AutosarC++19_03-A18.1.1 : C-style array is used to acquire size of the array safely. Safe
    // access is guaranteed since the char array is not accessed but only the size is obtained
    template <uint32_t N>
    // NOLINTNEXTLINE(hicpp-avoid-c-arrays, cppcoreguidelines-avoid-c-arrays)
    static constexpr uint32_t bufferSize(const char (&)[N]) noexcept;

    template <typename T>
    static constexpr void unused(T&&) noexcept;

    // AXIVION Next Construct AutosarC++19_03-A3.9.1 : Not used as an integer but format string literal
    template <typename T>
    static void logArithmetic(const T value, const char* format) noexcept;

    struct ThreadLocalData final
    {
        ThreadLocalData() noexcept = default;
        ~ThreadLocalData() = default;

        ThreadLocalData(const ThreadLocalData&) = delete;
        ThreadLocalData(ThreadLocalData&&) = delete;

        ThreadLocalData& operator=(const ThreadLocalData&) = delete;
        ThreadLocalData& operator=(ThreadLocalData&&) = delete;

        /// @todo iox-#1755 this could be made a compile time option
        static constexpr uint32_t BUFFER_SIZE{1024};
        static constexpr uint32_t NULL_TERMINATED_BUFFER_SIZE{BUFFER_SIZE + 1};

        // AXIVION Next Construct AutosarC++19_03-A3.9.1 : Not used as an integer but as actual character
        // AXIVION Next Construct AutosarC++19_03-A18.1.1 : This is a low-level component with minimal dependencies.
        // Safe access is guaranteed since the char array is wrapped inside the class
        // NOLINTNEXTLINE(hicpp-avoid-c-arrays, cppcoreguidelines-avoid-c-arrays)
        char buffer[NULL_TERMINATED_BUFFER_SIZE];
        uint32_t bufferWriteIndex; // initialized in corresponding cpp file
        /// @todo iox-#1755 add thread local storage with thread id and print it in the log messages
    };

    static ThreadLocalData& getThreadLocalData() noexcept;

  private:
    // NOLINTJUSTIFICATION needed for the functionality and a private member of the class
    // NOLINTNEXTLINE(cppcoreguidelines-avoid-non-const-global-variables)
    static concurrent::Atomic<LogLevel> s_activeLogLevel; // initialized in corresponding cpp file
};

} // namespace log
} // namespace iox

#include "iox/detail/log/building_blocks/console_logger.inl"

#endif // IOX_HOOFS_REPORTING_LOG_BUILDING_BLOCKS_CONSOLE_LOGGER_HPP
