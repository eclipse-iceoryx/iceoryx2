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

#include "iceoryx_platform/logging.hpp"
#include "iox/logging.hpp"

#include "iceoryx_hoofs/testing/testing_logger.hpp"
#include "test.hpp"

namespace
{
using namespace ::testing;

void testLogLevelThreshold(const iox::log::LogLevel loggerLogLevel,
                           const std::function<void(iox::log::LogLevel)>& loggerCall)
{
    iox::log::Logger::setLogLevel(loggerLogLevel);

    struct LogLevel
    {
        LogLevel(iox::log::LogLevel logLevel, std::string str)
            : value(logLevel)
            , string(std::move(str))
        {
        }
        iox::log::LogLevel value;
        std::string string;
    };

    const std::initializer_list<LogLevel> logEntryLogLevels{{iox::log::LogLevel::Fatal, "Fatal"},
                                                            {iox::log::LogLevel::Error, "Error"},
                                                            {iox::log::LogLevel::Warn, "Warn"},
                                                            {iox::log::LogLevel::Info, "Info"},
                                                            {iox::log::LogLevel::Debug, "Debug"},
                                                            {iox::log::LogLevel::Trace, "Trace"}};

    for (const auto& logEntryLogLevel : logEntryLogLevels)
    {
        if (!iox::testing::TestingLogger::doesLoggerSupportLogLevel(logEntryLogLevel.value))
        {
            continue;
        }

        dynamic_cast<iox::testing::TestingLogger&>(iox::log::Logger::get()).clearLogBuffer();
        loggerCall(logEntryLogLevel.value);

        if (logEntryLogLevel.value <= loggerLogLevel)
        {
            ASSERT_THAT(iox::testing::TestingLogger::getNumberOfLogMessages(), Eq(1U));
            iox::testing::TestingLogger::checkLogMessageIfLogLevelIsSupported(
                logEntryLogLevel.value, [&](const auto& logMessages) {
                    const auto& logMessage = logMessages.back();
                    EXPECT_THAT(logMessage.find(logEntryLogLevel.string), Ne(std::string::npos));
                });
        }
        else
        {
            ASSERT_THAT(iox::testing::TestingLogger::getNumberOfLogMessages(), Eq(0U));
        }
    }
}

TEST(LoggingLogLevelThreshold_test, LogLevel)
{
    ::testing::Test::RecordProperty("TEST_ID", "829a6634-43be-4fa4-94bf-18d53ce816a9");
    for (const auto loggerLogLevel : {iox::log::LogLevel::Off,
                                      iox::log::LogLevel::Fatal,
                                      iox::log::LogLevel::Error,
                                      iox::log::LogLevel::Warn,
                                      iox::log::LogLevel::Info,
                                      iox::log::LogLevel::Debug,
                                      iox::log::LogLevel::Trace})
    {
        SCOPED_TRACE(std::string("Logger LogLevel: ") + iox::log::asStringLiteral(loggerLogLevel));

        testLogLevelThreshold(loggerLogLevel, [](auto logLevel) { IOX_LOG_INTERNAL("", 0, "", logLevel, ""); });
    }
}

TEST(LoggingLogLevelThreshold_test, LogLevelForPlatform)
{
    ::testing::Test::RecordProperty("TEST_ID", "574007ac-62ed-4cd1-95e8-e18a9f20e1e1");
    for (const auto loggerLogLevel : {iox::log::LogLevel::Off,
                                      iox::log::LogLevel::Fatal,
                                      iox::log::LogLevel::Error,
                                      iox::log::LogLevel::Warn,
                                      iox::log::LogLevel::Info,
                                      iox::log::LogLevel::Debug,
                                      iox::log::LogLevel::Trace})
    {
        SCOPED_TRACE(std::string("Logger LogLevel: ") + iox::log::asStringLiteral(loggerLogLevel));

        testLogLevelThreshold(loggerLogLevel, [](auto logLevel) {
            iox_platform_detail_log("", 0, "", static_cast<IceoryxPlatformLogLevel>(logLevel), "");
        });
    }
}

} // namespace
