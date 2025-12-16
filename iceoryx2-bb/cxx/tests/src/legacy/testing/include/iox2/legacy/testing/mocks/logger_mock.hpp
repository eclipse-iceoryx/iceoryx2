// Copyright (c) 2019 by Robert Bosch GmbH. All rights reserved.
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

#ifndef IOX2_BB_MOCKS_LOGGER_MOCK_HPP
#define IOX2_BB_MOCKS_LOGGER_MOCK_HPP

#include "iox2/legacy/log/logger.hpp"
#include "iox2/legacy/log/logstream.hpp"
#include "iox2/legacy/logging.hpp"

#include <mutex>
#include <vector>

namespace iox2 {
namespace legacy {
namespace testing {
// NOLINTNEXTLINE(cppcoreguidelines-macro-usage) required to be able to easily test custom types
#define IOX2_LOGSTREAM_MOCK(logger)                                                                                    \
    iox2::legacy::log::LogStream((logger), "file", 42, "function", iox2::legacy::log::LogLevel::Trace).self()

/// @brief This mock can be used to test implementations of LogStream::operator<< for custom types. It should be used
/// with the 'IOX2_LOGSTREAM_MOCK' macro
/// @code
/// iox2::legacy::testing::Logger_Mock loggerMock;
///
/// MyType sut;
/// IOX2_LOGSTREAM_MOCK(loggerMock) << sut;
///
/// ASSERT_THAT(loggerMock.logs.size(), Eq(1U));
/// EXPECT_THAT(loggerMock.logs[0].message, StrEq(EXPECTED_STRING_REPRESENTATION);
/// @endcode
class Logger_Mock : public log::TestingLoggerBase {
    using Base = log::TestingLoggerBase;

  public:
    Logger_Mock() noexcept = default;

    struct LogEntry {
        std::string file;
        int line { 0 };
        std::string function;
        log::LogLevel logLevel { iox2::legacy::log::LogLevel::Off };
        std::string message;
    };

    std::vector<LogEntry> logs;

  private:
    /// @brief Overrides the base implementation to store the
    void createLogMessageHeader(const char* file,
                                const int line,
                                const char* function,
                                log::LogLevel logLevel) noexcept override {
        Base::assumeFlushed();

        LogEntry logEntry;
        logEntry.file = file;
        logEntry.line = line;
        logEntry.function = function;
        logEntry.logLevel = logLevel;

        logs.emplace_back(std::move(logEntry));
    }

    void flush() noexcept override {
        const auto logBuffer = Base::getLogBuffer();
        logs.back().message = logBuffer.buffer;
        Base::assumeFlushed();
    }
};

} // namespace testing
} // namespace legacy
} // namespace iox2

#endif // IOX2_BB_MOCKS_LOGGER_MOCK_HPP
