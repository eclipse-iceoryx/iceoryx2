// Copyright (c) 2019 by Robert Bosch GmbH. All rights reserved.
// Copyright (c) 2021 - 2023 by Apex.AI Inc. All rights reserved.
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

#include "test_vocabulary_string.hpp"

#include "iox/detail/hoofs_error_reporting.hpp"
#include "iox/string.hpp"

#include "iceoryx_hoofs/testing/error_reporting/testing_support.hpp"
#include "iceoryx_hoofs/testing/fatal_failure.hpp"
#include "test.hpp"

#include <cstring>

namespace
{
using namespace ::testing;
using namespace iox;
using namespace iox::testing;

TYPED_TEST_SUITE(stringTyped_test, StringImplementations, );

/// @note template <uint64_t N>
/// int64_t compare(const string<N>& other) const noexcept
TYPED_TEST(stringTyped_test, CompareEqStringsResultsInZero)
{
    ::testing::Test::RecordProperty("TEST_ID", "8271fa31-3301-4d2e-a290-a4b61a70dfb5");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    std::string testString(STRINGCAP, 'M');
    EXPECT_THAT(this->testSubject.unsafe_assign(testString.c_str()), Eq(true));
    string<STRINGCAP> fuu;
    EXPECT_THAT(fuu.unsafe_assign(testString.c_str()), Eq(true));
    EXPECT_THAT(this->testSubject.compare(this->testSubject), Eq(0));
    EXPECT_THAT(this->testSubject.compare(fuu), Eq(0));
}

TYPED_TEST(stringTyped_test, CompareResultNegative)
{
    ::testing::Test::RecordProperty("TEST_ID", "b17fc495-a82b-4ee8-af17-28afaabd3f0e");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    std::string testString1(STRINGCAP, 'M');
    EXPECT_THAT(this->testSubject.unsafe_assign(testString1.c_str()), Eq(true));
    string<STRINGCAP> fuu;
    std::string testString2(STRINGCAP, 'L');
    EXPECT_THAT(fuu.unsafe_assign(testString2.c_str()), Eq(true));
    EXPECT_THAT(fuu.compare(this->testSubject), Lt(0));
}

TYPED_TEST(stringTyped_test, CompareResultPositive)
{
    ::testing::Test::RecordProperty("TEST_ID", "ff95b244-937a-4519-90d8-8c82acf01b6b");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    std::string testString1(STRINGCAP, 'M');
    EXPECT_THAT(this->testSubject.unsafe_assign(testString1.c_str()), Eq(true));
    string<STRINGCAP> fuu;
    std::string testString2(STRINGCAP, 'L');
    EXPECT_THAT(fuu.unsafe_assign(testString2.c_str()), Eq(true));
    EXPECT_THAT(this->testSubject.compare(fuu), Gt(0));
}

TYPED_TEST(stringTyped_test, CompareWithEmptyStringResultsInPositive)
{
    ::testing::Test::RecordProperty("TEST_ID", "0495349d-2e63-442f-8214-b44d249f057f");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    string<STRINGCAP> fuu("M");
    EXPECT_THAT(fuu.compare(this->testSubject), Gt(0));
}

TEST(String100, CompareStringsInclNullCharacterWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "a6c1b983-a88c-46bb-a38f-33947e52f209");
    constexpr uint64_t STRING_CAPACITY = 100U;
    constexpr uint64_t STRING_SIZE = 7U;
    std::string testString1{"ice\0ryx", STRING_SIZE};
    std::string testString2{"ice\0rYx", STRING_SIZE};
    string<STRING_CAPACITY> testSubject1(TruncateToCapacity, testString1.c_str(), STRING_SIZE);
    string<STRING_CAPACITY> testSubject2(TruncateToCapacity, testString2.c_str(), STRING_SIZE);
    EXPECT_THAT(testSubject1.compare(testSubject2), Gt(0));
}

TYPED_TEST(stringTyped_test, CompareEqStringsWithDifferentCapaResultsInZero)
{
    ::testing::Test::RecordProperty("TEST_ID", "ddf78ea1-b5cd-44fd-9320-6801893c30e7");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    std::string testString(STRINGCAP, 'M');
    EXPECT_THAT(this->testSubject.unsafe_assign(testString.c_str()), Eq(true));
    string<STRINGCAP + 1U> fuu;
    EXPECT_THAT(fuu.unsafe_assign(testString.c_str()), Eq(true));
    EXPECT_THAT(this->testSubject.compare(this->testSubject), Eq(0));
    EXPECT_THAT(this->testSubject.compare(fuu), Eq(0));
}

TYPED_TEST(stringTyped_test, CompareResultNegativeWithDifferentCapa)
{
    ::testing::Test::RecordProperty("TEST_ID", "a0499ebc-249c-4b31-a1b0-0e34035e77f2");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    std::string testString1(STRINGCAP, 'M');
    EXPECT_THAT(this->testSubject.unsafe_assign(testString1.c_str()), Eq(true));
    string<STRINGCAP + 1U> fuu;
    std::string testString2(STRINGCAP + 1U, 'M');
    EXPECT_THAT(fuu.unsafe_assign(testString2.c_str()), Eq(true));
    EXPECT_THAT(this->testSubject.compare(fuu), Lt(0));
}

TYPED_TEST(stringTyped_test, CompareResultPositiveWithDifferentCapa)
{
    ::testing::Test::RecordProperty("TEST_ID", "97fed2cb-4f25-4732-9bbe-4d710b9a35f7");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    std::string testString1(STRINGCAP, 'M');
    EXPECT_THAT(this->testSubject.unsafe_assign(testString1.c_str()), Eq(true));
    string<STRINGCAP + 1U> fuu;
    std::string testString2(STRINGCAP + 1U, 'M');
    EXPECT_THAT(fuu.unsafe_assign(testString2.c_str()), Eq(true));
    EXPECT_THAT(fuu.compare(this->testSubject), Gt(0));
}

