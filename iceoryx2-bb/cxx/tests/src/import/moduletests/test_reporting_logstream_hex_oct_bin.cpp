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

namespace
{
using namespace ::testing;

template <class T>
class IoxLogStreamHexOctBinIntegral_test : public IoxLogStreamBase_test
{
  public:
    T LogValueLow = std::numeric_limits<T>::lowest();
    T LogValueMin = std::numeric_limits<T>::min();
    T LogValueMax = std::numeric_limits<T>::max();
};

using LogHexOctBinIntegralTypes = Types<uint8_t, int8_t, uint16_t, int16_t, uint32_t, int32_t, uint64_t, int64_t>;

TYPED_TEST_SUITE(IoxLogStreamHexOctBinIntegral_test, LogHexOctBinIntegralTypes, );

template <typename LogType>
void testStreamOperatorLogHex(Logger_Mock& loggerMock, LogType logValue)
{
    LogStreamSut(loggerMock) << iox::log::hex(logValue);

    // we need to check negative numbers in two's complement, therefore make the output value unsigned
    using TestType = typename std::make_unsigned<LogType>::type;
    auto outputValue = static_cast<TestType>(logValue);

    std::stringstream ss;
    ss << "0x" << std::hex << +outputValue; // the '+' is to prevent to interpret the (u)int8_t as char

    ASSERT_THAT(loggerMock.logs.size(), Eq(1U));
    EXPECT_THAT(loggerMock.logs[0].message, StrEq(ss.str()));
}

TYPED_TEST(IoxLogStreamHexOctBinIntegral_test, StreamOperatorLogHex_ValueLow)
{
    ::testing::Test::RecordProperty("TEST_ID", "bd47c99a-0808-4a19-bafb-580c65009e0d");
    testStreamOperatorLogHex(this->loggerMock, this->LogValueLow);
}

TYPED_TEST(IoxLogStreamHexOctBinIntegral_test, StreamOperatorLogHex_ValueMin)
{
    ::testing::Test::RecordProperty("TEST_ID", "ee806b08-0e2a-49fd-b16b-5ad1c8da3150");
    testStreamOperatorLogHex(this->loggerMock, this->LogValueMin);
}

TYPED_TEST(IoxLogStreamHexOctBinIntegral_test, StreamOperatorLogHex_ValueMax)
{
    ::testing::Test::RecordProperty("TEST_ID", "acfa2bbf-c2e1-42bf-88c6-2888d7d3a42a");
    testStreamOperatorLogHex(this->loggerMock, this->LogValueMax);
}

template <typename LogType>
void testStreamOperatorLogOct(Logger_Mock& loggerMock, LogType logValue)
{
    LogStreamSut(loggerMock) << iox::log::oct(logValue);

    // we need to check negative numbers in two's complement, therefore make the output value unsigned
    using TestType = typename std::make_unsigned<LogType>::type;
    auto outputValue = static_cast<TestType>(logValue);

    std::stringstream ss;
    ss << "0o" << std::oct << +outputValue; // the '+' is to prevent to interpret the (u)int8_t as char

    ASSERT_THAT(loggerMock.logs.size(), Eq(1U));
    EXPECT_THAT(loggerMock.logs[0].message, StrEq(ss.str()));
}

TYPED_TEST(IoxLogStreamHexOctBinIntegral_test, StreamOperatorLogOct_ValueLow)
{
    ::testing::Test::RecordProperty("TEST_ID", "0163ef2b-7e62-4ded-b093-d77cc4dc360e");
    testStreamOperatorLogOct(this->loggerMock, this->LogValueLow);
}

TYPED_TEST(IoxLogStreamHexOctBinIntegral_test, StreamOperatorLogOct_ValueMin)
{
    ::testing::Test::RecordProperty("TEST_ID", "68583225-9b47-486d-90da-31f6c1ca7480");
    testStreamOperatorLogOct(this->loggerMock, this->LogValueMin);
}

TYPED_TEST(IoxLogStreamHexOctBinIntegral_test, StreamOperatorLogOct_ValueMax)
{
    ::testing::Test::RecordProperty("TEST_ID", "c4d104c3-a12a-46e1-8755-cc4b9e0e480c");
    testStreamOperatorLogOct(this->loggerMock, this->LogValueMax);
}

template <typename LogType>
void testStreamOperatorLogBin(Logger_Mock& loggerMock, LogType logValue)
{
    LogStreamSut(loggerMock) << iox::log::bin(logValue);

    // we need to check negative numbers in two's complement, therefore make the output value unsigned
    using TestType = typename std::make_unsigned<LogType>::type;
    auto outputValue = static_cast<TestType>(logValue);

    auto expectedValue = "0b" + std::bitset<std::numeric_limits<TestType>::digits>(outputValue).to_string();

    ASSERT_THAT(loggerMock.logs.size(), Eq(1U));
    EXPECT_THAT(loggerMock.logs[0].message, StrEq(expectedValue));
}

TYPED_TEST(IoxLogStreamHexOctBinIntegral_test, StreamOperatorLogBin_ValueLow)
{
    ::testing::Test::RecordProperty("TEST_ID", "e4e684b7-5bcf-4e8d-8cb1-b6df95c3b37c");
    testStreamOperatorLogBin(this->loggerMock, this->LogValueLow);
}

TYPED_TEST(IoxLogStreamHexOctBinIntegral_test, StreamOperatorLogBin_ValueMin)
{
    ::testing::Test::RecordProperty("TEST_ID", "f13b3e6a-8f7c-48c2-ae43-35e0a195556e");
    testStreamOperatorLogBin(this->loggerMock, this->LogValueMin);
}

TYPED_TEST(IoxLogStreamHexOctBinIntegral_test, StreamOperatorLogBin_ValueMax)
{
    ::testing::Test::RecordProperty("TEST_ID", "b583014a-700f-46e3-8b7c-0a128c59598a");
    testStreamOperatorLogBin(this->loggerMock, this->LogValueMax);
}

template <class T>
class IoxLogStreamHexFloatingPoint_test : public IoxLogStreamBase_test
{
  public:
    T LogValueLow = std::numeric_limits<T>::lowest();
    T LogValueMin = std::numeric_limits<T>::min();
    T LogValueMax = std::numeric_limits<T>::max();
};

using LogHexFloatingPointTypes = Types<float, double, long double>;

TYPED_TEST_SUITE(IoxLogStreamHexFloatingPoint_test, LogHexFloatingPointTypes, );

template <typename T>
constexpr const char* floatingPointFormatSpecifier();
template <>
constexpr const char* floatingPointFormatSpecifier<float>()
{
    return "%a";
}
template <>
constexpr const char* floatingPointFormatSpecifier<double>()
{
    return "%la";
}
template <>
constexpr const char* floatingPointFormatSpecifier<long double>()
{
    return "%La";
}

template <typename LogType>
void testStreamOperatorLogHexFloatingPoint(Logger_Mock& loggerMock, LogType logValue)
{
    LogStreamSut(loggerMock) << iox::log::hex(logValue);

    constexpr uint64_t BUFFER_SIZE{1000};
    std::array<char, BUFFER_SIZE> buffer{0};
    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-vararg,hicpp-vararg) required to create a hex formatted float
    EXPECT_THAT(snprintf(buffer.data(), BUFFER_SIZE - 1, floatingPointFormatSpecifier<LogType>(), logValue), Gt(0));

    ASSERT_THAT(loggerMock.logs.size(), Eq(1U));
    EXPECT_THAT(loggerMock.logs[0].message, StrEq(buffer.data()));
}

TYPED_TEST(IoxLogStreamHexFloatingPoint_test, StreamOperatorLogHex_ValueLow)
{
    ::testing::Test::RecordProperty("TEST_ID", "abcad269-7e1b-478a-9535-c9fff9ba1d05");
    testStreamOperatorLogHexFloatingPoint(this->loggerMock, this->LogValueLow);
}

TYPED_TEST(IoxLogStreamHexFloatingPoint_test, StreamOperatorLogHex_ValueMin)
{
    ::testing::Test::RecordProperty("TEST_ID", "78d3a9c5-3f3e-410a-baba-942c5a1db46e");
    testStreamOperatorLogHexFloatingPoint(this->loggerMock, this->LogValueMin);
}

TYPED_TEST(IoxLogStreamHexFloatingPoint_test, StreamOperatorLogHex_ValueMax)
{
    ::testing::Test::RecordProperty("TEST_ID", "e87982ae-58e7-4563-99bc-d08484fb60c9");
    testStreamOperatorLogHexFloatingPoint(this->loggerMock, this->LogValueMax);
}

} // namespace
