// Copyright (c) 2019 by Robert Bosch GmbH. All rights reserved.
// Copyright (c) 2021 - 2022 by Apex.AI Inc. All rights reserved.
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

#ifndef IOX2_BB_REPORTING_LOG_BUILDING_BLOCKS_LOGGER_HPP
#define IOX2_BB_REPORTING_LOG_BUILDING_BLOCKS_LOGGER_HPP

#include "iox2/legacy/atomic.hpp"

#include <cstdint>
#include <cstring>
#include <mutex>

namespace iox2 {
namespace legacy {
namespace log {
class LogStream;

/// @brief This enum defines the log levels used for logging.
enum class LogLevel : uint8_t {
    Off = 0,
    Fatal,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
};

/// @brief converts LogLevel into a string literal
/// @param[in] value the LogLevel to convert
/// @return string literal of the LogLevel value
// AXIVION Next Construct AutosarC++19_03-A3.9.1 : This function return a string literal
// which corresponds to a const char *
constexpr const char* asStringLiteral(const LogLevel value) noexcept;

/// @todo iox-#1755 move this to e.g. helplets once we are able to depend on on it
/// @brief Compares C-style strings with a char array, i.g. string literal for equality
/// @tparam[in] N size of the char array
/// @param[in] lhs C-style string to compare
/// @param[in] rhs char array to compare
/// @return true if the strings are equal, false otherwise
template <uint32_t N>
// NOLINTJUSTIFICATION required for C-style string comparison; safety guaranteed by strncmp
// NOLINTNEXTLINE(hicpp-avoid-c-arrays, cppcoreguidelines-avoid-c-arrays)
bool equalStrings(const char* lhs, const char (&rhs)[N]) noexcept;

/// @brief Tries to get the log level from the 'IOX2_LOG_LEVEL' env variable or uses the specified one if the env
/// variable is not set
/// @param[in] logLevel is the log level to be used when the env variable is not set
/// @note The function uses 'getenv' which is not thread safe and can result in undefined behavior when it is called
/// from multiple threads or the env variable is changed while the function holds a pointer to the data. For this reason
/// the function should only be used in the startup phase of the application and only in the main thread.
LogLevel logLevelFromEnvOr(const LogLevel logLevel) noexcept;

namespace internal {
/// @brief This class acts as common interface for the Logger. It provides the common functionality and inherits from
/// the BaseLogger which is provided as template parameter. Please have a look at the design document for more details.
/// @tparam[in] BaseLogger is the actual implementation
template <typename BaseLogger>
class Logger : public BaseLogger {
  public:
    friend class log::LogStream;

    Logger() = default;

    Logger(const Logger&) = delete;
    Logger(Logger&&) = delete;

    Logger& operator=(const Logger&) = delete;
    Logger& operator=(Logger&&) = delete;

    ~Logger() = default;

    /// @brief Access to the logger singleton instance
    /// @return a reference to the active logger
    static Logger& get() noexcept;

    /// @brief Initializes the logger
    /// @param[in] logLevel the log level which will be used to determine which messages will be logged. By default it
    /// is everything with a log level higher than specified by the 'IOX2_LOG_LEVEL' environment variable or equal to
    /// 'INFO' if the environment variable is not set.
    /// @note The function uses 'getenv' which is not thread safe and can result in undefined behavior when it is called
    /// from multiple threads or the env variable is changed while the function holds a pointer to the data. For this
    /// reason the function should only be used in the startup phase of the application and only in the main thread.
    static void init(const LogLevel logLevel = logLevelFromEnvOr(LogLevel::Info)) noexcept;

    /// @brief Replaces the default logger with the specified one
    /// @param[in] newLogger is the logger which shall be used after the call
    /// @note this must be called before 'init'. If this is called after 'init' or called multiple times, the current
    /// logger will not be replaced and an error message will be logged in the current and the provided new logger.
    static void setActiveLogger(Logger& newLogger) noexcept;

  private:
    static Logger& activeLogger(Logger* newLogger = nullptr) noexcept;

    void initLoggerInternal(const LogLevel logLevel) noexcept;

  private:
    concurrent::Atomic<bool> m_isActive { true };
    concurrent::Atomic<bool> m_isFinalized { false };
};

} // namespace internal
} // namespace log
} // namespace legacy
} // namespace iox2

#include "iox2/legacy/detail/log/building_blocks/logger.inl"

#endif // IOX2_BB_REPORTING_LOG_BUILDING_BLOCKS_LOGGER_HPP
