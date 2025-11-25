// Copyright (c) 2019 by Robert Bosch GmbH. All rights reserved.
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
#ifndef IOX_HOOFS_MOCKS_LOGGER_MOCK_HPP
#define IOX_HOOFS_MOCKS_LOGGER_MOCK_HPP

#include "iox/log/logger.hpp"
#include "iox/log/logstream.hpp"
#include "iox/logging.hpp"
#include "iox/optional.hpp"

#include <mutex>
#include <vector>

namespace iox
{
namespace testing
{
// NOLINTNEXTLINE(cppcoreguidelines-macro-usage) required to be able to easily test custom types
#define IOX_LOGSTREAM_MOCK(logger)                                                                                     \
    iox::log::LogStream((logger), "file", 42, "function", iox::log::LogLevel::Trace).self()

/// @brief This mock can be used to test implementations of LogStream::operator<< for custom types. It should be used
/// with the 'IOX_LOGSTREAM_MOCK' macro
/// @code
/// iox::testing::Logger_Mock loggerMock;
///
/// MyType sut;
/// IOX_LOGSTREAM_MOCK(loggerMock) << sut;
///
/// ASSERT_THAT(loggerMock.logs.size(), Eq(1U));
/// EXPECT_THAT(loggerMock.logs[0].message, StrEq(EXPECTED_STRING_REPRESENTATION);
/// @endcode
class Logger_Mock : public log::TestingLoggerBase
{
    using Base = log::TestingLoggerBase;

  public:
    Logger_Mock() noexcept = default;

    struct LogEntry
    {
        std::string file;
        int line{0};
        std::string function;
        log::LogLevel logLevel{iox::log::LogLevel::Off};
        std::string message;
    };

    std::vector<LogEntry> logs;

  private:
    /// @brief Overrides the base implementation to store the
    void createLogMessageHeader(const char* file,
                                const int line,
                                const char* function,
                                log::LogLevel logLevel) noexcept override
    {
        Base::assumeFlushed();

        LogEntry logEntry;
        logEntry.file = file;
        logEntry.line = line;
        logEntry.function = function;
        logEntry.logLevel = logLevel;

        logs.emplace_back(std::move(logEntry));
    }

    void flush() noexcept override
    {
        const auto logBuffer = Base::getLogBuffer();
        logs.back().message = logBuffer.buffer;
        Base::assumeFlushed();
    }
};

} // namespace testing
} // namespace iox

#endif // IOX_HOOFS_MOCKS_LOGGER_MOCK_HPP
