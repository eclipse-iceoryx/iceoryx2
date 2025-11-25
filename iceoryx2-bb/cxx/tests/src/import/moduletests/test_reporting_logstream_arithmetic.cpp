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

template <typename T>
class Wrapper
{
  public:
    constexpr explicit Wrapper(T value) noexcept
        : m_value(value)
    {
    }

    // NOLINTJUSTIFICATION Implicit conversion is required for the test
    // NOLINTNEXTLINE(hicpp-explicit-conversions)
    constexpr operator T() const noexcept
    {
        return m_value;
    }

  private:
    T m_value{};
};

using ArithmeticTypes =
    Types<bool, int8_t, int16_t, int32_t, int64_t, uint8_t, uint16_t, uint32_t, uint64_t, size_t, float, double>;

TYPED_TEST_SUITE(IoxLogStreamArithmetic_test, ArithmeticTypes, );

template <typename Arithmetic>
class IoxLogStreamArithmetic_test : public IoxLogStreamBase_test
{
  public:
    using type_t = Arithmetic;

    Arithmetic LogValueLow = std::numeric_limits<Arithmetic>::lowest();
    Arithmetic LogValueMin = std::numeric_limits<Arithmetic>::min();
    Arithmetic LogValueMax = std::numeric_limits<Arithmetic>::max();

    const Arithmetic ConstLogValueLow = std::numeric_limits<Arithmetic>::lowest();
    const Arithmetic ConstLogValueMin = std::numeric_limits<Arithmetic>::min();
    const Arithmetic ConstLogValueMax = std::numeric_limits<Arithmetic>::max();

    static constexpr Arithmetic CONSTEXPR_LOG_VALUE_LOW = std::numeric_limits<Arithmetic>::lowest();
    static constexpr Arithmetic CONSTEXPR_LOG_VALUE_MIN = std::numeric_limits<Arithmetic>::min();
    static constexpr Arithmetic CONSTEXPR_LOG_VALUE_MAX = std::numeric_limits<Arithmetic>::max();
};

template <typename Arithmetic>
constexpr Arithmetic IoxLogStreamArithmetic_test<Arithmetic>::CONSTEXPR_LOG_VALUE_LOW;

template <typename Arithmetic>
constexpr Arithmetic IoxLogStreamArithmetic_test<Arithmetic>::CONSTEXPR_LOG_VALUE_MIN;

template <typename Arithmetic>
constexpr Arithmetic IoxLogStreamArithmetic_test<Arithmetic>::CONSTEXPR_LOG_VALUE_MAX;

template <typename T>
std::string convertToString(const T val)
{
    return std::to_string(val);
}

template <>
std::string convertToString<float>(const float val)
{
    std::stringstream ss;
    ss << val;
    return ss.str();
}

template <>
std::string convertToString<double>(const double val)
{
    std::stringstream ss;
    ss << val;
    return ss.str();
}

template <>
std::string convertToString<bool>(const bool val)
{
    return std::string(val ? "true" : "false");
}

TYPED_TEST(IoxLogStreamArithmetic_test, StreamOperator_ValueLow)
{
    ::testing::Test::RecordProperty("TEST_ID", "31f1504a-9353-4c46-9c8b-d7e430b07bd6");
    LogStreamSut(this->loggerMock) << this->LogValueLow;

    ASSERT_THAT(this->loggerMock.logs.size(), Eq(1U));
    EXPECT_THAT(this->loggerMock.logs[0].message, StrEq(convertToString(this->LogValueLow)));
}

TYPED_TEST(IoxLogStreamArithmetic_test, StreamOperator_ValueMin)
{
    ::testing::Test::RecordProperty("TEST_ID", "e784ceb9-1e23-4e95-b667-855835897717");
    LogStreamSut(this->loggerMock) << this->LogValueMin;

    ASSERT_THAT(this->loggerMock.logs.size(), Eq(1U));
    EXPECT_THAT(this->loggerMock.logs[0].message, StrEq(convertToString(this->LogValueMin)));
}

TYPED_TEST(IoxLogStreamArithmetic_test, StreamOperator_ValueMax)
{
    ::testing::Test::RecordProperty("TEST_ID", "3bff0182-07ad-4c7a-b8b7-3950a8aa9f4e");
    LogStreamSut(this->loggerMock) << this->LogValueMax;

    ASSERT_THAT(this->loggerMock.logs.size(), Eq(1U));
    EXPECT_THAT(this->loggerMock.logs[0].message, StrEq(convertToString(this->LogValueMax)));
}

