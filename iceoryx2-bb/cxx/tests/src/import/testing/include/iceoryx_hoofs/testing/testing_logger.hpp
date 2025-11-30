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

#ifndef IOX_HOOFS_TESTING_TESTING_LOGGER_HPP
#define IOX_HOOFS_TESTING_TESTING_LOGGER_HPP

#include "iox/log/logger.hpp"

#include <gmock/gmock.h>
#include <gtest/gtest.h>

#include <functional>
#include <mutex>

namespace iox {
namespace testing {
/// @brief This logger is used for tests. It caches all the log messages and prints them to the console when a test
/// fails. For debug purposes this behaviour can be overwritten with the 'IOX_TESTING_ALLOW_LOG' environment variable,
/// e.g. 'IOX_TESTING_ALLOW_LOG=ON ./hoofs_moduletests --gtest_filter=SharedMemoryObject_Test\*'. Furthermore, it can
/// also be used to check for the occurrence on specific log messages, e.g. when a function is expected to log an error.
/// @code
/// callToFunctionWhichLogsAnError();
/// iox::testing::TestingLogger::checkLogMessageIfLogLevelIsSupported(iox::log::LogLevel::Error, [](const auto&
/// logMessages){
///     ASSERT_THAT(logMessages.size(), Eq(1U));
///     EXPECT_THAT(logMessages[0], HasSubstr(expectedOutput));
/// });
/// @endcode
class TestingLogger : public log::TestingLoggerBase {
    using Base = log::TestingLoggerBase;

  public:
    ~TestingLogger() override = default;

    TestingLogger(const TestingLogger&) = delete;
    TestingLogger(TestingLogger&&) = delete;

    TestingLogger& operator=(const TestingLogger&) = delete;
    TestingLogger& operator=(TestingLogger&&) = delete;

    /// @brief Initialized the logger. This should be called in the main function of the test binary
    /// @code
    /// #include "iceoryx_hoofs/testing/testing_logger.hpp"
    ///
    /// #include "test.hpp"
    ///
    /// int main(int argc, char* argv[])
    /// {
    ///     ::testing::InitGoogleTest(&argc, argv);
    ///
    ///     iox::testing::TestingLogger::init();
    ///
    ///     return RUN_ALL_TESTS();
    /// }
    /// @endcode
    static void init() noexcept;

    /// @brief Removes all log messages from the internal cache. This is automatically done at the start of each test.
    void clearLogBuffer() noexcept;

    /// @brief Prints all log messages from the internal cache. This is automatically done at the end of a failed test.
    void printLogBuffer() noexcept;

    /// @brief Number of caches log messages
    /// @return the number of the log messages from the internal cache
    /// @note This can be used in tests which check for a specific log output
    static uint64_t getNumberOfLogMessages() noexcept;

    /// @brief Runs the provided checker function for the collected log messages
    /// @note This can be used in tests to verify the collected log messages
    static void checkLogMessageIfLogLevelIsSupported(iox::log::LogLevel logLevel,
                                                     const std::function<void(const std::vector<std::string>&)>& check);

    /// @brief Checks if the the LogLevel is above the minimal supported LogLevel compiled into the binary
    /// @param[in] logLevel is the log level to check if it is supported
    /// @return true if the log level support is compiled into the binary, false otherwise
    /// @note This can be used in tests which check for a specific log output
    static inline constexpr bool doesLoggerSupportLogLevel(const log::LogLevel logLevel) noexcept {
        return log::MINIMAL_LOG_LEVEL >= logLevel;
    }

  private:
    TestingLogger() noexcept = default;

    void flush() noexcept override;
    static std::vector<std::string> getLogMessages() noexcept;

    struct LoggerData {
        std::vector<std::string> buffer;
        bool allowLog { true };
    };

    std::mutex m_loggerDataLock;
    LoggerData m_loggerData;
};

/// @brief This class hooks into gTest to automatically clear the log messages on the start of a test an print the
/// cached log messages from failed tests
class LogPrinter : public ::testing::EmptyTestEventListener {
    void OnTestStart(const ::testing::TestInfo&) override;
    void OnTestPartResult(const ::testing::TestPartResult& result) override;
};

} // namespace testing
} // namespace iox

#endif // IOX_HOOFS_TESTING_TESTING_LOGGER_HPP