TYPED_TEST(stringTyped_test, CompareWithEmptyStringOfDifferentCapaResultsInPositive)
{
    ::testing::Test::RecordProperty("TEST_ID", "9d54c681-fc19-444b-8e1b-059ff6237a8f");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    string<STRINGCAP + 1U> fuu("M");
    EXPECT_THAT(fuu.compare(this->testSubject), Gt(0));
}

TEST(String100, CompareStringsWithDifferentCapaInclNullCharacterWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "fe260cb6-5d77-42b1-89b8-073c9ea9593d");
    constexpr uint64_t STRING_SIZE = 7U;
    std::string testString1{"ice\0ryx", STRING_SIZE};
    std::string testString2{"ice\0rYx", STRING_SIZE};
    string<200> testSubject1(TruncateToCapacity, testString1.c_str(), STRING_SIZE);
    string<100> testSubject2(TruncateToCapacity, testString2.c_str(), STRING_SIZE);
    EXPECT_THAT(testSubject1.compare(testSubject2), Gt(0));
}

/// @note template <uint64_t N>
/// bool operator==(const string<N>& rhs) const noexcept
TYPED_TEST(stringTyped_test, CompareOperatorEqualResultTrue)
{
    ::testing::Test::RecordProperty("TEST_ID", "149069dd-a2f2-441a-9d16-30aa038a7c5e");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    string<STRINGCAP> fuu("M");
    EXPECT_THAT(fuu == fuu, Eq(true));
}

TYPED_TEST(stringTyped_test, CompareOperatorEqualResultFalse)
{
    ::testing::Test::RecordProperty("TEST_ID", "30c9bb50-03ea-437e-99c8-c663ed62340b");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    string<STRINGCAP> fuu("M");
    string<STRINGCAP> bar("L");
    EXPECT_THAT(fuu == bar, Eq(false));
}

TYPED_TEST(stringTyped_test, CompareOperatorEqualResultTrueWithDifferentCapa)
{
    ::testing::Test::RecordProperty("TEST_ID", "10ce496d-635e-4aeb-9969-95881d1efc87");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    string<STRINGCAP> testString1("M");
    string<STRINGCAP + 1U> testString2("M");
    EXPECT_THAT(testString1 == testString2, Eq(true));
}

TYPED_TEST(stringTyped_test, CompareOperatorEqualResultFalseWithDifferentCapa)
{
    ::testing::Test::RecordProperty("TEST_ID", "15444c15-7319-4224-8519-091d2b47da22");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    string<STRINGCAP + 1U> testString1("M");
    string<STRINGCAP> testString2("L");
    string<STRINGCAP + 1U> testString3;
    std::string testStdString(STRINGCAP + 1U, 'L');
    EXPECT_THAT(testString3.unsafe_assign(testStdString.c_str()), Eq(true));
    EXPECT_THAT(testString1 == testString2, Eq(false));
    EXPECT_THAT(testString3 == testString2, Eq(false));
}

/// @note template <uint64_t N>
/// bool operator!=(const string<N>& rhs) const noexcept
TYPED_TEST(stringTyped_test, CompareOperatorNotEqualResultFalse)
{
    ::testing::Test::RecordProperty("TEST_ID", "b31770c0-2695-4fd6-b5b6-cae479852417");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    string<STRINGCAP> fuu("M");
    EXPECT_THAT(fuu != fuu, Eq(false));
}

TYPED_TEST(stringTyped_test, CompareOperatorNotEqualResultTrue)
{
    ::testing::Test::RecordProperty("TEST_ID", "57db46d4-df73-49da-aa43-0dfc8ff04f44");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    string<STRINGCAP> fuu("M");
    string<STRINGCAP> bar("L");
    EXPECT_THAT(fuu != bar, Eq(true));
}

TYPED_TEST(stringTyped_test, CompareOperatorNotEqualResultFalseWithDifferentCapa)
{
    ::testing::Test::RecordProperty("TEST_ID", "f5b53871-4bb0-4f2a-adea-e5335c6c4611");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    string<STRINGCAP> testString1("M");
    string<STRINGCAP + 1U> testString2("M");
    EXPECT_THAT(testString1 != testString2, Eq(false));
}

TYPED_TEST(stringTyped_test, CompareOperatorNotEqualResultTrueWithDifferentCapa)
{
    ::testing::Test::RecordProperty("TEST_ID", "394d3dd9-2304-4608-9f29-12cfaacaeef7");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    string<STRINGCAP + 1U> testString1("M");
    string<STRINGCAP> testString2("L");
    string<STRINGCAP + 1U> testString3;
    std::string testStdString(STRINGCAP + 1U, 'L');
    EXPECT_THAT(testString3.unsafe_assign(testStdString.c_str()), Eq(true));
    EXPECT_THAT(testString1 != testString2, Eq(true));
    EXPECT_THAT(testString3 != testString2, Eq(true));
}

/// @note template <uint64_t N>
/// bool operator<(const string<N>& rhs) const noexcept
TYPED_TEST(stringTyped_test, CompareOperatorLessResultTrue)
{
    ::testing::Test::RecordProperty("TEST_ID", "cb8495ba-f2f9-4e7c-a017-f8295fcff518");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    string<STRINGCAP> fuu("M");
    string<STRINGCAP> bar("L");
    EXPECT_THAT(bar < fuu, Eq(true));
}

TYPED_TEST(stringTyped_test, CompareOperatorLessResultFalse)
{
    ::testing::Test::RecordProperty("TEST_ID", "6bf80f49-f637-4207-900e-2cfec89eb556");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    string<STRINGCAP> fuu("M");
    string<STRINGCAP> bar("L");
    EXPECT_THAT(fuu < bar, Eq(false));
    EXPECT_THAT(fuu < fuu, Eq(false));
}