TYPED_TEST(IoxLogStreamArithmetic_test, StreamOperator_WrappedAndImplicitlyConverted_ValueLow)
{
    ::testing::Test::RecordProperty("TEST_ID", "86dccc28-0007-4ab3-88c6-216eeb861fef");
    using type_t = typename TestFixture::type_t;
    LogStreamSut(this->loggerMock) << Wrapper<type_t>(this->LogValueLow);

    ASSERT_THAT(this->loggerMock.logs.size(), Eq(1U));
    EXPECT_THAT(this->loggerMock.logs[0].message, StrEq(convertToString(this->LogValueLow)));
}

TYPED_TEST(IoxLogStreamArithmetic_test, StreamOperator_WrappedAndImplicitlyConverted_ValueMin)
{
    ::testing::Test::RecordProperty("TEST_ID", "72bc51fe-a23f-4c7d-a17f-0e4ad29bee9f");
    using type_t = typename TestFixture::type_t;
    LogStreamSut(this->loggerMock) << Wrapper<type_t>(this->LogValueMin);

    ASSERT_THAT(this->loggerMock.logs.size(), Eq(1U));
    EXPECT_THAT(this->loggerMock.logs[0].message, StrEq(convertToString(this->LogValueMin)));
}

TYPED_TEST(IoxLogStreamArithmetic_test, StreamOperator_WrappedAndImplicitlyConverted_ValueMax)
{
    ::testing::Test::RecordProperty("TEST_ID", "49e79c8b-98c0-4074-aed1-25293bf49bd0");
    using type_t = typename TestFixture::type_t;
    LogStreamSut(this->loggerMock) << Wrapper<type_t>(this->LogValueMax);

    ASSERT_THAT(this->loggerMock.logs.size(), Eq(1U));
    EXPECT_THAT(this->loggerMock.logs[0].message, StrEq(convertToString(this->LogValueMax)));
}

TYPED_TEST(IoxLogStreamArithmetic_test, StreamOperator_ConstValueLow)
{
    ::testing::Test::RecordProperty("TEST_ID", "65cfbc9b-a535-47fa-a543-0c31ba63d4ba");
    LogStreamSut(this->loggerMock) << this->ConstLogValueLow;

    ASSERT_THAT(this->loggerMock.logs.size(), Eq(1U));
    EXPECT_THAT(this->loggerMock.logs[0].message, StrEq(convertToString(this->ConstLogValueLow)));
}

TYPED_TEST(IoxLogStreamArithmetic_test, StreamOperator_ConstValueMin)
{
    ::testing::Test::RecordProperty("TEST_ID", "fba70497-e252-4458-b00e-2dad8b94b8c8");
    LogStreamSut(this->loggerMock) << this->ConstLogValueMin;

    ASSERT_THAT(this->loggerMock.logs.size(), Eq(1U));
    EXPECT_THAT(this->loggerMock.logs[0].message, StrEq(convertToString(this->ConstLogValueMin)));
}

TYPED_TEST(IoxLogStreamArithmetic_test, StreamOperator_ConstValueMax)
{
    ::testing::Test::RecordProperty("TEST_ID", "e5e28a6e-4321-4030-b53e-90089b3ee9b9");
    LogStreamSut(this->loggerMock) << this->ConstLogValueMax;

    ASSERT_THAT(this->loggerMock.logs.size(), Eq(1));
    EXPECT_THAT(this->loggerMock.logs[0].message, StrEq(convertToString(this->ConstLogValueMax)));
}

TYPED_TEST(IoxLogStreamArithmetic_test, StreamOperator_WrappedAndImplicitlyConverted_ConstValueLow)
{
    ::testing::Test::RecordProperty("TEST_ID", "77cb68ac-bffb-4cf5-a4c2-119e5aaae307");
    using type_t = typename TestFixture::type_t;
    LogStreamSut(this->loggerMock) << Wrapper<type_t>(this->ConstLogValueLow);

    ASSERT_THAT(this->loggerMock.logs.size(), Eq(1U));
    EXPECT_THAT(this->loggerMock.logs[0].message, StrEq(convertToString(this->ConstLogValueLow)));
}

TYPED_TEST(IoxLogStreamArithmetic_test, StreamOperator_WrappedAndImplicitlyConverted_ConstValueMin)
{
    ::testing::Test::RecordProperty("TEST_ID", "3b042ec2-0a6d-4d24-9a12-dab81736847f");
    using type_t = typename TestFixture::type_t;
    LogStreamSut(this->loggerMock) << Wrapper<type_t>(this->ConstLogValueMin);

    ASSERT_THAT(this->loggerMock.logs.size(), Eq(1U));
    EXPECT_THAT(this->loggerMock.logs[0].message, StrEq(convertToString(this->ConstLogValueMin)));
}

