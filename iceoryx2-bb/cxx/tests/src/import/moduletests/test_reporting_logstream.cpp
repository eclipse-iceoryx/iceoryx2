// Copyright (c) 2019, 2021 by Robert Bosch GmbH. All rights reserved.
// Copyright (c) 2021 by Apex.AI Inc. All rights reserved.
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

#include "test_reporting_logstream.hpp"

#include <array>
#include <bitset>
#include <cstdint>
#include <limits>
#include <sstream>
#include <string>

// NOTE: To speed up compilation and the clang-tidy check, the tests for 'iox::string' are split into multiple files

namespace
{
using namespace ::testing;

class IoxLogStream_test : public IoxLogStreamBase_test
{
};

TEST_F(IoxLogStream_test, CTorDelegatesParameterToLogger)
{
    ::testing::Test::RecordProperty("TEST_ID", "209aadb5-9ea6-4620-a6d1-f8fb2d12b97d");
    constexpr const char* EXPECTED_FILE{"hypnotoad.hpp"};
    constexpr const char* EXPECTED_FUNCTION{"void all_glory_to_the_hypnotoad()"};
    constexpr int EXPECTED_LINE{42};
    constexpr auto EXPECTED_LOG_LEVEL{iox::log::LogLevel::Warn};
    iox::log::LogStream(loggerMock, EXPECTED_FILE, EXPECTED_LINE, EXPECTED_FUNCTION, EXPECTED_LOG_LEVEL) << "";

    ASSERT_THAT(loggerMock.logs.size(), Eq(1U));
    EXPECT_THAT(loggerMock.logs.back().file, StrEq(EXPECTED_FILE));
    EXPECT_THAT(loggerMock.logs.back().function, StrEq(EXPECTED_FUNCTION));
    EXPECT_THAT(loggerMock.logs.back().logLevel, Eq(EXPECTED_LOG_LEVEL));
    EXPECT_THAT(loggerMock.logs.back().message, Eq(""));
}

TEST_F(IoxLogStream_test, UnnamedTemporaryLogStreamObject)
{
    ::testing::Test::RecordProperty("TEST_ID", "b0f2c616-a373-4e6e-8083-c3bff5de58b4");
    const std::string claim = "The answer is ";
    const uint8_t answer = 42;
    const std::string bang = "!";

    LogStreamSut(loggerMock) << claim << answer << bang;

    std::string expected = claim + std::to_string(answer) + bang;

    ASSERT_THAT(loggerMock.logs.size(), Eq(1U));
    EXPECT_THAT(loggerMock.logs[0].message, StrEq(expected));
}

TEST_F(IoxLogStream_test, LocalLogStreamObject)
{
    ::testing::Test::RecordProperty("TEST_ID", "0f4b4d78-e3c4-454a-a839-8f75b288e878");
    const std::string claim = "The answer is ";
    const uint8_t answer = 42;
    const std::string bang = "!";

    {
        LogStreamSut sut(loggerMock);
        sut << claim;
        sut << answer;
        sut << bang;

        // the destructor flushes the log to the logger
        ASSERT_THAT(loggerMock.logs.size(), Eq(1U));
        EXPECT_THAT(loggerMock.logs[0].message, StrEq(""));
    }

    std::string expected = claim + std::to_string(answer) + bang;

    ASSERT_THAT(loggerMock.logs.size(), Eq(1U));
    EXPECT_THAT(loggerMock.logs.back().message, StrEq(expected));
}

TEST_F(IoxLogStream_test, StreamOperatorCStyleString)
{
    ::testing::Test::RecordProperty("TEST_ID", "68b034d7-a424-4e75-b6db-a5c4172ee271");
    std::string logValue("This is the iceoryx logger!");
    const std::string constLogValue{"Nothing to see here, move along!"};
    LogStreamSut(loggerMock) << logValue.c_str();
    LogStreamSut(loggerMock) << constLogValue.c_str();

    ASSERT_THAT(loggerMock.logs.size(), Eq(2U));
    EXPECT_THAT(loggerMock.logs[0].message, StrEq(logValue));
    EXPECT_THAT(loggerMock.logs[1].message, StrEq(constLogValue));
}

TEST_F(IoxLogStream_test, StreamOperatorStdString)
{
    ::testing::Test::RecordProperty("TEST_ID", "da8dde06-3f69-4549-b584-64c3ad328dbc");
    std::string logValue{"This is the iceoryx logger!"};
    const std::string constLogValue{"Nothing to see here, move along!"};
    LogStreamSut(loggerMock) << logValue;
    LogStreamSut(loggerMock) << constLogValue;

    ASSERT_THAT(loggerMock.logs.size(), Eq(2U));
    EXPECT_THAT(loggerMock.logs[0].message, StrEq(logValue));
    EXPECT_THAT(loggerMock.logs[1].message, StrEq(constLogValue));
}

TEST_F(IoxLogStream_test, StreamOperatorChar)
{
    ::testing::Test::RecordProperty("TEST_ID", "2a1fff17-e388-4f84-bb16-30bb3432ae9d");
    char logValue{'b'};
    const char constLogValue{'o'};
    constexpr char CONSTEXPR_LOG_VALUE{'b'};
    LogStreamSut(loggerMock) << logValue;
    LogStreamSut(loggerMock) << constLogValue;
    LogStreamSut(loggerMock) << CONSTEXPR_LOG_VALUE;

    ASSERT_THAT(loggerMock.logs.size(), Eq(3U));
    EXPECT_THAT(loggerMock.logs[0].message, StrEq("b"));
    EXPECT_THAT(loggerMock.logs[1].message, StrEq("o"));
    EXPECT_THAT(loggerMock.logs[2].message, StrEq("b"));
}

TEST_F(IoxLogStream_test, StreamOperator8BitTypesWithCharAsCharacterAndEverythingElseAsNumber)
{
    ::testing::Test::RecordProperty("TEST_ID", "707d4c04-1999-4713-b930-e113969617e0");
    char cc{'a'};
    signed char sc{'a'};
    unsigned char uc{'a'};
    int8_t i8{'a'};
    uint8_t u8{'a'};

    LogStreamSut(loggerMock) << cc;
    LogStreamSut(loggerMock) << sc;
    LogStreamSut(loggerMock) << uc;
    LogStreamSut(loggerMock) << i8;
    LogStreamSut(loggerMock) << u8;

    ASSERT_THAT(loggerMock.logs.size(), Eq(5U));
    EXPECT_THAT(loggerMock.logs[0].message, StrEq("a"));
    EXPECT_THAT(loggerMock.logs[1].message, StrEq("97"));
    EXPECT_THAT(loggerMock.logs[2].message, StrEq("97"));
    EXPECT_THAT(loggerMock.logs[3].message, StrEq("97"));
    EXPECT_THAT(loggerMock.logs[4].message, StrEq("97"));
}

TEST_F(IoxLogStream_test, StreamOperatorLogLevel)
{
    ::testing::Test::RecordProperty("TEST_ID", "d85b7ef4-35de-4e11-b0fd-f0de6581a9e6");
    std::string logValue{"This is the iceoryx logger!"};
    const auto logLevel = iox::log::LogLevel::Warn;
    LogStreamSut(loggerMock) << logValue << logLevel;

    EXPECT_THAT(loggerMock.logs[0].message, StrEq("This is the iceoryx logger!LogLevel::Warn"));
}

constexpr bool isBigEndian()
{
    constexpr uint16_t endianess{0x0100};
    return static_cast<const uint8_t&>(endianess) == 1;
}

TEST_F(IoxLogStream_test, StreamOperatorLogRawBufferWithObject)
{
    ::testing::Test::RecordProperty("TEST_ID", "24974c62-3ec6-4a02-83ff-cbb61a3de664");
    struct DummyStruct
    {
        uint16_t a{0xAFFE};
        uint16_t b{0xDEAD};
        uint32_t c{0xC0FFEE};
    };
    constexpr const char* EXPECTED_DATA{isBigEndian() ? "0x[af fe de ad 00 c0 ff ee]" : "0x[fe af ad de ee ff c0 00]"};

    DummyStruct d;

    LogStreamSut(loggerMock) << iox::log::raw(d);

    EXPECT_THAT(loggerMock.logs[0].message, StrEq(EXPECTED_DATA));
}

TEST_F(IoxLogStream_test, StreamOperatorLogRawBufferWithPointer)
{
    ::testing::Test::RecordProperty("TEST_ID", "54579f85-0d7a-4d51-b3a8-e18f256f2703");
    struct DummyStruct
    {
        uint16_t a{0xBEEF};
        uint16_t b{0xAFFE};
        uint32_t c{0xBAADF00D};
    };
    constexpr const char* EXPECTED_DATA{isBigEndian() ? "0x[be ef af fe ba ad f0 0d]" : "0x[ef be fe af 0d f0 ad ba]"};

    DummyStruct d;

    LogStreamSut(loggerMock) << iox::log::raw(&d, sizeof(d));

    EXPECT_THAT(loggerMock.logs[0].message, StrEq(EXPECTED_DATA));
}

TEST_F(IoxLogStream_test, StreamOperatorLogRawBufferWithNullpointer)
{
    ::testing::Test::RecordProperty("TEST_ID", "4b1306cb-68d8-4345-b5dd-46fadff02c8d");
    constexpr const char* EXPECTED_DATA{"0x[nullptr, 42]"};

    LogStreamSut(loggerMock) << iox::log::raw(nullptr, 42);

    EXPECT_THAT(loggerMock.logs[0].message, StrEq(EXPECTED_DATA));
}
} // namespace
