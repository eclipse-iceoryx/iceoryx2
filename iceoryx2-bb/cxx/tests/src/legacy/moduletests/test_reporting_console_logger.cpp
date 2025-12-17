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

#include "iox2/legacy/log/building_blocks/console_logger.hpp"

#include "iox2/legacy/logging.hpp"

#include <gmock/gmock.h>
#include <gtest/gtest.h>

#include <cstdio>
#include <iostream>

namespace {
using namespace ::testing;

class LoggerSUT : public iox2::legacy::log::ConsoleLogger {
  public:
    using iox2::legacy::log::ConsoleLogger::flush;
    using iox2::legacy::log::ConsoleLogger::logString;
};

TEST(ConsoleLogger_test, TestOutput) {
    ::testing::Test::RecordProperty("TEST_ID", "67f1dac5-b425-414a-9690-268ecb06c1ee");

    GTEST_SKIP() << "This is tested via the integration tests by launch testing waiting for the 'RouDi is ready for "
                    "clients' string";
}

/// @note the actual log API is tested via the LogStream tests

TEST(ConsoleLogger_test, SettingTheLogLevelWorks) {
    ::testing::Test::RecordProperty("TEST_ID", "e8225d29-ee35-4864-8528-b1e290a83311");

    GTEST_SKIP() << "This test will probably deleted once the log level is set via iceoryx2-bb-log in Rust";

    constexpr auto LOG_LEVEL { iox2::legacy::log::LogLevel::Info };
    EXPECT_THAT(LoggerSUT::getLogLevel(), Ne(LOG_LEVEL));

    LoggerSUT::setLogLevel(LOG_LEVEL);
    EXPECT_THAT(LoggerSUT::getLogLevel(), Eq(LOG_LEVEL));
}

} // namespace