TYPED_TEST(IoxLogStreamArithmetic_test, StreamOperator_WrappedAndImplicitlyConverted_ConstValueMax)
{
    ::testing::Test::RecordProperty("TEST_ID", "83194cc1-3c2a-4c0e-a413-972a95f36b66");
    using type_t = typename TestFixture::type_t;
    LogStreamSut(this->loggerMock) << Wrapper<type_t>(this->ConstLogValueMax);

    ASSERT_THAT(this->loggerMock.logs.size(), Eq(1));
    EXPECT_THAT(this->loggerMock.logs[0].message, StrEq(convertToString(this->ConstLogValueMax)));
}

TYPED_TEST(IoxLogStreamArithmetic_test, StreamOperator_ConstexprValueLow)
{
    ::testing::Test::RecordProperty("TEST_ID", "e9688979-d209-4718-9810-49684fdd9261");
    LogStreamSut(this->loggerMock) << this->CONSTEXPR_LOG_VALUE_LOW;

    ASSERT_THAT(this->loggerMock.logs.size(), Eq(1U));
    EXPECT_THAT(this->loggerMock.logs[0].message, StrEq(convertToString(this->CONSTEXPR_LOG_VALUE_LOW)));
}

TYPED_TEST(IoxLogStreamArithmetic_test, StreamOperator_ConstexprValueMin)
{
    ::testing::Test::RecordProperty("TEST_ID", "f6799599-582a-454c-85b8-b2059a5d50c6");
    LogStreamSut(this->loggerMock) << this->CONSTEXPR_LOG_VALUE_MIN;

    ASSERT_THAT(this->loggerMock.logs.size(), Eq(1U));
    EXPECT_THAT(this->loggerMock.logs[0].message, StrEq(convertToString(this->CONSTEXPR_LOG_VALUE_MIN)));
}

TYPED_TEST(IoxLogStreamArithmetic_test, StreamOperator_ConstexprValueMax)
{
    ::testing::Test::RecordProperty("TEST_ID", "4a6dc777-a53b-4a42-9ab1-e1da893ad884");
    LogStreamSut(this->loggerMock) << this->CONSTEXPR_LOG_VALUE_MAX;

    ASSERT_THAT(this->loggerMock.logs.size(), Eq(1U));
    EXPECT_THAT(this->loggerMock.logs[0].message, StrEq(convertToString(this->CONSTEXPR_LOG_VALUE_MAX)));
}

TYPED_TEST(IoxLogStreamArithmetic_test, StreamOperator_WrappedAndImplicitlyConverted_ConstexprValueLow)
{
    ::testing::Test::RecordProperty("TEST_ID", "3025b6e3-3abd-49e8-8d3c-f6fa0c164011");
    using type_t = typename TestFixture::type_t;
    LogStreamSut(this->loggerMock) << Wrapper<type_t>(this->CONSTEXPR_LOG_VALUE_LOW);

    ASSERT_THAT(this->loggerMock.logs.size(), Eq(1U));
    EXPECT_THAT(this->loggerMock.logs[0].message, StrEq(convertToString(this->CONSTEXPR_LOG_VALUE_LOW)));
}

TYPED_TEST(IoxLogStreamArithmetic_test, StreamOperator_WrappedAndImplicitlyConverted_ConstexprValueMin)
{
    ::testing::Test::RecordProperty("TEST_ID", "c49ad58d-948a-499a-9e2b-8bbdeb9b5c5e");
    using type_t = typename TestFixture::type_t;
    LogStreamSut(this->loggerMock) << Wrapper<type_t>(this->CONSTEXPR_LOG_VALUE_MIN);

    ASSERT_THAT(this->loggerMock.logs.size(), Eq(1U));
    EXPECT_THAT(this->loggerMock.logs[0].message, StrEq(convertToString(this->CONSTEXPR_LOG_VALUE_MIN)));
}

TYPED_TEST(IoxLogStreamArithmetic_test, StreamOperator_WrappedAndImplicitlyConverted_ConstexprValueMax)
{
    ::testing::Test::RecordProperty("TEST_ID", "8c7637ce-8155-4edc-88e2-a155169934f2");
    using type_t = typename TestFixture::type_t;
    LogStreamSut(this->loggerMock) << Wrapper<type_t>(this->CONSTEXPR_LOG_VALUE_MAX);

    ASSERT_THAT(this->loggerMock.logs.size(), Eq(1U));
    EXPECT_THAT(this->loggerMock.logs[0].message, StrEq(convertToString(this->CONSTEXPR_LOG_VALUE_MAX)));
}

} // namespace
