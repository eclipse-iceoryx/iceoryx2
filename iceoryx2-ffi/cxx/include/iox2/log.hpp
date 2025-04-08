// Copyright (c) 2024 Contributors to the Eclipse Foundation
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

#ifndef IOX2_LOG_HPP
#define IOX2_LOG_HPP

#include "iox2/log_level.hpp"

namespace iox2 {

/// The abstract base class every custom logger has to implement.
///
/// # Example
///
/// @code
/// class ConsoleLogger : public Log {
///   public:
///     void log(LogLevel log_level, const char* origin, const char* message) override {
///         std::cout << "origin = " << origin << ", message = " << message << std::endl;
///     }
/// };
///
/// static ConsoleLogger CUSTOM_LOGGER = ConsoleLogger();
///
/// set_logger(CUSTOM_LOGGER);
/// @endcode
class Log {
  public:
    Log() = default;
    Log(const Log&) = default;
    Log(Log&&) = default;
    auto operator=(const Log&) -> Log& = default;
    auto operator=(Log&&) -> Log& = default;
    virtual ~Log() = default;

    /// The actual log method. The system provides the log level, the origin of the message and
    /// the actual message.
    virtual void log(LogLevel log_level, const char* origin, const char* message) = 0;
};

/// Adds a log message to the logger.
void log(LogLevel log_level, const char* origin, const char* message);

/// Sets the console logger as default logger. Returns true if the logger was set, otherwise false.
auto use_console_logger() -> bool;

/// Sets the file logger as default logger. Returns true if the logger was set, otherwise false.
auto use_file_logger(const char* log_file) -> bool;

/// Sets the logger that shall be used. This function can only be called once and must be called
/// before any log message was created.
/// It returns true if the logger was set, otherwise false.
auto set_logger(Log& logger) -> bool;

/// Sets the global log level for the application using `IOX2_LOG_LEVEL` environment variable
/// or defaults it to LogLevel::INFO if variable does not exist.
auto set_log_level_from_env_or_default() -> void;

/// Sets the global log level for the application using `IOX2_LOG_LEVEL` environment variable
/// or sets it to a user-given value if variable does not exist.
auto set_log_level_from_env_or(LogLevel level) -> void;

/// Sets the global log level for the application
auto set_log_level(LogLevel level) -> void;

/// Returns the current global log level of the application
auto get_log_level() -> LogLevel;

} // namespace iox2

#endif