TYPED_TEST(stringTyped_test, CompareOperatorLessResultTrueWithDifferentCapa)
{
    ::testing::Test::RecordProperty("TEST_ID", "bbe252cb-6fba-45d1-ad31-cac42fb73caa");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    string<STRINGCAP> testString1("M");
    string<STRINGCAP + 1U> testString2("L");
    EXPECT_THAT(testString2 < testString1, Eq(true));
}

TYPED_TEST(stringTyped_test, CompareOperatorLessResultFalseWithDifferentCapa)
{
    ::testing::Test::RecordProperty("TEST_ID", "884217ee-085b-479d-8ab1-a0476f14105c");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    string<STRINGCAP + 1U> testString1("M");
    string<STRINGCAP> testString2("L");
    EXPECT_THAT(testString1 < testString2, Eq(false));
    EXPECT_THAT(testString1 < testString1, Eq(false));
}

/// @note template <uint64_t N>
/// bool operator<=(const string<N>& rhs) const noexcept
TYPED_TEST(stringTyped_test, CompareOperatorLessEqResultTrue)
{
    ::testing::Test::RecordProperty("TEST_ID", "3a8ba399-4e2c-483d-835a-6d4bcbfa9a29");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    string<STRINGCAP> fuu("M");
    string<STRINGCAP> bar("L");
    EXPECT_THAT(this->testSubject <= fuu, Eq(true));
    EXPECT_THAT(bar <= fuu, Eq(true));
}

TYPED_TEST(stringTyped_test, CompareOperatorLessEqResultFalse)
{
    ::testing::Test::RecordProperty("TEST_ID", "59928731-2d4b-4122-8924-7809001fa631");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    string<STRINGCAP> fuu("M");
    string<STRINGCAP> bar("L");
    EXPECT_THAT(fuu <= bar, Eq(false));
}

TYPED_TEST(stringTyped_test, CompareOperatorLessEqResultTrueWithDifferentCapa)
{
    ::testing::Test::RecordProperty("TEST_ID", "60c20357-c67a-4d4f-aca7-1ce3ab84677c");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    string<STRINGCAP> fuu("M");
    string<STRINGCAP + 1U> bar("L");
    EXPECT_THAT(this->testSubject <= fuu, Eq(true));
    EXPECT_THAT(bar <= fuu, Eq(true));
}

TYPED_TEST(stringTyped_test, CompareOperatorLessEqResultFalseWithDifferentCapa)
{
    ::testing::Test::RecordProperty("TEST_ID", "9126ec2b-eb0c-4567-aa9e-8009e9910571");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    string<STRINGCAP + 1U> fuu("M");
    string<STRINGCAP> bar("L");
    EXPECT_THAT(fuu <= bar, Eq(false));
}

/// @note template <uint64_t N>
/// bool operator>(const string<N>& rhs) const noexcept
TYPED_TEST(stringTyped_test, CompareOperatorGreaterResultTrue)
{
    ::testing::Test::RecordProperty("TEST_ID", "92c81295-3169-49ac-a12f-700a5dd2179b");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    string<STRINGCAP> fuu("M");
    string<STRINGCAP> bar("L");
    EXPECT_THAT(fuu > bar, Eq(true));
}

TYPED_TEST(stringTyped_test, CompareOperatorGreaterResultFalse)
{
    ::testing::Test::RecordProperty("TEST_ID", "5b3d33af-c905-44de-9ece-c66a96e59fb9");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    string<STRINGCAP> fuu("M");
    string<STRINGCAP> bar("L");
    EXPECT_THAT(bar > fuu, Eq(false));
    EXPECT_THAT(bar > bar, Eq(false));
}

TYPED_TEST(stringTyped_test, CompareOperatorGreaterResultTrueWithDifferentCapa)
{
    ::testing::Test::RecordProperty("TEST_ID", "fb2b2abd-4670-44e9-a1ea-e1ae2d437ce2");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    string<STRINGCAP + 1U> fuu("M");
    string<STRINGCAP> bar("L");
    EXPECT_THAT(fuu > bar, Eq(true));
}

TYPED_TEST(stringTyped_test, CompareOperatorGreaterResultFalseWithDifferentCapa)
{
    ::testing::Test::RecordProperty("TEST_ID", "5bacb492-2d28-4260-96d2-516858462054");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    string<STRINGCAP> fuu("M");
    string<STRINGCAP + 1U> bar("L");
    EXPECT_THAT(bar > fuu, Eq(false));
    EXPECT_THAT(bar > bar, Eq(false));
}

/// @note template <uint64_t N>
/// bool operator>=(const string<N>& rhs) const noexcept
TYPED_TEST(stringTyped_test, CompareOperatorGreaterEqResultTrue)
{
    ::testing::Test::RecordProperty("TEST_ID", "97184a06-2153-4b8e-93b7-0c620b90efb2");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    string<STRINGCAP> fuu("M");
    string<STRINGCAP> bar("L");
    this->testSubject = "M";
    EXPECT_THAT(fuu >= bar, Eq(true));
    EXPECT_THAT(fuu >= this->testSubject, Eq(true));
}

TYPED_TEST(stringTyped_test, CompareOperatorGreaterEqResultFalse)
{
    ::testing::Test::RecordProperty("TEST_ID", "72dca46d-86aa-4a01-b282-97b46f7f1504");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    string<STRINGCAP> fuu("M");
    string<STRINGCAP> bar("L");
    EXPECT_THAT(bar >= fuu, Eq(false));
}

TYPED_TEST(stringTyped_test, CompareOperatorGreaterEqResultTrueWithDifferentCapa)
{
    ::testing::Test::RecordProperty("TEST_ID", "edb0b737-630e-4305-9fd6-0f9c8be524b4");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    string<STRINGCAP + 1U> fuu("M");
    string<STRINGCAP> bar("L");
    this->testSubject = "M";
    EXPECT_THAT(fuu >= bar, Eq(true));
    EXPECT_THAT(fuu >= this->testSubject, Eq(true));
}

