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

#include "iox2/legacy/logging.hpp"

#include "iox2/legacy/testing/testing_logger.hpp"

#include <gmock/gmock.h>
#include <gtest/gtest.h>

namespace {
using namespace ::testing;

void testLogLevelThreshold(const iox2::legacy::log::LogLevel loggerLogLevel,
                           const std::function<void(iox2::legacy::log::LogLevel)>& loggerCall) {
    iox2::legacy::log::Logger::setLogLevel(loggerLogLevel);

    struct LogLevel {
        LogLevel(iox2::legacy::log::LogLevel logLevel, std::string str)
            : value(logLevel)
            , string(std::move(str)) {
        }
        iox2::legacy::log::LogLevel value;
        std::string string;
    };

    const std::initializer_list<LogLevel> logEntryLogLevels {
        { iox2::legacy::log::LogLevel::Fatal, "Fatal" }, { iox2::legacy::log::LogLevel::Error, "Error" },
        { iox2::legacy::log::LogLevel::Warn, "Warn" },   { iox2::legacy::log::LogLevel::Info, "Info" },
        { iox2::legacy::log::LogLevel::Debug, "Debug" }, { iox2::legacy::log::LogLevel::Trace, "Trace" }
    };

    for (const auto& logEntryLogLevel : logEntryLogLevels) {
        if (!iox2::legacy::testing::TestingLogger::doesLoggerSupportLogLevel(logEntryLogLevel.value)) {
            continue;
        }

        dynamic_cast<iox2::legacy::testing::TestingLogger&>(iox2::legacy::log::Logger::get()).clearLogBuffer();
        loggerCall(logEntryLogLevel.value);

        if (logEntryLogLevel.value <= loggerLogLevel) {
            ASSERT_THAT(iox2::legacy::testing::TestingLogger::getNumberOfLogMessages(), Eq(1U));
            iox2::legacy::testing::TestingLogger::checkLogMessageIfLogLevelIsSupported(
                logEntryLogLevel.value, [&](const auto& logMessages) {
                    const auto& logMessage = logMessages.back();
                    EXPECT_THAT(logMessage.find(logEntryLogLevel.string), Ne(std::string::npos));
                });
        } else {
            ASSERT_THAT(iox2::legacy::testing::TestingLogger::getNumberOfLogMessages(), Eq(0U));
        }
    }
}

TEST(LoggingLogLevelThreshold_test, LogLevel) {
    ::testing::Test::RecordProperty("TEST_ID", "829a6634-43be-4fa4-94bf-18d53ce816a9");

    GTEST_SKIP() << "This test will probably deleted once the log level is set via iceoryx2-bb-log in Rust";

    for (const auto loggerLogLevel : { iox2::legacy::log::LogLevel::Off,
                                       iox2::legacy::log::LogLevel::Fatal,
                                       iox2::legacy::log::LogLevel::Error,
                                       iox2::legacy::log::LogLevel::Warn,
                                       iox2::legacy::log::LogLevel::Info,
                                       iox2::legacy::log::LogLevel::Debug,
                                       iox2::legacy::log::LogLevel::Trace }) {
        SCOPED_TRACE(std::string("Logger LogLevel: ") + iox2::legacy::log::asStringLiteral(loggerLogLevel));

        testLogLevelThreshold(loggerLogLevel, [](auto logLevel) { IOX2_LOG_INTERNAL("", 0, "", logLevel, ""); });
    }
}

} // namespace
