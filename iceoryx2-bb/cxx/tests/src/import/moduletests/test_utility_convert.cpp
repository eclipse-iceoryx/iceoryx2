// Copyright (c) 2019 by Robert Bosch GmbH. All rights reserved.
// Copyright (c) 2021 - 2022 by Apex.AI Inc. All rights reserved.
// Copyright (c) 2022 by NXP. All rights reserved.
// Copyright (c) 2023 by Dennis Liu. All rights reserved.
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

#include "iox/detail/convert.hpp"
#include "iox/std_string_support.hpp"
#include "test.hpp"

#include <cmath>
#include <cstdint>
#include <iomanip>
#include <tuple>
namespace
{
using namespace ::testing;

using NumberType = iox::convert::NumberType;

class LongDouble
{
  public:
    static bool Eq(long double a, long double b)
    {
        IOX_LOG(Debug, "a: " << a << ", b: " << b);

        long double min_val = std::min(std::fabs(a), std::fabs(b));
        long double epsilon = std::fabs(min_val - std::nextafter(min_val, static_cast<long double>(0)));

        IOX_LOG(Debug, "epsilon from min_val: " << epsilon);
        IOX_LOG(Debug, "abs min_val: " << min_val);

        if (epsilon <= 0 || epsilon < std::numeric_limits<long double>::min())
        {
            epsilon = std::numeric_limits<long double>::min();
        }
        IOX_LOG(Debug, "epsilon: " << epsilon);

        long double abs_diff = std::fabs(a - b);
        IOX_LOG(Debug, "fabs result: " << abs_diff);

        bool is_equal = abs_diff <= epsilon;
        IOX_LOG(Debug, "<< a and b " << ((is_equal) ? "IS" : "IS NOT") << " considered equal! >>");

        return is_equal;
    }
};

class convert_test : public Test
{
  public:
    void SetUp() override
    {
    }
    void TearDown() override
    {
    }
    template <typename T>
    std::string fp_to_string(T value, uint16_t digits = std::numeric_limits<T>::digits10)
    {
        static_assert(std::is_floating_point<T>::value, "requires floating point type");

        std::ostringstream oss;
        oss << std::scientific << std::setprecision(digits) << value;
        return oss.str();
    }
};


TEST_F(convert_test, toString_uint8_t)
{
    ::testing::Test::RecordProperty("TEST_ID", "24321fe1-52e2-48e1-b31a-436473b3e5f0");
    constexpr uint8_t DATA = 131U;
    EXPECT_THAT(iox::convert::toString(DATA), Eq("131"));
}

TEST_F(convert_test, toString_int8_t)
{
    ::testing::Test::RecordProperty("TEST_ID", "3ec95300-04e9-4282-a7a4-92a7d8717343");
    constexpr int8_t DATA = 31U;
    EXPECT_THAT(iox::convert::toString(DATA), Eq("31"));
}

TEST_F(convert_test, toString_Integer)
{
    ::testing::Test::RecordProperty("TEST_ID", "c426bfd9-3cfa-4986-90ed-d55147434a3e");
    constexpr int DATA = 33331;
    EXPECT_THAT(iox::convert::toString(DATA), Eq("33331"));
}

TEST_F(convert_test, toString_Float)
{
    ::testing::Test::RecordProperty("TEST_ID", "e00f7b9c-325c-4eb1-885c-83f8d5fa3f72");
    constexpr float DATA = 333.1F;
    EXPECT_THAT(iox::convert::toString(DATA), Eq("333.1"));
}

TEST_F(convert_test, toString_LongLongUnsignedInt)
{
    ::testing::Test::RecordProperty("TEST_ID", "5d70c7e8-801e-4492-9f01-036c62b4ce54");
    constexpr long long unsigned DATA = 123LLU;
    EXPECT_THAT(iox::convert::toString(DATA), Eq("123"));
}

TEST_F(convert_test, toString_Char)
{
    ::testing::Test::RecordProperty("TEST_ID", "fb223438-73e9-409a-b644-088bb6509d9c");
    constexpr char DATA = 'x';
    EXPECT_THAT(iox::convert::toString(DATA), Eq("x"));
}

TEST_F(convert_test, toString_String)
{
    ::testing::Test::RecordProperty("TEST_ID", "43eb7090-619c-42a5-bad9-9f452e81228b");
    const std::string DATA = "hello";
    EXPECT_THAT(iox::convert::toString(DATA), Eq("hello"));
}

TEST_F(convert_test, toString_StringConvertableClass)
{
    ::testing::Test::RecordProperty("TEST_ID", "39601439-ec94-49d0-ac30-168dd0598bdc");
    struct A
    {
        // we want to test the implicit conversion
        // NOLINTNEXTLINE(hicpp-explicit-conversions)
        operator std::string() const
        {
            return "fuu";
        }
    };

    EXPECT_THAT(iox::convert::toString(A()), Eq("fuu"));
}

TEST_F(convert_test, FromString_String)
{
    ::testing::Test::RecordProperty("TEST_ID", "22463da5-0fcb-4aa2-a7e5-68b863278a81");
    std::string source = "hello";
    auto result = iox::convert::from_string<std::string>(source.c_str());
    ASSERT_THAT(result.has_value(), Eq(true));
    EXPECT_THAT(result.value(), Eq(source));
}

TEST_F(convert_test, fromString_Char_Success)
{
    ::testing::Test::RecordProperty("TEST_ID", "a15825c9-536a-4671-a502-6973490022e7");
    std::string source = "h";
    auto result = iox::convert::from_string<char>(source.c_str());
    ASSERT_THAT(result.has_value(), Eq(true));
    EXPECT_THAT(result.value(), Eq(source[0]));
}

TEST_F(convert_test, fromString_Char_Fail)
{
    ::testing::Test::RecordProperty("TEST_ID", "656e87ad-6fdb-42d7-bf49-23f81a4f5a31");
    std::string source = "hasd";
    auto result = iox::convert::from_string<char>(source.c_str());
    ASSERT_THAT(result.has_value(), Eq(false));
}

TEST_F(convert_test, fromString_FLOAT_Success)
{
    ::testing::Test::RecordProperty("TEST_ID", "d6255c3e-369e-43a0-a1ab-03f7b13d03c2");
    std::string source = "123.01";
    auto result = iox::convert::from_string<float>(source.c_str());
    ASSERT_THAT(result.has_value(), Eq(true));
    EXPECT_FLOAT_EQ(result.value(), 123.01F);
}

TEST_F(convert_test, fromString_FLOAT_Fail)
{
    ::testing::Test::RecordProperty("TEST_ID", "e2b94d50-664c-4f9e-be4f-99212c6fa165");
    std::string source = "hasd";
    auto result = iox::convert::from_string<float>(source.c_str());
    ASSERT_THAT(result.has_value(), Eq(false));
}

TEST_F(convert_test, fromString_Double_Success)
{
    ::testing::Test::RecordProperty("TEST_ID", "95ba379e-120e-4b80-a829-33fe54f1bfed");
    std::string source = "123.04";
    auto result = iox::convert::from_string<double>(source.c_str());
    ASSERT_THAT(result.has_value(), Eq(true));
    EXPECT_THAT(result.value(), DoubleEq(123.04));
}

TEST_F(convert_test, fromString_Double_Fail)
{
    ::testing::Test::RecordProperty("TEST_ID", "f4ace11b-a056-47b1-b6c5-6fb2c58e1a06");
    std::string source = "hasd";
    auto result = iox::convert::from_string<double>(source.c_str());
    ASSERT_THAT(result.has_value(), Eq(false));
}

TEST_F(convert_test, fromString_LongDouble_Success)
{
    ::testing::Test::RecordProperty("TEST_ID", "2864fbae-ef1c-48ab-97f2-745baadc4dc5");
    constexpr long double VERIFY = 121.01L;
    std::string source = "121.01";

    auto result = iox::convert::from_string<long double>(source.c_str());
    ASSERT_THAT(result.has_value(), Eq(true));
    // NOLINTNEXTLINE(clang-analyzer-core.CallAndMessage)
    EXPECT_THAT(LongDouble::Eq(VERIFY, result.value()), Eq(true));
}

TEST_F(convert_test, fromString_LongDouble_Fail)
{
    ::testing::Test::RecordProperty("TEST_ID", "519f2ac5-8836-419e-8034-377230a88a09");
    std::string source = "hasd";
    auto result = iox::convert::from_string<long double>(source.c_str());
    ASSERT_THAT(result.has_value(), Eq(false));
}

TEST_F(convert_test, fromString_UNSIGNED_Int_Success)
{
    ::testing::Test::RecordProperty("TEST_ID", "1edb8d5f-c42d-4d02-bc31-477f48898bbb");
    std::string source = "100";
    auto result = iox::convert::from_string<unsigned int>(source.c_str());
    ASSERT_THAT(result.has_value(), Eq(true));
    EXPECT_THAT(result.value(), Eq(100U));
}

TEST_F(convert_test, fromString_UNSIGNED_Int_Fail)
{
    ::testing::Test::RecordProperty("TEST_ID", "6ce6de82-a6c0-4562-9c5c-663b93d768b3");
    std::string source = "-331";
    auto result = iox::convert::from_string<unsigned int>(source.c_str());
    ASSERT_THAT(result.has_value(), Eq(false));
}

TEST_F(convert_test, fromString_UNSIGNED_LongInt_Success)
{
    ::testing::Test::RecordProperty("TEST_ID", "054b08b2-54e1-4191-91b6-e6bec415612f");
    std::string source = "999";
    auto result = iox::convert::from_string<uint64_t>(source.c_str());
    ASSERT_THAT(result.has_value(), Eq(true));
    EXPECT_THAT(result.value(), Eq(999LU));
}

TEST_F(convert_test, fromString_UNSIGNED_LongInt_Fail)
{
    ::testing::Test::RecordProperty("TEST_ID", "4b215747-90b2-4ca2-97ee-517c07597b1b");
    std::string source = "-a123";
    auto result = iox::convert::from_string<uint64_t>(source.c_str());
    ASSERT_THAT(result.has_value(), Eq(false));
}

TEST_F(convert_test, fromString_Int_Success)
{
    ::testing::Test::RecordProperty("TEST_ID", "9318ee60-f2e0-445a-b32d-c718cf918b18");
    std::string source = "3331";
    auto result = iox::convert::from_string<int>(source.c_str());
    ASSERT_THAT(result.has_value(), Eq(true));
    EXPECT_THAT(result.value(), Eq(3331));
}

TEST_F(convert_test, fromString_Int_Fail)
{
    ::testing::Test::RecordProperty("TEST_ID", "f8e698a9-054d-4441-b196-bcd58a72b1d9");
    std::string source = "-+321";
    auto result = iox::convert::from_string<int>(source.c_str());
    ASSERT_THAT(result.has_value(), Eq(false));
}

TEST_F(convert_test, fromString_ShortInt_Success)
{
    ::testing::Test::RecordProperty("TEST_ID", "e804f821-157d-4c52-81a7-75fce5a43805");
    std::string source = "12345";
    auto result = iox::convert::from_string<short>(source.c_str());
    ASSERT_THAT(result.has_value(), Eq(true));
    EXPECT_THAT(result.value(), Eq(12345));
}

TEST_F(convert_test, fromString_ShortInt_Fail)
{
    ::testing::Test::RecordProperty("TEST_ID", "1150066b-cb42-4055-9927-2f20fb40bc87");
    std::string source = "-+123321";
    auto result = iox::convert::from_string<short>(source.c_str());
    ASSERT_THAT(result.has_value(), Eq(false));
}

TEST_F(convert_test, fromString_Bool_Success)
{
    ::testing::Test::RecordProperty("TEST_ID", "893723fc-dfb8-46a4-b446-badaf8bad25a");
    std::string source = "1";
    auto result = iox::convert::from_string<bool>(source.c_str());
    ASSERT_THAT(result.has_value(), Eq(true));
    EXPECT_THAT(result.value(), Eq(true));
}

TEST_F(convert_test, fromString_Bool_Fail)
{
    ::testing::Test::RecordProperty("TEST_ID", "1c937da6-29ea-49cf-a7d0-4c46f564c16e");
    std::string source = "-+222";
    auto result = iox::convert::from_string<bool>(source.c_str());
    ASSERT_THAT(result.has_value(), Eq(false));
}

TEST_F(convert_test, fromString_UShortInt_Success)
{
    ::testing::Test::RecordProperty("TEST_ID", "99d22d80-3860-47fa-9f98-f11ff9629815");
    std::string source = "333";
    auto result = iox::convert::from_string<unsigned short>(source.c_str());
    ASSERT_THAT(result.has_value(), Eq(true));
    EXPECT_THAT(result.value(), Eq(333));
}

TEST_F(convert_test, fromString_UShortInt_Fail)
{
    ::testing::Test::RecordProperty("TEST_ID", "6ab6ded6-dff3-401a-8a7f-98326da7cca6");
    std::string source = "-+111";
    auto result = iox::convert::from_string<unsigned short>(source.c_str());
    ASSERT_THAT(result.has_value(), Eq(false));
}

TEST_F(convert_test, fromString_LongInt_Success)
{
    ::testing::Test::RecordProperty("TEST_ID", "37133256-ae79-45c7-8c86-56bd33fa7bd8");
    std::string source = "-1123";
    auto result = iox::convert::from_string<int64_t>(source.c_str());
    ASSERT_THAT(result.has_value(), Eq(true));
    EXPECT_THAT(result.value(), Eq(-1123L));
}

TEST_F(convert_test, fromString_LongInt_Fail)
{
    ::testing::Test::RecordProperty("TEST_ID", "0e368bf3-cb16-4829-a4cc-dc56e0bde958");
    std::string source = "-a121";
    auto result = iox::convert::from_string<int64_t>(source.c_str());
    ASSERT_THAT(result.has_value(), Eq(false));
}

template <typename T>
void expect_failure(std::vector<std::string>& invalid_input)
{
    for (const auto& v : invalid_input)
    {
        auto invalid_ret = iox::convert::from_string<T>(v.c_str());
        ASSERT_THAT(invalid_ret.has_value(), Eq(false));
    }
}

TEST_F(convert_test, fromString_Integer_InvalidTrailingChar_Fail)
{
    ::testing::Test::RecordProperty("TEST_ID", "6a70f10f-227b-4b0a-8149-e5ca3c793b5d");

    std::vector<std::string> invalid_input = {"42a", "74 ", "-52-"};

    expect_failure<signed char>(invalid_input);
    expect_failure<short>(invalid_input);
    expect_failure<int>(invalid_input);
    expect_failure<long>(invalid_input);
    expect_failure<long long>(invalid_input);
    expect_failure<unsigned char>(invalid_input);
    expect_failure<unsigned short>(invalid_input);
    expect_failure<unsigned int>(invalid_input);
    expect_failure<unsigned long>(invalid_input);
    expect_failure<unsigned long long>(invalid_input);
}

/// SINGED INTEGRAL EDGE CASES START
/// inc: increment, dec: decrement

TEST_F(convert_test, fromString_SignedChar_EdgeCase_InRange_Success)
{
    ::testing::Test::RecordProperty("TEST_ID", "3a70481e-ae86-4b96-92c4-44169700e93a");

    std::string source = "-128";
    auto signed_char_min = iox::convert::from_string<signed char>(source.c_str());
    ASSERT_THAT(signed_char_min.has_value(), Eq(true));
    EXPECT_THAT(signed_char_min.value(), Eq(std::numeric_limits<signed char>::min()));

    source = "127";
    auto signed_char_max = iox::convert::from_string<signed char>(source.c_str());
    ASSERT_THAT(signed_char_max.has_value(), Eq(true));
    EXPECT_THAT(signed_char_max.value(), Eq(std::numeric_limits<signed char>::max()));
}

TEST_F(convert_test, fromString_SignedChar_EdgeCase_OutOfRange_Fail)
{
    ::testing::Test::RecordProperty("TEST_ID", "23a3aa3d-954f-43f3-96e9-6b885bbb0c86");

    std::string source = "-129";
    auto signed_char_min_dec_1 = iox::convert::from_string<signed char>(source.c_str());
    ASSERT_THAT(signed_char_min_dec_1.has_value(), Eq(false));

    source = "128";
    auto signed_char_max_inc_1 = iox::convert::from_string<signed char>(source.c_str());
    ASSERT_THAT(signed_char_max_inc_1.has_value(), Eq(false));
}

TEST_F(convert_test, fromString_SignedShort_EdgeCase_InRange_Success)
{
    ::testing::Test::RecordProperty("TEST_ID", "68f802ce-feb5-46d9-a956-dd3ca1a3ce53");

    std::string source = "-32768";
    auto short_min = iox::convert::from_string<short>(source.c_str());
    ASSERT_THAT(short_min.has_value(), Eq(true));
    EXPECT_THAT(short_min.value(), Eq(std::numeric_limits<short>::min()));

    source = "32767";
    auto short_max = iox::convert::from_string<short>(source.c_str());
    ASSERT_THAT(short_max.has_value(), Eq(true));
    EXPECT_THAT(short_max.value(), Eq(std::numeric_limits<short>::max()));
}

TEST_F(convert_test, fromString_SignedShort_EdgeCase_OutOfRange_Fail)
{
    ::testing::Test::RecordProperty("TEST_ID", "98e33efd-ba39-4b88-8307-358be30e4e73");

    std::string source = "-32769";
    auto short_min_dec_1 = iox::convert::from_string<short>(source.c_str());
    ASSERT_THAT(short_min_dec_1.has_value(), Eq(false));

    source = "32768";
    auto short_max_inc_1 = iox::convert::from_string<short>(source.c_str());
    ASSERT_THAT(short_max_inc_1.has_value(), Eq(false));
}

TEST_F(convert_test, fromString_SignedInt_EdgeCase_InRange_Success)
{
    ::testing::Test::RecordProperty("TEST_ID", "7333d0f9-dd48-4a88-85c9-2167d65633db");

    std::string source = "-2147483648";
    auto int_min = iox::convert::from_string<int>(source.c_str());
    ASSERT_THAT(int_min.has_value(), Eq(true));
    EXPECT_THAT(int_min.value(), Eq(std::numeric_limits<int>::min()));

    source = "2147483647";
    auto int_max = iox::convert::from_string<int>(source.c_str());
    ASSERT_THAT(int_max.has_value(), Eq(true));
    EXPECT_THAT(int_max.value(), Eq(std::numeric_limits<int>::max()));
}

TEST_F(convert_test, fromString_SignedInt_EdgeCase_OutOfRange_Fail)
{
    ::testing::Test::RecordProperty("TEST_ID", "abf0fda5-044e-4f1b-bb1e-31b701578a3d");

    std::string source = "-2147483649";
    auto int_min_dec_1 = iox::convert::from_string<int>(source.c_str());
    ASSERT_THAT(int_min_dec_1.has_value(), Eq(false));

    source = "2147483648";
    auto int_max_inc_1 = iox::convert::from_string<int>(source.c_str());
    ASSERT_THAT(int_max_inc_1.has_value(), Eq(false));
}

// platform dependent (32/64 bit system only)
TEST_F(convert_test, fromString_SignedLong_EdgeCase_InRange_Success)
{
    ::testing::Test::RecordProperty("TEST_ID", "5dc4c773-6a51-42b6-ad94-7ec885263856");

    std::string source = std::to_string(std::numeric_limits<long>::min());
    auto long_min = iox::convert::from_string<long>(source.c_str());
    ASSERT_THAT(long_min.has_value(), Eq(true));
    EXPECT_THAT(long_min.value(), Eq(std::numeric_limits<long>::min()));

    source = std::to_string(std::numeric_limits<long>::max());
    auto long_max = iox::convert::from_string<long>(source.c_str());
    ASSERT_THAT(long_max.has_value(), Eq(true));
    EXPECT_THAT(long_max.value(), Eq(std::numeric_limits<long>::max()));
}

TEST_F(convert_test, fromString_SignedLong_EdgeCase_OutOfRange_Fail)
{
    ::testing::Test::RecordProperty("TEST_ID", "fb349f9c-0800-4f0b-8940-9012f9946b77");

    constexpr bool IS_32_BIT{sizeof(long) != sizeof(long long)};

    std::string source = IS_32_BIT ? "-2147483649" : "-9223372036854775809";
    auto long_min_dec_1 = iox::convert::from_string<long>(source.c_str());
    ASSERT_THAT(long_min_dec_1.has_value(), Eq(false));

    source = IS_32_BIT ? "2147483648" : "9223372036854775808";
    auto long_max_inc_1 = iox::convert::from_string<long>(source.c_str());
    ASSERT_THAT(long_max_inc_1.has_value(), Eq(false));
}

TEST_F(convert_test, fromString_SignedLongLong_EdgeCase_InRange_Success)
{
    ::testing::Test::RecordProperty("TEST_ID", "1f1b0456-3b50-49ce-a69d-fe1d71a39230");

    std::string source = "-9223372036854775808";
    auto long_long_min = iox::convert::from_string<long long>(source.c_str());
    ASSERT_THAT(long_long_min.has_value(), Eq(true));
    EXPECT_THAT(long_long_min.value(), Eq(std::numeric_limits<long long>::min()));

    source = "9223372036854775807";
    auto long_long_max = iox::convert::from_string<long long>(source.c_str());
    ASSERT_THAT(long_long_max.has_value(), Eq(true));
    EXPECT_THAT(long_long_max.value(), Eq(std::numeric_limits<long long>::max()));
}

TEST_F(convert_test, fromString_SignedLongLong_EdgeCase_OutOfRange_Fail)
{
    ::testing::Test::RecordProperty("TEST_ID", "7c015ac0-06a7-407d-aa93-d39c50734951");

    std::string source = "-9223372036854775809";
    auto long_long_min_dec_1 = iox::convert::from_string<long long>(source.c_str());
    ASSERT_THAT(long_long_min_dec_1.has_value(), Eq(false));

    source = "9223372036854775808";
    auto long_long_max_inc_1 = iox::convert::from_string<long long>(source.c_str());
    ASSERT_THAT(long_long_max_inc_1.has_value(), Eq(false));
}

/// SINGED INTEGRAL EDGE CASES END

/// UNSINGED INTEGRAL EDGE CASES START

TEST_F(convert_test, fromString_UnSignedChar_EdgeCase_InRange_Success)
{
    ::testing::Test::RecordProperty("TEST_ID", "4d2debf7-a6fb-40e1-b672-cb3520be98dd");

    std::string source = "0";
    auto unchar_min = iox::convert::from_string<unsigned char>(source.c_str());
    ASSERT_THAT(unchar_min.has_value(), Eq(true));
    EXPECT_THAT(unchar_min.value(), Eq(0));

    source = "255";
    auto unchar_max = iox::convert::from_string<unsigned char>(source.c_str());
    ASSERT_THAT(unchar_max.has_value(), Eq(true));
    EXPECT_THAT(unchar_max.value(), Eq(std::numeric_limits<unsigned char>::max()));
}

TEST_F(convert_test, fromString_UnSignedChar_EdgeCase_OutOfRange_Fail)
{
    ::testing::Test::RecordProperty("TEST_ID", "c11d74a1-be55-41fc-952f-519546eb04fe");

    std::string source = "-1";
    auto unchar_min_dec_1 = iox::convert::from_string<unsigned char>(source.c_str());
    ASSERT_THAT(unchar_min_dec_1.has_value(), Eq(false));

    source = "256";
    auto unchar_max_inc_1 = iox::convert::from_string<unsigned char>(source.c_str());
    ASSERT_THAT(unchar_max_inc_1.has_value(), Eq(false));
}

TEST_F(convert_test, fromString_UnSignedShort_EdgeCase_InRange_Success)
{
    ::testing::Test::RecordProperty("TEST_ID", "cf168594-e673-4dba-90b7-9b11a3edb967");

    std::string source = "0";
    auto unshort_min = iox::convert::from_string<unsigned short>(source.c_str());
    ASSERT_THAT(unshort_min.has_value(), Eq(true));
    EXPECT_THAT(unshort_min.value(), Eq(0));

    source = "65535";
    auto unshort_max = iox::convert::from_string<unsigned short>(source.c_str());
    ASSERT_THAT(unshort_max.has_value(), Eq(true));
    EXPECT_THAT(unshort_max.value(), Eq(std::numeric_limits<unsigned short>::max()));
}

TEST_F(convert_test, fromString_UnSignedShort_EdgeCase_OutOfRange_Fail)
{
    ::testing::Test::RecordProperty("TEST_ID", "f9196939-ae5d-4c27-85bf-b3b084343261");

    std::string source = "-1";
    auto unshort_min_dec_1 = iox::convert::from_string<unsigned short>(source.c_str());
    ASSERT_THAT(unshort_min_dec_1.has_value(), Eq(false));

    source = "65536";
    auto unshort_max_inc_1 = iox::convert::from_string<unsigned short>(source.c_str());
    ASSERT_THAT(unshort_max_inc_1.has_value(), Eq(false));
}

TEST_F(convert_test, fromString_UnSignedInt_EdgeCase_InRange_Success)
{
    ::testing::Test::RecordProperty("TEST_ID", "c2a832ef-3e86-4303-a98c-63c7b11ea789");

    std::string source = "0";
    auto unint_min = iox::convert::from_string<unsigned int>(source.c_str());
    ASSERT_THAT(unint_min.has_value(), Eq(true));
    EXPECT_THAT(unint_min.value(), Eq(0));

    source = "4294967295";
    auto unint_max = iox::convert::from_string<unsigned int>(source.c_str());
    ASSERT_THAT(unint_max.has_value(), Eq(true));
    EXPECT_THAT(unint_max.value(), Eq(std::numeric_limits<unsigned int>::max()));
}

TEST_F(convert_test, fromString_UnSignedInt_EdgeCase_OutOfRange_Fail)
{
    ::testing::Test::RecordProperty("TEST_ID", "64216d84-c008-47fc-ab9b-0e8d77aeb6c2");

    std::string source = "-1";
    auto unint_min_dec_1 = iox::convert::from_string<unsigned int>(source.c_str());
    ASSERT_THAT(unint_min_dec_1.has_value(), Eq(false));

    source = "4294967296";
    auto unint_max_inc_1 = iox::convert::from_string<unsigned int>(source.c_str());
    ASSERT_THAT(unint_max_inc_1.has_value(), Eq(false));
}

// platform dependent (32/64 bit system only)
TEST_F(convert_test, fromString_UnSignedLong_EdgeCase_InRange_Success)
{
    ::testing::Test::RecordProperty("TEST_ID", "1aaac1ae-3d59-4443-8d88-bfe4e50569a8");

    std::string source = "0";
    auto unlong_min = iox::convert::from_string<unsigned long>(source.c_str());
    ASSERT_THAT(unlong_min.has_value(), Eq(true));
    EXPECT_THAT(unlong_min.value(), Eq(0));

    source = std::to_string(std::numeric_limits<unsigned long>::max());
    auto unlong_max = iox::convert::from_string<unsigned long>(source.c_str());
    ASSERT_THAT(unlong_max.has_value(), Eq(true));
    EXPECT_THAT(unlong_max.value(), Eq(std::numeric_limits<unsigned long>::max()));
}

TEST_F(convert_test, fromString_UnSignedLong_EdgeCase_OutOfRange_Fail)
{
    ::testing::Test::RecordProperty("TEST_ID", "6e74e284-7f13-4d77-8d3f-009df216828f");

    constexpr bool IS_32_BIT{sizeof(long) != sizeof(long long)};

    std::string source = "-1";
    auto unlong_min_dec_1 = iox::convert::from_string<unsigned long>(source.c_str());
    ASSERT_THAT(unlong_min_dec_1.has_value(), Eq(false));

    source = IS_32_BIT ? "4294967296" : "18446744073709551616";
    auto unlong_max_inc_1 = iox::convert::from_string<unsigned long>(source.c_str());
    ASSERT_THAT(unlong_max_inc_1.has_value(), Eq(false));
}

TEST_F(convert_test, fromString_UnSignedLongLong_EdgeCase_InRange_Success)
{
    ::testing::Test::RecordProperty("TEST_ID", "b3c70efd-f875-45bd-b86c-126dc44d3238");

    std::string source = "0";
    auto unlong_long_min = iox::convert::from_string<unsigned long long>(source.c_str());
    ASSERT_THAT(unlong_long_min.has_value(), Eq(true));
    EXPECT_THAT(unlong_long_min.value(), Eq(0));

    source = "18446744073709551615";
    auto unlong_long_max = iox::convert::from_string<unsigned long long>(source.c_str());
    ASSERT_THAT(unlong_long_max.has_value(), Eq(true));
    EXPECT_THAT(unlong_long_max.value(), Eq(std::numeric_limits<unsigned long long>::max()));
}

TEST_F(convert_test, fromString_UnSignedLongLong_EdgeCase_OutOfRange_Fail)
{
    ::testing::Test::RecordProperty("TEST_ID", "96456d6f-2493-4db2-b5fa-f96f92ec64dd");

    std::string source = "-1";
    auto unlong_long_min_dec_1 = iox::convert::from_string<unsigned long long>(source.c_str());
    ASSERT_THAT(unlong_long_min_dec_1.has_value(), Eq(false));

    source = "18446744073709551616";
    auto unlong_long_max_inc_1 = iox::convert::from_string<unsigned long long>(source.c_str());
    ASSERT_THAT(unlong_long_max_inc_1.has_value(), Eq(false));
}

/// UNSINGED INTEGRAL EDGE CASES END

/// NORMAL FLOATING POINT TYPE EDGE CASES START

TEST_F(convert_test, fromString_Float_EdgeCase_InRange_Success)
{
    ::testing::Test::RecordProperty("TEST_ID", "cf849d5d-d0ed-4447-89b8-d6b9f47287c7");

    // the number larger than numeric_limits<long double>::digits10 that will pass all tests for all platforms
    constexpr uint16_t PLATFORM_DIGIT_WORKAROUND_MIN{7};
    constexpr uint16_t PLATFORM_DIGIT_WORKAROUND_MAX{7};

    std::string source = fp_to_string(std::numeric_limits<float>::min(), PLATFORM_DIGIT_WORKAROUND_MIN);
    auto float_min = iox::convert::from_string<float>(source.c_str());
    ASSERT_THAT(float_min.has_value(), Eq(true));
    EXPECT_THAT(float_min.value(), FloatEq(std::numeric_limits<float>::min()));

    source = fp_to_string(std::numeric_limits<float>::lowest(), PLATFORM_DIGIT_WORKAROUND_MAX);
    auto float_lowest = iox::convert::from_string<float>(source.c_str());
    ASSERT_THAT(float_lowest.has_value(), Eq(true));
    EXPECT_THAT(float_lowest.value(), FloatEq(std::numeric_limits<float>::lowest()));

    source = fp_to_string(std::numeric_limits<float>::max(), PLATFORM_DIGIT_WORKAROUND_MAX);
    auto float_max = iox::convert::from_string<float>(source.c_str());
    ASSERT_THAT(float_max.has_value(), Eq(true));
    EXPECT_THAT(float_max.value(), FloatEq(std::numeric_limits<float>::max()));
}

TEST_F(convert_test, fromString_Float_EdgeCase_SubNormalFloat_ShouldFail)
{
    ::testing::Test::RecordProperty("TEST_ID", "68d4f096-a93c-406b-b081-fe50e4b1a2c9");

    auto normal_float_min_eps = std::nextafter(std::numeric_limits<float>::min(), 0.0F);
    std::string source = fp_to_string(std::numeric_limits<float>::min() - normal_float_min_eps);
    auto float_min_dec_eps = iox::convert::from_string<float>(source.c_str());
    ASSERT_THAT(float_min_dec_eps.has_value(), Eq(false));
}

TEST_F(convert_test, fromString_Double_EdgeCase_InRange_Success)
{
    ::testing::Test::RecordProperty("TEST_ID", "d5e5e5ad-92ed-4229-8128-4ee82059fbf7");

    // the number larger than numeric_limits<double>::digits10 that will pass all tests for all platforms
    constexpr uint16_t PLATFORM_DIGIT_WORKAROUND_MIN{19};
    constexpr uint16_t PLATFORM_DIGIT_WORKAROUND_MAX{18};

    std::string source = fp_to_string(std::numeric_limits<double>::min(), PLATFORM_DIGIT_WORKAROUND_MIN);
    auto double_min = iox::convert::from_string<double>(source.c_str());
    ASSERT_THAT(double_min.has_value(), Eq(true));
    EXPECT_THAT(double_min.value(), DoubleEq(std::numeric_limits<double>::min()));

    source = fp_to_string(std::numeric_limits<double>::lowest(), PLATFORM_DIGIT_WORKAROUND_MAX);
    auto double_lowest = iox::convert::from_string<double>(source.c_str());
    ASSERT_THAT(double_lowest.has_value(), Eq(true));
    EXPECT_THAT(double_lowest.value(), DoubleEq(std::numeric_limits<double>::lowest()));

    source = fp_to_string(std::numeric_limits<double>::max(), PLATFORM_DIGIT_WORKAROUND_MAX);
    auto double_max = iox::convert::from_string<double>(source.c_str());
    ASSERT_THAT(double_max.has_value(), Eq(true));
    EXPECT_THAT(double_max.value(), DoubleEq(std::numeric_limits<double>::max()));
}

TEST_F(convert_test, fromString_Double_EdgeCase_SubNormalDouble_ShouldFail)
{
    ::testing::Test::RecordProperty("TEST_ID", "af7ca2e6-ba7e-41f7-a321-5f68617d3566");

    auto normal_double_min_eps = std::nextafter(std::numeric_limits<double>::min(), 0.0);
    std::string source = fp_to_string(std::numeric_limits<double>::min() - normal_double_min_eps);
    auto double_min_dec_eps = iox::convert::from_string<double>(source.c_str());
    ASSERT_THAT(double_min_dec_eps.has_value(), Eq(false));
}

TEST_F(convert_test, fromString_LongDouble_EdgeCase_InRange_Success)
{
    ::testing::Test::RecordProperty("TEST_ID", "cab1c90b-1de0-4654-bbea-4bb4e55e4fc3");

    // the number larger than numeric_limits<long double>::digits10 that will pass all tests for all platforms
    constexpr uint16_t PLATFORM_DIGIT_WORKAROUND_MIN{36};
    constexpr uint16_t PLATFORM_DIGIT_WORKAROUND_MAX{34};

    std::string source = fp_to_string(std::numeric_limits<long double>::min(), PLATFORM_DIGIT_WORKAROUND_MIN);
    auto long_double_min = iox::convert::from_string<long double>(source.c_str());
    ASSERT_THAT(long_double_min.has_value(), Eq(true));
    // NOLINTNEXTLINE(clang-analyzer-core.CallAndMessage)
    EXPECT_THAT(LongDouble::Eq(long_double_min.value(), std::numeric_limits<long double>::min()), Eq(true));

    source = fp_to_string(std::numeric_limits<long double>::lowest(), PLATFORM_DIGIT_WORKAROUND_MAX);
    auto long_double_lowest = iox::convert::from_string<long double>(source.c_str());
    ASSERT_THAT(long_double_lowest.has_value(), Eq(true));
    // NOLINTNEXTLINE(clang-analyzer-core.CallAndMessage)
    EXPECT_THAT(LongDouble::Eq(long_double_lowest.value(), std::numeric_limits<long double>::lowest()), Eq(true));

    source = fp_to_string(std::numeric_limits<long double>::max(), PLATFORM_DIGIT_WORKAROUND_MAX);
    auto long_double_max = iox::convert::from_string<long double>(source.c_str());
    ASSERT_THAT(long_double_max.has_value(), Eq(true));
    // NOLINTNEXTLINE(clang-analyzer-core.CallAndMessage)
    EXPECT_THAT(LongDouble::Eq(long_double_max.value(), std::numeric_limits<long double>::max()), Eq(true));
}

TEST_F(convert_test, fromString_LongDouble_EdgeCase_SubNormalLongDouble_ShouldFail)
{
    ::testing::Test::RecordProperty("TEST_ID", "fb96e526-8fb6-4af9-87f0-dfd4193237a5");

    auto normal_long_double_min_eps = std::nextafter(std::numeric_limits<long double>::min(), 0.0L);
    std::string source = fp_to_string(std::numeric_limits<long double>::min() - normal_long_double_min_eps);
    auto long_double_min_dec_eps = iox::convert::from_string<long double>(source.c_str());
    ASSERT_THAT(long_double_min_dec_eps.has_value(), Eq(false));
}

/// NORMAL FLOATING POINT TYPE EDGE CASES END

/// SPECIAL FLOATING POINT TYPE EDGE CASES START

TEST_F(convert_test, fromString_Float_EdgeCase_Nan_Success)
{
    ::testing::Test::RecordProperty("TEST_ID", "772bcbc3-d55b-464f-873f-82754ad543f3");

    std::vector<std::string> nan_vec = {"NAN", "NaN", "nan"};

    for (const auto& v : nan_vec)
    {
        auto nan_ret = iox::convert::from_string<float>(v.c_str());
        ASSERT_THAT(nan_ret.has_value(), Eq(true));
        ASSERT_THAT(std::isnan(nan_ret.value()), Eq(true));
    }
}

TEST_F(convert_test, fromString_Double_EdgeCase_Nan_Success)
{
    ::testing::Test::RecordProperty("TEST_ID", "a27c8575-658c-465d-a1a2-4f2f6b9a723a");

    std::vector<std::string> nan_vec = {"NAN", "NaN", "nan"};

    for (const auto& v : nan_vec)
    {
        auto nan_ret = iox::convert::from_string<double>(v.c_str());
        ASSERT_THAT(nan_ret.has_value(), Eq(true));
        ASSERT_THAT(std::isnan(nan_ret.value()), Eq(true));
    }
}

TEST_F(convert_test, fromString_LongDouble_EdgeCase_Nan_Success)
{
    ::testing::Test::RecordProperty("TEST_ID", "486f4e78-6000-4401-bb66-62d26b1d0cce");

    std::vector<std::string> nan_vec = {"NAN", "NaN", "nan"};

    for (const auto& v : nan_vec)
    {
        auto nan_ret = iox::convert::from_string<long double>(v.c_str());
        ASSERT_THAT(nan_ret.has_value(), Eq(true));
        ASSERT_THAT(std::isnan(nan_ret.value()), Eq(true));
    }
}

TEST_F(convert_test, fromString_Float_EdgeCase_Inf_Success)
{
    ::testing::Test::RecordProperty("TEST_ID", "82dba3ae-5802-4fbc-aa91-15f4a2953573");

    std::vector<std::string> inf_vec = {
        "INF", "Inf", "inf", "INFINITY", "Infinity", "-INF", "-Inf", "-inf", "-INFINITY", "-Infinity"};

    for (const auto& v : inf_vec)
    {
        auto inf_ret = iox::convert::from_string<float>(v.c_str());
        ASSERT_THAT(inf_ret.has_value(), Eq(true));
        ASSERT_THAT(std::isinf(inf_ret.value()), Eq(true));
    }
}

TEST_F(convert_test, fromString_Double_EdgeCase_Inf_Success)
{
    ::testing::Test::RecordProperty("TEST_ID", "e4ccd01d-b1d1-433e-ba04-548dcc479bb1");

    std::vector<std::string> inf_vec = {
        "INF", "Inf", "inf", "INFINITY", "Infinity", "-INF", "-Inf", "-inf", "-INFINITY", "-Infinity"};

    for (const auto& v : inf_vec)
    {
        auto inf_ret = iox::convert::from_string<double>(v.c_str());
        ASSERT_THAT(inf_ret.has_value(), Eq(true));
        ASSERT_THAT(std::isinf(inf_ret.value()), Eq(true));
    }
}

TEST_F(convert_test, fromString_LongDouble_EdgeCase_Inf_Success)
{
    ::testing::Test::RecordProperty("TEST_ID", "6b8a3284-5f20-4cd6-9958-a2abb348ebe2");

    std::vector<std::string> inf_vec = {
        "INF", "Inf", "inf", "INFINITY", "Infinity", "-INF", "-Inf", "-inf", "-INFINITY", "-Infinity"};

    for (const auto& v : inf_vec)
    {
        auto inf_ret = iox::convert::from_string<long double>(v.c_str());
        ASSERT_THAT(inf_ret.has_value(), Eq(true));
        ASSERT_THAT(std::isinf(inf_ret.value()), Eq(true));
    }
}

TEST_F(convert_test, fromString_Float_EdgeCase_ZeroDecimalNotation_Success)
{
    ::testing::Test::RecordProperty("TEST_ID", "4ac285f9-e107-4d74-8aca-5d87032794db");

    std::vector<std::string> decimal_notation_vec = {"0", "-0", ".0", "-.0", "0.0", "-0.0", "0.", "-0."};

    for (const auto& v : decimal_notation_vec)
    {
        auto decimal_ret = iox::convert::from_string<float>(v.c_str());
        ASSERT_THAT(decimal_ret.has_value(), Eq(true));
        ASSERT_THAT(decimal_ret.value(), Eq(0.0F));
    }
}

TEST_F(convert_test, fromString_Double_EdgeCase_ZeroDecimalNotation_Success)
{
    ::testing::Test::RecordProperty("TEST_ID", "98938eaa-c472-4338-a153-5c2de9eb4940");

    std::vector<std::string> decimal_notation_vec = {"0", "-0", ".0", "-.0", "0.0", "-0.0", "0.", "-0."};

    for (const auto& v : decimal_notation_vec)
    {
        auto decimal_ret = iox::convert::from_string<double>(v.c_str());
        ASSERT_THAT(decimal_ret.has_value(), Eq(true));
        ASSERT_THAT(decimal_ret.value(), Eq(0.0));
    }
}

TEST_F(convert_test, fromString_LongDouble_EdgeCase_ZeroDecimalNotation_Success)
{
    ::testing::Test::RecordProperty("TEST_ID", "49fc0812-47c0-4815-8a15-a94a81493ea0");

    std::vector<std::string> decimal_notation_vec = {"0", "-0", ".0", "-.0", "0.0", "-0.0", "0.", "-0."};

    for (const auto& v : decimal_notation_vec)
    {
        auto decimal_ret = iox::convert::from_string<long double>(v.c_str());
        ASSERT_THAT(decimal_ret.has_value(), Eq(true));
        // NOLINTNEXTLINE(clang-analyzer-core.CallAndMessage)
        ASSERT_THAT(LongDouble::Eq(decimal_ret.value(), 0.0L), Eq(true));
    }
}

TEST_F(convert_test, fromString_Float_EdgeCase_OtherDecimalNotation_Success)
{
    ::testing::Test::RecordProperty("TEST_ID", "278ff5af-28bd-4c11-839e-160e148c5a64");

    std::string source = ".1";

    auto decimal_ret = iox::convert::from_string<float>(source.c_str());
    ASSERT_THAT(decimal_ret.has_value(), Eq(true));
    ASSERT_THAT(decimal_ret.value(), Eq(0.1F));
}

TEST_F(convert_test, fromString_Double_EdgeCase_OtherDecimalNotation_Success)
{
    ::testing::Test::RecordProperty("TEST_ID", "a8539f9a-1c7a-4d81-9a88-ef8a6630f065");

    std::string source = ".1";

    auto decimal_ret = iox::convert::from_string<double>(source.c_str());
    ASSERT_THAT(decimal_ret.has_value(), Eq(true));
    ASSERT_THAT(decimal_ret.value(), Eq(0.1));
}

TEST_F(convert_test, fromString_LongDouble_EdgeCase_OtherDecimalNotation_Success)
{
    ::testing::Test::RecordProperty("TEST_ID", "d71ec687-aaab-45d5-aee5-2ec9a51602d0");

    std::string source = ".1";

    auto decimal_ret = iox::convert::from_string<long double>(source.c_str());
    ASSERT_THAT(decimal_ret.has_value(), Eq(true));
    ASSERT_THAT(decimal_ret.value(), Eq(0.1L));
}

/// SPECIAL FLOATING POINT TYPE EDGE CASES END

TEST_F(convert_test, fromString_ioxString)
{
    ::testing::Test::RecordProperty("TEST_ID", "dbf015bb-5f51-47e1-9d0e-0525f65e7803");
    std::string source = "hello";
    constexpr uint64_t STRING_CAPACITY{8};
    EXPECT_THAT(iox::convert::from_string<iox::string<STRING_CAPACITY>>(source.c_str()).has_value(), Eq(true));
    source = "";
    EXPECT_THAT(iox::convert::from_string<iox::string<STRING_CAPACITY>>(source.c_str()).has_value(), Eq(true));
    source = "12345678";
    EXPECT_THAT(iox::convert::from_string<iox::string<STRING_CAPACITY>>(source.c_str()).has_value(), Eq(true));
    source = "123456789";
    EXPECT_THAT(iox::convert::from_string<iox::string<STRING_CAPACITY>>(source.c_str()).has_value(), Eq(false));
    source = "this_is_a_very_long_string";
    EXPECT_THAT(iox::convert::from_string<iox::string<STRING_CAPACITY>>(source.c_str()).has_value(), Eq(false));
}

} // namespace