TYPED_TEST(stringTyped_test, CompareOperatorGreaterEqResultFalseWithDifferentCapa)
{
    ::testing::Test::RecordProperty("TEST_ID", "aa2e606e-28a2-424b-ac98-b1d194a11738");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    string<STRINGCAP> fuu("M");
    string<STRINGCAP + 1U> bar("L");
    this->testSubject = "L";
    EXPECT_THAT(bar >= fuu, Eq(false));
}

/// @note int64_t compare(const T& other) const noexcept
/// with T = {char array}
TYPED_TEST(stringTyped_test, CompareEqCharArrayResultsInZero)
{
    ::testing::Test::RecordProperty("TEST_ID", "13a0f1a3-b006-4686-a5a0-3c6a2c7113e0");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    std::string testStdString(STRINGCAP, 'M');
    ASSERT_THAT(this->testSubject.unsafe_assign(testStdString.c_str()), Eq(true));

    // required to verify string literal functionality of iox::string
    // NOLINTNEXTLINE(hicpp-avoid-c-arrays, cppcoreguidelines-avoid-c-arrays)
    char testCharArray[STRINGCAP + 1U];
    for (uint64_t i = 0U; i < STRINGCAP; ++i)
    {
        // NOLINTJUSTIFICATION no other way to populate testCharArray
        // NOLINTNEXTLINE(cppcoreguidelines-pro-bounds-constant-array-index)
        testCharArray[i] = 'M';
    }
    testCharArray[STRINGCAP] = '\0';
    EXPECT_THAT(this->testSubject.compare(testCharArray), Eq(0));
}

TYPED_TEST(stringTyped_test, CompareWithCharArrayResultNegative)
{
    ::testing::Test::RecordProperty("TEST_ID", "df4a32d0-72b1-4c65-86f3-15b007ab003c");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    std::string temp(STRINGCAP, 'L');
    ASSERT_THAT(this->testSubject.unsafe_assign(temp.c_str()), Eq(true));

    // required to verify string literal functionality of iox::string
    // NOLINTNEXTLINE(hicpp-avoid-c-arrays, cppcoreguidelines-avoid-c-arrays)
    char testCharArray[STRINGCAP + 1U];
    for (uint64_t i = 0U; i < STRINGCAP; ++i)
    {
        // NOLINTJUSTIFICATION no other way to populate testCharArray
        // NOLINTNEXTLINE(cppcoreguidelines-pro-bounds-constant-array-index)
        testCharArray[i] = 'M';
    }
    testCharArray[STRINGCAP] = '\0';
    EXPECT_THAT(this->testSubject.compare(testCharArray), Lt(0));
}

TYPED_TEST(stringTyped_test, CompareWithCharArrayResultPositive)
{
    ::testing::Test::RecordProperty("TEST_ID", "d315afbc-558d-474a-8ae5-f53451526c73");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    std::string temp(STRINGCAP, 'M');
    ASSERT_THAT(this->testSubject.unsafe_assign(temp.c_str()), Eq(true));

    // required to verify string literal functionality of iox::string
    // NOLINTNEXTLINE(hicpp-avoid-c-arrays, cppcoreguidelines-avoid-c-arrays)
    char testCharArray[STRINGCAP + 1U];
    for (uint64_t i = 0U; i < STRINGCAP; ++i)
    {
        // NOLINTJUSTIFICATION no other way to populate testCharArray
        // NOLINTNEXTLINE(cppcoreguidelines-pro-bounds-constant-array-index)
        testCharArray[i] = 'L';
    }
    testCharArray[STRINGCAP] = '\0';
    EXPECT_THAT(this->testSubject.compare(testCharArray), Gt(0));
}

TYPED_TEST(stringTyped_test, CompareWithEmptyCharArrayResultsInPositive)
{
    ::testing::Test::RecordProperty("TEST_ID", "a871dfe3-4acd-437c-b315-de3c43ece19b");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    this->testSubject = "M";

    // required to verify string literal functionality of iox::string
    // NOLINTNEXTLINE(hicpp-avoid-c-arrays, cppcoreguidelines-avoid-c-arrays)
    char testCharArray[STRINGCAP + 1U] = {'\0'};
    EXPECT_THAT(this->testSubject.compare(testCharArray), Gt(0));
}

TYPED_TEST(stringTyped_test, CompareEqStringAndCharArrayWithDifferentCapaResultsInZero)
{
    ::testing::Test::RecordProperty("TEST_ID", "d0e86a0c-f68d-4d88-8e8b-f65f82a1e7aa");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    std::string temp(STRINGCAP, 'M');
    ASSERT_THAT(this->testSubject.unsafe_assign(temp.c_str()), Eq(true));

    // required to verify string literal functionality of iox::string
    // NOLINTNEXTLINE(hicpp-avoid-c-arrays, cppcoreguidelines-avoid-c-arrays)
    char testCharArray[STRINGCAP + 2U];
    for (uint64_t i = 0U; i < STRINGCAP; ++i)
    {
        // NOLINTJUSTIFICATION no other way to populate testCharArray
        // NOLINTNEXTLINE(cppcoreguidelines-pro-bounds-constant-array-index)
        testCharArray[i] = 'M';
    }
    testCharArray[STRINGCAP] = '\0';
    EXPECT_THAT(this->testSubject.compare(testCharArray), Eq(0));
}

TYPED_TEST(stringTyped_test, CompareWithCharArrayResultNegativeWithDifferentCapa)
{
    ::testing::Test::RecordProperty("TEST_ID", "3efb771d-88e5-4775-8ce8-ba14a4158930");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    std::string temp(STRINGCAP, 'M');
    ASSERT_THAT(this->testSubject.unsafe_assign(temp.c_str()), Eq(true));

    // required to verify string literal functionality of iox::string
    // NOLINTNEXTLINE(hicpp-avoid-c-arrays, cppcoreguidelines-avoid-c-arrays)
    char testCharArray[STRINGCAP + 2U];
    for (uint64_t i = 0U; i < STRINGCAP + 1U; ++i)
    {
        // NOLINTJUSTIFICATION no other way to populate testCharArray
        // NOLINTNEXTLINE(cppcoreguidelines-pro-bounds-constant-array-index)
        testCharArray[i] = 'M';
    }
    testCharArray[STRINGCAP + 1U] = '\0';
    EXPECT_THAT(this->testSubject.compare(testCharArray), Lt(0));
}

TYPED_TEST(stringTyped_test, CompareWithCharArrayResultPositiveWithDifferentCapa)
{
    ::testing::Test::RecordProperty("TEST_ID", "c2c627d5-c633-4368-b7dc-f674315553f7");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    std::string temp(STRINGCAP + 1U, 'M');
    string<STRINGCAP + 1U> sut;
    ASSERT_THAT(sut.unsafe_assign(temp.c_str()), Eq(true));

    // required to verify string literal functionality of iox::string
    // NOLINTNEXTLINE(hicpp-avoid-c-arrays, cppcoreguidelines-avoid-c-arrays)
    char testCharArray[STRINGCAP];
    for (uint64_t i = 0U; i < STRINGCAP - 1U; ++i)
    {
        // NOLINTJUSTIFICATION no other way to populate testCharArray
        // NOLINTNEXTLINE(cppcoreguidelines-pro-bounds-constant-array-index)
        testCharArray[i] = 'M';
    }
    testCharArray[STRINGCAP - 1U] = '\0';
    EXPECT_THAT(sut.compare(testCharArray), Gt(0));
}

TYPED_TEST(stringTyped_test, CompareWithEmptyCharArrayOfDifferentCapaResultsInPositive)
{
    ::testing::Test::RecordProperty("TEST_ID", "7fe1a772-6004-470e-8355-e54156571fa0");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    string<STRINGCAP + 1U> sut("M");

    // required to verify string literal functionality of iox::string
    // NOLINTNEXTLINE(hicpp-avoid-c-arrays, cppcoreguidelines-avoid-c-arrays)
    char testCharArray[STRINGCAP] = {'\0'};
    EXPECT_THAT(sut.compare(testCharArray), Gt(0));
}

/// @note bool operator==(const T& rhs) const noexcept
/// bool operator!=(const T& rhs) const noexcept
/// bool operator==(const T& lhs, const string<Capacity>& rhs) noexcept
/// bool operator!=(const T& lhs, const string<Capacity>& rhs) noexcept
/// with T = {char array}
TYPED_TEST(stringTyped_test, CheckForEqualityWithEqualStringsWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "38d8e0ca-97c5-4e3f-9cb7-589bb7de3b71");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    this->testSubject = "M";

    // required to verify string literal functionality of iox::string
    // NOLINTNEXTLINE(hicpp-avoid-c-arrays, cppcoreguidelines-avoid-c-arrays)
    const char testCharArray[STRINGCAP + 1U] = {'M'};
    EXPECT_THAT(this->testSubject == testCharArray, Eq(true));
    EXPECT_THAT(testCharArray == this->testSubject, Eq(true));
    EXPECT_THAT(this->testSubject != testCharArray, Eq(false));
    EXPECT_THAT(testCharArray != this->testSubject, Eq(false));
}

TYPED_TEST(stringTyped_test, CheckForEqualityWithUnequalStringsWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "129d4f55-71cd-4b6a-b07e-1f53069c117b");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    this->testSubject = "M";

    // required to verify string literal functionality of iox::string
    // NOLINTNEXTLINE(hicpp-avoid-c-arrays, cppcoreguidelines-avoid-c-arrays)
    char testCharArray[STRINGCAP + 1U] = {'L'};
    EXPECT_THAT(this->testSubject == testCharArray, Eq(false));
    EXPECT_THAT(testCharArray == this->testSubject, Eq(false));
    EXPECT_THAT(this->testSubject != testCharArray, Eq(true));
    EXPECT_THAT(testCharArray != this->testSubject, Eq(true));
}

TYPED_TEST(stringTyped_test, CheckForEqualityWithEqualStringWithDifferentCapaWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "a01cc09b-fe94-42db-bcbd-4f813f4acd62");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    this->testSubject = "M";

    // required to verify string literal functionality of iox::string
    // NOLINTNEXTLINE(hicpp-avoid-c-arrays, cppcoreguidelines-avoid-c-arrays)
    char testCharArray[STRINGCAP + 5U] = {'M'};
    EXPECT_THAT(this->testSubject == testCharArray, Eq(true));
    EXPECT_THAT(testCharArray == this->testSubject, Eq(true));
    EXPECT_THAT(this->testSubject != testCharArray, Eq(false));
    EXPECT_THAT(testCharArray != this->testSubject, Eq(false));

    // required to verify string literal functionality of iox::string
    // NOLINTNEXTLINE(hicpp-avoid-c-arrays, cppcoreguidelines-avoid-c-arrays)
    const char test[] = {'M'};
    EXPECT_THAT(test == this->testSubject, Eq(true));
}

TYPED_TEST(stringTyped_test, CheckForEqualityWithUnequalStringWithDifferentSizeWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "d678927a-f629-433c-ad10-1d62338c816b");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    this->testSubject = "M";

    // required to verify string literal functionality of iox::string
    // NOLINTNEXTLINE(hicpp-avoid-c-arrays, cppcoreguidelines-avoid-c-arrays)
    char testCharArray[STRINGCAP + 3U] = {'M', 'L'};
    EXPECT_THAT(this->testSubject == testCharArray, Eq(false));
    EXPECT_THAT(testCharArray == this->testSubject, Eq(false));
    EXPECT_THAT(this->testSubject != testCharArray, Eq(true));
    EXPECT_THAT(testCharArray != this->testSubject, Eq(true));
}

/// @note bool operator<(const T& rhs) const noexcept
/// bool operator<=(const T& rhs) const noexcept
/// bool operator>(const T& rhs) const noexcept
/// bool operator>=(const T& rhs) const noexcept
/// bool operator<(const T& lhs, const string<Capacity>& rhs) noexcept
/// bool operator<=(const T& lhs, const string<Capacity>& rhs) noexcept
/// bool operator>(const T& lhs, const string<Capacity>& rhs) noexcept
/// bool operator>=(const T& lhs, const string<Capacity>& rhs) noexcept
/// with T = {char array}
TYPED_TEST(stringTyped_test, CompareOperatorsWithDifferentStrings)
{
    ::testing::Test::RecordProperty("TEST_ID", "9dcd5cce-ce7d-4cf9-8c36-edca46d09ff7");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();

    // required to verify string literal functionality of iox::string
    // NOLINTNEXTLINE(hicpp-avoid-c-arrays, cppcoreguidelines-avoid-c-arrays)
    char testCharArray[STRINGCAP + 1U] = {'L'};

    // compare with greater string
    string<STRINGCAP> sutGreater("M");

    EXPECT_THAT(sutGreater < testCharArray, Eq(false));
    EXPECT_THAT(sutGreater <= testCharArray, Eq(false));
    EXPECT_THAT(sutGreater > testCharArray, Eq(true));
    EXPECT_THAT(sutGreater >= testCharArray, Eq(true));
    EXPECT_THAT(testCharArray < sutGreater, Eq(true));
    EXPECT_THAT(testCharArray <= sutGreater, Eq(true));
    EXPECT_THAT(testCharArray > sutGreater, Eq(false));
    EXPECT_THAT(testCharArray >= sutGreater, Eq(false));

    // compare with less string
    string<STRINGCAP> sutLess("F");

    EXPECT_THAT(sutLess < testCharArray, Eq(true));
    EXPECT_THAT(sutLess <= testCharArray, Eq(true));
    EXPECT_THAT(sutLess > testCharArray, Eq(false));
    EXPECT_THAT(sutLess >= testCharArray, Eq(false));
    EXPECT_THAT(testCharArray < sutLess, Eq(false));
    EXPECT_THAT(testCharArray <= sutLess, Eq(false));
    EXPECT_THAT(testCharArray > sutLess, Eq(true));
    EXPECT_THAT(testCharArray >= sutLess, Eq(true));
}

TYPED_TEST(stringTyped_test, CompareOperatorsWithEqualStrings)
{
    ::testing::Test::RecordProperty("TEST_ID", "6a07445e-8d0b-43f6-92b7-70a390ad6a27");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    this->testSubject = "M";

    // required to verify string literal functionality of iox::string
    // NOLINTNEXTLINE(hicpp-avoid-c-arrays, cppcoreguidelines-avoid-c-arrays)
    char testCharArray[STRINGCAP + 1U] = {'M'};
    EXPECT_THAT(this->testSubject < testCharArray, Eq(false));
    EXPECT_THAT(this->testSubject <= testCharArray, Eq(true));
    EXPECT_THAT(this->testSubject > testCharArray, Eq(false));
    EXPECT_THAT(this->testSubject >= testCharArray, Eq(true));

    EXPECT_THAT(testCharArray < this->testSubject, Eq(false));
    EXPECT_THAT(testCharArray <= this->testSubject, Eq(true));
    EXPECT_THAT(testCharArray > this->testSubject, Eq(false));
    EXPECT_THAT(testCharArray >= this->testSubject, Eq(true));
}

TYPED_TEST(stringTyped_test, CompareOperatorsWithDifferentStringWithDifferentSize)
{
    ::testing::Test::RecordProperty("TEST_ID", "6ee0d514-12b8-48e5-b19b-997af585fc95");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();

    // required to verify string literal functionality of iox::string
    // NOLINTNEXTLINE(hicpp-avoid-c-arrays, cppcoreguidelines-avoid-c-arrays)
    char testCharArray[STRINGCAP + 1U];
    for (uint64_t i = 0U; i < STRINGCAP; ++i)
    {
        // NOLINTJUSTIFICATION no other way to populate testCharArray
        // NOLINTNEXTLINE(cppcoreguidelines-pro-bounds-constant-array-index)
        testCharArray[i] = 'L';
    }
    testCharArray[STRINGCAP] = '\0';

    // compare with greater string
    std::string temp1(STRINGCAP + 5U, 'M');
    string<STRINGCAP + 5U> sutGreater;
    ASSERT_THAT(sutGreater.unsafe_assign(temp1.c_str()), Eq(true));

    EXPECT_THAT(sutGreater < testCharArray, Eq(false));
    EXPECT_THAT(sutGreater <= testCharArray, Eq(false));
    EXPECT_THAT(sutGreater > testCharArray, Eq(true));
    EXPECT_THAT(sutGreater >= testCharArray, Eq(true));
    EXPECT_THAT(testCharArray < sutGreater, Eq(true));
    EXPECT_THAT(testCharArray <= sutGreater, Eq(true));
    EXPECT_THAT(testCharArray > sutGreater, Eq(false));
    EXPECT_THAT(testCharArray >= sutGreater, Eq(false));

    // compare with less string
    std::string temp2(STRINGCAP + 5U, 'F');
    string<STRINGCAP + 5U> sutLess;
    ASSERT_THAT(sutLess.unsafe_assign(temp2.c_str()), Eq(true));

    EXPECT_THAT(sutLess < testCharArray, Eq(true));
    EXPECT_THAT(sutLess <= testCharArray, Eq(true));
    EXPECT_THAT(sutLess > testCharArray, Eq(false));
    EXPECT_THAT(sutLess >= testCharArray, Eq(false));
    EXPECT_THAT(testCharArray < sutLess, Eq(false));
    EXPECT_THAT(testCharArray <= sutLess, Eq(false));
    EXPECT_THAT(testCharArray > sutLess, Eq(true));
    EXPECT_THAT(testCharArray >= sutLess, Eq(true));
}

TYPED_TEST(stringTyped_test, CompareOperatorsWithEqualStringWithDifferentCapa)
{
    ::testing::Test::RecordProperty("TEST_ID", "7ccedecd-215c-4bd7-89ff-d0bb25c206bb");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    std::string temp(STRINGCAP, 'M');
    ASSERT_THAT(this->testSubject.unsafe_assign(temp.c_str()), Eq(true));

    constexpr uint64_t TEST_CHAR_ARRAY_CAPACITY = STRINGCAP + 6U;
    // NOLINTJUSTIFICATION required to verify string literal functionality of iox::string
    // NOLINTNEXTLINE(hicpp-avoid-c-arrays, cppcoreguidelines-avoid-c-arrays)
    char testCharArray[TEST_CHAR_ARRAY_CAPACITY];
    for (auto& c : testCharArray)
    {
        c = 'M';
    }
    testCharArray[STRINGCAP] = '\0';
    EXPECT_THAT(this->testSubject < testCharArray, Eq(false));
    EXPECT_THAT(this->testSubject <= testCharArray, Eq(true));
    EXPECT_THAT(this->testSubject > testCharArray, Eq(false));
    EXPECT_THAT(this->testSubject >= testCharArray, Eq(true));

    EXPECT_THAT(testCharArray < this->testSubject, Eq(false));
    EXPECT_THAT(testCharArray <= this->testSubject, Eq(true));
    EXPECT_THAT(testCharArray > this->testSubject, Eq(false));
    EXPECT_THAT(testCharArray >= this->testSubject, Eq(true));
}

/// @note int64_t compare(char other) const noexcept
TYPED_TEST(stringTyped_test, CompareEqCharResultsInZero)
{
    ::testing::Test::RecordProperty("TEST_ID", "94837615-8171-4da4-8157-19b4f8f170d1");
    this->testSubject = "M";
    const char testChar = 'M';
    EXPECT_THAT(this->testSubject.compare(testChar), Eq(0));
}

TYPED_TEST(stringTyped_test, CompareWithCharResultNegative)
{
    ::testing::Test::RecordProperty("TEST_ID", "b84785ae-3162-4ff1-a9b0-24405c4b381e");
    this->testSubject = "L";
    const char testChar = 'M';
    EXPECT_THAT(this->testSubject.compare(testChar), Lt(0));
}

TYPED_TEST(stringTyped_test, CompareWithCharResultPositive)
{
    ::testing::Test::RecordProperty("TEST_ID", "3339c92d-a45e-4f95-9825-d9699e2bc734");
    this->testSubject = "M";
    const char testChar = 'L';
    EXPECT_THAT(this->testSubject.compare(testChar), Gt(0));
}

TYPED_TEST(stringTyped_test, CompareWithCharResultPositiveWithDifferentSize)
{
    ::testing::Test::RecordProperty("TEST_ID", "93e57b3b-0c17-4314-bcfb-26f776519d44");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    std::string temp(STRINGCAP + 3U, 'M');
    string<STRINGCAP + 3U> sut;
    ASSERT_THAT(sut.unsafe_assign(temp.c_str()), Eq(true));

    char testChar1 = 'L';
    EXPECT_THAT(sut.compare(testChar1), Gt(0));
    char testChar2 = 'M';
    EXPECT_THAT(sut.compare(testChar2), Gt(0));
}

TYPED_TEST(stringTyped_test, CompareEmptyStringWithCharWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "d0857725-4f4a-4052-a957-71fc906d535e");
    EXPECT_THAT(this->testSubject.compare('A'), Lt(0));
    EXPECT_THAT(this->testSubject.compare('\0'), Lt(0));
}

/// @note bool operator==(const char& rhs) const noexcept
/// bool operator!=(const char& rhs) const noexcept
TYPED_TEST(stringTyped_test, CheckForEqualityWithEqualCharWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "d4e79679-939f-4aa9-9db1-4b3ba38b5dc7");
    this->testSubject = "M";

    const char testChar = 'M';
    EXPECT_THAT(this->testSubject == testChar, Eq(true));
    EXPECT_THAT(testChar == this->testSubject, Eq(true));
    EXPECT_THAT(this->testSubject != testChar, Eq(false));
    EXPECT_THAT(testChar != this->testSubject, Eq(false));
}

TYPED_TEST(stringTyped_test, CheckForEqualityWithUnequalCharWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "1547d1dd-bf34-45ec-be4e-64fdc7164404");
    this->testSubject = "M";

    char testChar = 'L';
    EXPECT_THAT(this->testSubject == testChar, Eq(false));
    EXPECT_THAT(testChar == this->testSubject, Eq(false));
    EXPECT_THAT(this->testSubject != testChar, Eq(true));
    EXPECT_THAT(testChar != this->testSubject, Eq(true));
}

TYPED_TEST(stringTyped_test, CheckForEqualityWithCharWithDifferentSizeWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "c49cc0a3-b31c-49ec-90e2-707216df5eaa");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    std::string temp(STRINGCAP + 4U, 'M');
    string<STRINGCAP + 4U> sut;
    ASSERT_THAT(sut.unsafe_assign(temp.c_str()), Eq(true));

    char testChar = 'M';
    EXPECT_THAT(sut == testChar, Eq(false));
    EXPECT_THAT(testChar == sut, Eq(false));
    EXPECT_THAT(sut != testChar, Eq(true));
    EXPECT_THAT(testChar != sut, Eq(true));
}

/// @note bool operator<(const char& rhs) const noexcept
/// bool operator<=(const char& rhs) const noexcept
/// bool operator>(const char& rhs) const noexcept
/// bool operator>=(const char& rhs) const noexcept
TYPED_TEST(stringTyped_test, CompareOperatorsWithDifferentChar)
{
    ::testing::Test::RecordProperty("TEST_ID", "c818b150-b926-4f3c-8405-6327303f12f6");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();

    char testChar = 'L';

    // compare testChar with greater string
    string<STRINGCAP> sutGreaterTestChar("M");

    EXPECT_THAT(sutGreaterTestChar < testChar, Eq(false));
    EXPECT_THAT(sutGreaterTestChar <= testChar, Eq(false));
    EXPECT_THAT(sutGreaterTestChar > testChar, Eq(true));
    EXPECT_THAT(sutGreaterTestChar >= testChar, Eq(true));
    EXPECT_THAT(testChar < sutGreaterTestChar, Eq(true));
    EXPECT_THAT(testChar <= sutGreaterTestChar, Eq(true));
    EXPECT_THAT(testChar > sutGreaterTestChar, Eq(false));
    EXPECT_THAT(testChar >= sutGreaterTestChar, Eq(false));


    // compare testChar with less string
    string<STRINGCAP> sutLessTestChar("F");

    EXPECT_THAT(sutLessTestChar < testChar, Eq(true));
    EXPECT_THAT(sutLessTestChar <= testChar, Eq(true));
    EXPECT_THAT(sutLessTestChar > testChar, Eq(false));
    EXPECT_THAT(sutLessTestChar >= testChar, Eq(false));
    EXPECT_THAT(testChar < sutLessTestChar, Eq(false));
    EXPECT_THAT(testChar <= sutLessTestChar, Eq(false));
    EXPECT_THAT(testChar > sutLessTestChar, Eq(true));
    EXPECT_THAT(testChar >= sutLessTestChar, Eq(true));
}

TYPED_TEST(stringTyped_test, CompareOperatorsWithEqualChar)
{
    ::testing::Test::RecordProperty("TEST_ID", "6d6344a8-d8d6-426d-b288-b429eb9a6eca");
    this->testSubject = "M";

    char testChar = 'M';
    EXPECT_THAT(this->testSubject < testChar, Eq(false));
    EXPECT_THAT(this->testSubject <= testChar, Eq(true));
    EXPECT_THAT(this->testSubject > testChar, Eq(false));
    EXPECT_THAT(this->testSubject >= testChar, Eq(true));

    EXPECT_THAT(testChar < this->testSubject, Eq(false));
    EXPECT_THAT(testChar <= this->testSubject, Eq(true));
    EXPECT_THAT(testChar > this->testSubject, Eq(false));
    EXPECT_THAT(testChar >= this->testSubject, Eq(true));
}

TYPED_TEST(stringTyped_test, CompareOperatorsWithDifferentCharWithDifferentSize)
{
    ::testing::Test::RecordProperty("TEST_ID", "c9f2e472-3fbf-4e08-a0ee-6bb725e7c0d7");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();

    char testChar = 'L';

    // compare testChar with greater string
    std::string temp1(STRINGCAP + 5U, 'M');
    string<STRINGCAP + 5U> sutGreaterTestChar;
    ASSERT_THAT(sutGreaterTestChar.unsafe_assign(temp1.c_str()), Eq(true));

    EXPECT_THAT(sutGreaterTestChar < testChar, Eq(false));
    EXPECT_THAT(sutGreaterTestChar <= testChar, Eq(false));
    EXPECT_THAT(sutGreaterTestChar > testChar, Eq(true));
    EXPECT_THAT(sutGreaterTestChar >= testChar, Eq(true));
    EXPECT_THAT(testChar < sutGreaterTestChar, Eq(true));
    EXPECT_THAT(testChar <= sutGreaterTestChar, Eq(true));
    EXPECT_THAT(testChar > sutGreaterTestChar, Eq(false));
    EXPECT_THAT(testChar >= sutGreaterTestChar, Eq(false));

    // compare testChar with less string
    std::string temp2(STRINGCAP + 5U, 'F');
    string<STRINGCAP + 5U> sutLessTestChar;
    ASSERT_THAT(sutLessTestChar.unsafe_assign(temp2.c_str()), Eq(true));

    EXPECT_THAT(sutLessTestChar < testChar, Eq(true));
    EXPECT_THAT(sutLessTestChar <= testChar, Eq(true));
    EXPECT_THAT(sutLessTestChar > testChar, Eq(false));
    EXPECT_THAT(sutLessTestChar >= testChar, Eq(false));
    EXPECT_THAT(testChar < sutLessTestChar, Eq(false));
    EXPECT_THAT(testChar <= sutLessTestChar, Eq(false));
    EXPECT_THAT(testChar > sutLessTestChar, Eq(true));
    EXPECT_THAT(testChar >= sutLessTestChar, Eq(true));

    // compare testChar with equal string
    std::string temp3(STRINGCAP + 5U, 'L');
    string<STRINGCAP + 5U> sutEqualTestChar;
    ASSERT_THAT(sutEqualTestChar.unsafe_assign(temp3.c_str()), Eq(true));

    EXPECT_THAT(sutEqualTestChar < testChar, Eq(false));
    EXPECT_THAT(sutEqualTestChar <= testChar, Eq(false));
    EXPECT_THAT(sutEqualTestChar > testChar, Eq(true));
    EXPECT_THAT(sutEqualTestChar >= testChar, Eq(true));
    EXPECT_THAT(testChar < sutEqualTestChar, Eq(true));
    EXPECT_THAT(testChar <= sutEqualTestChar, Eq(true));
    EXPECT_THAT(testChar > sutEqualTestChar, Eq(false));
    EXPECT_THAT(testChar >= sutEqualTestChar, Eq(false));
}
} // namespace
