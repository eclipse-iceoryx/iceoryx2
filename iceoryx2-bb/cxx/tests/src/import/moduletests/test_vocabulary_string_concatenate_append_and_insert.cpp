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

/// @note template <typename T1, typename T2, typename... Targs>
/// string<internal::SumCapa<T1, T2, Targs...>::value> concatenate(const T1& t1, const T2& t2, const Targs&... targs)
TYPED_TEST(stringTyped_test, ConcatenateTwoEmptyStringsReturnsEmptyStringWithTotalCapa)
{
    ::testing::Test::RecordProperty("TEST_ID", "ea0dea87-1bf7-44f9-9a4a-78ea2f0d834b");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    string<STRINGCAP + 1U> testString1;
    auto testString2 = concatenate(this->testSubject, testString1);

    EXPECT_THAT(this->testSubject.capacity(), Eq(STRINGCAP));
    EXPECT_THAT(this->testSubject.size(), Eq(0U));
    EXPECT_THAT(this->testSubject.c_str(), StrEq(""));
    EXPECT_THAT(testString1.capacity(), Eq(STRINGCAP + 1U));
    EXPECT_THAT(testString1.size(), Eq(0U));
    EXPECT_THAT(testString1.c_str(), StrEq(""));
    EXPECT_THAT(testString2.capacity(), Eq(2U * STRINGCAP + 1U));
    EXPECT_THAT(testString2.size(), Eq(0U));
    EXPECT_THAT(testString2.c_str(), StrEq(""));
}

TYPED_TEST(stringTyped_test, ConcatenateTwoStringsWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "783fec4d-6ab8-455c-b8a4-5eacff5865ab");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    std::string testStdString(STRINGCAP, 'M');
    EXPECT_THAT(this->testSubject.unsafe_assign(testStdString.c_str()), Eq(true));
    string<STRINGCAP + 2U> testString1;
    auto testString2 = concatenate(testString1, this->testSubject);

    EXPECT_THAT(testString2.capacity(), Eq(2U * STRINGCAP + 2U));
    EXPECT_THAT(testString2.size(), Eq(STRINGCAP));
    EXPECT_THAT(testString2.c_str(), StrEq(testStdString));
}

TYPED_TEST(stringTyped_test, ConcatenateTwoNotEmptyStringsWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "edc2f7a4-758c-47c0-bf66-283df11f636f");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    std::string testStdString0(STRINGCAP, 'M');
    EXPECT_THAT(this->testSubject.unsafe_assign(testStdString0.c_str()), Eq(true));
    std::string testStdString1(STRINGCAP + 3U, 'L');
    string<STRINGCAP + 3U> testString1(TruncateToCapacity, testStdString1.c_str(), testStdString1.size());
    auto testString2 = concatenate(this->testSubject, testString1);

    EXPECT_THAT(testString2.capacity(), Eq(2U * STRINGCAP + 3U));
    EXPECT_THAT(testString2.size(), Eq(this->testSubject.size() + testString1.size()));
    EXPECT_THAT(testString2.c_str(), StrEq(testStdString0 + testStdString1));
}

TYPED_TEST(stringTyped_test, ConcatenateThreeStringsWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "4f825a17-ae54-4478-8cad-e6a73f9d7801");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    string<STRINGCAP> testString1("A");
    string<STRINGCAP + 2U> testString2("YOD");
    auto testString3 = concatenate(testString2, this->testSubject, testString1);

    std::string cmpString = std::string(testString2.c_str(), static_cast<size_t>(testString2.size()))
                            + std::string(this->testSubject.c_str(), static_cast<size_t>(this->testSubject.size()))
                            + std::string(testString1.c_str(), static_cast<size_t>(testString1.size()));
    EXPECT_THAT(testString3.capacity(), Eq(3U * STRINGCAP + 2U));
    EXPECT_THAT(testString3.size(), Eq(cmpString.size()));
    EXPECT_THAT(testString3.c_str(), StrEq(cmpString));
}

TYPED_TEST(stringTyped_test, ConcatenateEmptyStringAndStringLiteralWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "e3eb16d0-1aec-40dc-8221-025e844d6dc8");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    auto testString = concatenate(this->testSubject, "M");
    EXPECT_THAT(testString.capacity(), Eq(STRINGCAP + 1U));
    EXPECT_THAT(testString.size(), Eq(1U));
    EXPECT_THAT(testString.c_str(), StrEq("M"));
}

TYPED_TEST(stringTyped_test, ConcatenateStringLiteralAndStringWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "727d85a2-73c4-4524-89d9-e4b8ebcdac0f");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    this->testSubject = "S";
    auto testString = concatenate("Ferdinand", this->testSubject);
    EXPECT_THAT(testString.capacity(), Eq(STRINGCAP + 9U));
    EXPECT_THAT(testString.size(), Eq(10));
    EXPECT_THAT(testString.c_str(), StrEq("FerdinandS"));
}

TEST(StringLiteralConcatenation, ConcatenateOnlyStringLiteralsWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "f591fb69-a8fa-4d8e-b42d-29573b4a20f7");
    auto testString = concatenate("Ferdi", "nandSpitzschnu", "ef", "fler");
    EXPECT_THAT(testString.capacity(), Eq(25U));
    EXPECT_THAT(testString.size(), Eq(25U));
    EXPECT_THAT(testString.c_str(), StrEq("FerdinandSpitzschnueffler"));
}

TYPED_TEST(stringTyped_test, ConcatenateEmptyStringAndNullCharReturnsStringWithSizeOne)
{
    ::testing::Test::RecordProperty("TEST_ID", "6a4374fb-b21f-4465-95dc-22b898079c81");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();

    auto result1 = concatenate(this->testSubject, '\0');

    EXPECT_THAT(this->testSubject.capacity(), Eq(STRINGCAP));
    EXPECT_THAT(this->testSubject.empty(), Eq(true));
    EXPECT_THAT(result1.capacity(), Eq(STRINGCAP + 1U));
    EXPECT_THAT(result1.size(), Eq(1U));
    EXPECT_THAT(result1.empty(), Eq(false));
    EXPECT_THAT(result1[0U], Eq('\0'));

    auto result2 = concatenate('\0', this->testSubject);

    EXPECT_THAT(this->testSubject.capacity(), Eq(STRINGCAP));
    EXPECT_THAT(this->testSubject.empty(), Eq(true));
    EXPECT_THAT(result2.capacity(), Eq(STRINGCAP + 1U));
    EXPECT_THAT(result2.size(), Eq(1U));
    EXPECT_THAT(result2.empty(), Eq(false));
    EXPECT_THAT(result2[0U], Eq('\0'));
}

TYPED_TEST(stringTyped_test, ConcatenateEmptyStringAndCharWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "f99022c0-1ea9-4c63-bb18-d0ec53020e66");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();

    auto result1 = concatenate('M', this->testSubject);

    EXPECT_THAT(result1.capacity(), Eq(STRINGCAP + 1U));
    EXPECT_THAT(result1.size(), Eq(1U));
    EXPECT_THAT(result1.c_str(), StrEq("M"));

    auto result2 = concatenate(this->testSubject, 'M');

    EXPECT_THAT(result2.capacity(), Eq(STRINGCAP + 1U));
    EXPECT_THAT(result2.size(), Eq(1U));
    EXPECT_THAT(result2.c_str(), StrEq("M"));
}

TYPED_TEST(stringTyped_test, ConcatenateStringAndCharWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "d8cf335a-4fa2-4dc5-a98d-8b34f3b47464");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    const string<STRINGCAP + 26U> expectedString("FerdinandSpitzschnueffler");
    string<STRINGCAP + 10U> testString1("Ferdinand");
    const char testChar = 'S';
    string<15U> testString2("pitzschnueffler");
    auto result = concatenate(testString1, testChar, testString2);

    EXPECT_THAT(result.capacity(), Eq(expectedString.capacity()));
    EXPECT_THAT(result.size(), Eq(expectedString.size()));
    EXPECT_THAT(result.c_str(), StrEq(expectedString.c_str()));
}

TEST(CharConcatenation, ConcatenateOnlyCharsWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "32900bfd-69c9-4116-b702-9d2165bd54b0");
    auto testString = concatenate('W', 'o', 'o', 'h', 'o', 'o');
    EXPECT_THAT(testString.capacity(), Eq(6U));
    EXPECT_THAT(testString.size(), Eq(6U));
    EXPECT_THAT(testString.c_str(), StrEq("Woohoo"));
}

/// @note template <typename T1, typename T2>
/// string<internal::GetCapa<T1>::capa + internal::GetCapa<T2>::capa> operator+(const T1& t1, const T2& t2);
TYPED_TEST(stringTyped_test, ConcatenateEmptyStringsReturnsEmptyString)
{
    ::testing::Test::RecordProperty("TEST_ID", "39fedfb3-f3fd-477a-8499-f6f4c63c46dd");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    auto testString = this->testSubject + this->testSubject;
    EXPECT_THAT(testString.capacity(), Eq(2U * STRINGCAP));
    EXPECT_THAT(testString.size(), Eq(0U));
    EXPECT_THAT(testString.c_str(), StrEq(""));
}

TYPED_TEST(stringTyped_test, ConcatenateStringsWithOperatorPlusWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "0a792faf-f944-49f2-980f-dc58d3aedd40");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    std::string testStdString(STRINGCAP, 'M');
    EXPECT_THAT(this->testSubject.unsafe_assign(testStdString.c_str()), Eq(true));
    string<STRINGCAP + 2U> testString1;
    string<2U * STRINGCAP + 2U> testString2;
    testString2 = testString1 + this->testSubject;
    EXPECT_THAT(testString2.capacity(), Eq(2U * STRINGCAP + 2U));
    EXPECT_THAT(testString2.size(), Eq(STRINGCAP));
    EXPECT_THAT(testString2.c_str(), StrEq(testStdString));
}

TYPED_TEST(stringTyped_test, ConcatenateNotEmptyStringsWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "161902aa-98a0-450f-874e-f300a99a144d");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    std::string testStdString0(STRINGCAP, 'M');
    EXPECT_THAT(this->testSubject.unsafe_assign(testStdString0.c_str()), Eq(true));
    std::string testStdString1(STRINGCAP + 3U, 'L');
    string<STRINGCAP + 3U> testString1(TruncateToCapacity, testStdString1.c_str(), testStdString1.size());
    string<6U * STRINGCAP> testString2 = this->testSubject + testString1 + this->testSubject;
    EXPECT_THAT(testString2.capacity(), Eq(6U * STRINGCAP));
    EXPECT_THAT(testString2.size(), Eq(2U * this->testSubject.size() + testString1.size()));
    EXPECT_THAT(testString2.c_str(), StrEq(testStdString0 + testStdString1 + testStdString0));
}

TYPED_TEST(stringTyped_test, ConcatenateEmptyStringAndStringLiteralWithOperatorPlusWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "f87151cd-ef6c-4f51-a986-7f238d608678");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    string<2U * STRINGCAP> testString1 = this->testSubject + "M";
    EXPECT_THAT(testString1.capacity(), Eq(2U * STRINGCAP));
    EXPECT_THAT(testString1.size(), Eq(1U));
    EXPECT_THAT(testString1.c_str(), StrEq("M"));

    // required to verify string literal functionality of iox::string
    // NOLINTNEXTLINE(hicpp-avoid-c-arrays, cppcoreguidelines-avoid-c-arrays)
    char testChar[3] = "ab";
    testChar[2] = 'c';
    string<3U * STRINGCAP> testString2 = this->testSubject + testChar;
    EXPECT_THAT(testString2.capacity(), Eq(3U * STRINGCAP));
    EXPECT_THAT(testString2.size(), Eq(3U));
    EXPECT_THAT(testString2.c_str(), StrEq("abc"));
}

TYPED_TEST(stringTyped_test, ConcatenateStringLiteralAndStringWithOperatorPlusWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "d6630140-592b-45c7-ab76-be8cd3fa3d98");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    this->testSubject = "e";
    string<STRINGCAP + 7U> testString = "AdmTass" + this->testSubject;
    EXPECT_THAT(testString.capacity(), Eq(STRINGCAP + 7U));
    EXPECT_THAT(testString.size(), Eq(8U));
    EXPECT_THAT(testString.c_str(), StrEq("AdmTasse"));
}

TYPED_TEST(stringTyped_test, ConcatenateEmptyStringAndCharWithOperatorPlusWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "09efee5c-985f-4852-aa3c-5c67fc61071b");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    string<2U * STRINGCAP> testString = this->testSubject + 'M';
    EXPECT_THAT(testString.capacity(), Eq(2U * STRINGCAP));
    EXPECT_THAT(testString.size(), Eq(1U));
    EXPECT_THAT(testString.c_str(), StrEq("M"));
}

TYPED_TEST(stringTyped_test, ConcatenateCharAndStringWithOperatorPlusWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "7d25ab82-48f9-4000-9764-e14522fd4ecc");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    this->testSubject = "S";
    string<STRINGCAP + 7U> testString = 'F' + this->testSubject;
    EXPECT_THAT(testString.capacity(), Eq(STRINGCAP + 7U));
    EXPECT_THAT(testString.size(), Eq(2U));
    EXPECT_THAT(testString.c_str(), StrEq("FS"));
}

TEST(String10, ConcatenateSeveralCharsAndStringsWithOperatorPlusWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "68e69585-fe83-497c-9249-59644501abd1");
    const string<3U> testString1("Hyp");
    const char testChar1 = 'n';
    const string<2U> testString2("ot");
    const char testChar2 = 'o';
    const string<2U> testString3("ad");
    auto result = testString1 + testChar1 + testString2 + testChar2 + testString3;

    EXPECT_THAT(result.capacity(), Eq(9U));
    EXPECT_THAT(result.size(), Eq(9U));
    EXPECT_THAT(result.c_str(), StrEq("Hypnotoad"));
}

/// @note template <typename T>
/// bool unsafe_append(const T& t) noexcept;
TYPED_TEST(stringTyped_test, UnsafeAppendEmptyStringWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "965e3b39-2cda-4816-9d64-ccf0d4e6f761");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    this->testSubject = "M";
    string<2U * STRINGCAP> testString;
    EXPECT_THAT(this->testSubject.unsafe_append(testString), Eq(true));
    EXPECT_THAT(this->testSubject.capacity(), Eq(STRINGCAP));
    EXPECT_THAT(this->testSubject.size(), Eq(1U));
    EXPECT_THAT(this->testSubject.c_str(), StrEq("M"));
}

TYPED_TEST(stringTyped_test, UnsafeAppendFittingStringWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "b609c480-c1dd-49d5-b230-09ccda1b0e78");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    this->testSubject = "2";
    string<5U * STRINGCAP> testString("R2-D");
    EXPECT_THAT(testString.unsafe_append(this->testSubject), Eq(true));
    EXPECT_THAT(testString.capacity(), Eq(5U * STRINGCAP));
    EXPECT_THAT(testString.size(), Eq(5));
    EXPECT_THAT(testString.c_str(), StrEq("R2-D2"));
}

TYPED_TEST(stringTyped_test, UnsafeAppendTooLargeStringFails)
{
    ::testing::Test::RecordProperty("TEST_ID", "1a572827-b778-4e7a-a79f-c62b48928a81");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    this->testSubject = "M";
    string<2U * STRINGCAP> testString;
    std::string testStdString(STRINGCAP, 'M');
    EXPECT_THAT(testString.unsafe_assign(testStdString.c_str()), Eq(true));

    EXPECT_THAT(this->testSubject.unsafe_append(testString), Eq(false));
    EXPECT_THAT(this->testSubject.capacity(), Eq(STRINGCAP));
    EXPECT_THAT(this->testSubject.size(), Eq(1U));
    EXPECT_THAT(this->testSubject.c_str(), StrEq("M"));
}

TYPED_TEST(stringTyped_test, UnsafeAppendEmptyStringLiteralWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "5725ff86-b3db-4a43-9278-fc358628a1fc");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    this->testSubject = "M";
    EXPECT_THAT(this->testSubject.unsafe_append(""), Eq(true));
    EXPECT_THAT(this->testSubject.capacity(), Eq(STRINGCAP));
    EXPECT_THAT(this->testSubject.size(), Eq(1U));
    EXPECT_THAT(this->testSubject.c_str(), StrEq("M"));
}

TEST(String10, UnsafeAppendFittingStringLiteralWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "ed188a09-9550-4377-95f6-ca51f8cfbf7e");
    constexpr uint64_t STRINGCAP = 10U;
    string<STRINGCAP> testString("R2-D");
    EXPECT_THAT(testString.unsafe_append("2"), Eq(true));
    EXPECT_THAT(testString.capacity(), Eq(STRINGCAP));
    EXPECT_THAT(testString.size(), Eq(5U));
    EXPECT_THAT(testString.c_str(), StrEq("R2-D2"));
}

TEST(String10, UnsafeAppendTooLargeStringLiteralFails)
{
    ::testing::Test::RecordProperty("TEST_ID", "f9305077-c3a8-4edb-be5f-ea8b1af19645");
    constexpr uint64_t STRINGCAP = 10U;
    string<STRINGCAP> testString("Kern");
    EXPECT_THAT(testString.unsafe_append("fusionsbaby"), Eq(false));
    EXPECT_THAT(testString.capacity(), Eq(STRINGCAP));
    EXPECT_THAT(testString.size(), Eq(4U));
    EXPECT_THAT(testString.c_str(), StrEq("Kern"));
}

TYPED_TEST(stringTyped_test, UnsafeAppendNullCharWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "b1a6743f-1146-4269-9c13-0baa1e14fbe4");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    string<STRINGCAP + 1U> testString("M");
    const char testChar = '\0';
    EXPECT_THAT(testString.unsafe_append(testChar), Eq(true));
    EXPECT_THAT(testString.capacity(), Eq(STRINGCAP + 1U));
    EXPECT_THAT(testString.size(), Eq(2U));
    EXPECT_THAT(testString.c_str(), StrEq("M"));
    EXPECT_THAT(testString[1U], Eq(testChar));
}

TYPED_TEST(stringTyped_test, UnsafeAppendCharWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "4243b52a-fe02-45ec-b201-dde2a0dca5b8");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    string<STRINGCAP + 5U> testString("R2-D");
    EXPECT_THAT(testString.unsafe_append('2'), Eq(true));
    EXPECT_THAT(testString.capacity(), Eq(STRINGCAP + 5U));
    EXPECT_THAT(testString.size(), Eq(5U));
    EXPECT_THAT(testString.c_str(), StrEq("R2-D2"));
}

TYPED_TEST(stringTyped_test, UnsafeAppendWithCharFailsWhenCapacityIsExceeded)
{
    ::testing::Test::RecordProperty("TEST_ID", "8e92266b-d9dc-4751-a4e6-3be772a1855d");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    std::string temp(STRINGCAP, 'M');
    EXPECT_THAT(this->testSubject.unsafe_assign(temp.c_str()), Eq(true));

    EXPECT_THAT(this->testSubject.unsafe_append('L'), Eq(false));
    EXPECT_THAT(this->testSubject.capacity(), Eq(STRINGCAP));
    EXPECT_THAT(this->testSubject.size(), Eq(STRINGCAP));
    EXPECT_THAT(this->testSubject.c_str(), StrEq(temp));

    EXPECT_THAT(this->testSubject.unsafe_append('\0'), Eq(false));
    EXPECT_THAT(this->testSubject.capacity(), Eq(STRINGCAP));
    EXPECT_THAT(this->testSubject.size(), Eq(STRINGCAP));
    EXPECT_THAT(this->testSubject.c_str(), StrEq(temp));
}

TYPED_TEST(stringTyped_test, UnsafeAppendWithCharToEmptyStringWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "0d6965c7-b0f1-45d4-989a-c7349a53f8f7");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    EXPECT_THAT(this->testSubject.unsafe_append('v'), Eq(true));
    EXPECT_THAT(this->testSubject.capacity(), Eq(STRINGCAP));
    EXPECT_THAT(this->testSubject.size(), Eq(1U));
    EXPECT_THAT(this->testSubject.c_str(), StrEq("v"));
}

/// @note template <typename T>
/// string& append(TruncateToCapacity_t, const T& t) noexcept
TYPED_TEST(stringTyped_test, AppendEmptyStringWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "9f286751-a900-4fd4-be49-105c2de25265");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    this->testSubject = "M";
    string<STRINGCAP + 1U> testString;
    this->testSubject.append(TruncateToCapacity, testString);
    EXPECT_THAT(this->testSubject.capacity(), Eq(STRINGCAP));
    EXPECT_THAT(this->testSubject.size(), Eq(1U));
    EXPECT_THAT(this->testSubject.c_str(), StrEq("M"));
}

TYPED_TEST(stringTyped_test, AppendStringToEmptyStringResultsInConcatenatedString)
{
    ::testing::Test::RecordProperty("TEST_ID", "cb8651ac-efcf-430a-94e8-8f6e20dcd265");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    string<STRINGCAP + 5U> testString("M");
    this->testSubject.append(TruncateToCapacity, testString);
    EXPECT_THAT(this->testSubject.capacity(), Eq(STRINGCAP));
    EXPECT_THAT(this->testSubject.size(), Eq(1U));
    EXPECT_THAT(this->testSubject.c_str(), StrEq("M"));
}

TYPED_TEST(stringTyped_test, AppendStringResultsInConcatenatedString)
{
    ::testing::Test::RecordProperty("TEST_ID", "9c7f7d12-4b5b-483b-acb4-0afbe6d59ec2");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    this->testSubject = "d";
    string<STRINGCAP + 5U> testString("Picar");
    testString.append(TruncateToCapacity, this->testSubject);
    EXPECT_THAT(testString.capacity(), Eq(STRINGCAP + 5U));
    EXPECT_THAT(testString.size(), Eq(6U));
    EXPECT_THAT(testString.c_str(), StrEq("Picard"));
}

TYPED_TEST(stringTyped_test, AppendTooLargeStringResultsInTruncatedString)
{
    ::testing::Test::RecordProperty("TEST_ID", "f4e3243d-744d-4f00-bdf5-118884fd42c0");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    this->testSubject = "M";
    string<STRINGCAP + 1U> testString;
    std::string testStdString(STRINGCAP + 1U, 'M');
    EXPECT_THAT(testString.unsafe_assign(testStdString.c_str()), Eq(true));
    this->testSubject.append(TruncateToCapacity, testString);
    EXPECT_THAT(this->testSubject.capacity(), Eq(STRINGCAP));
    EXPECT_THAT(this->testSubject.size(), Eq(STRINGCAP));
    EXPECT_THAT(this->testSubject.c_str(), StrEq(testStdString.substr(0, STRINGCAP)));
}

TYPED_TEST(stringTyped_test, AppendEmptyStringLiteralWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "f24afd76-6d2a-4d2f-8cf5-4883bbf5bb6b");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    this->testSubject = "M";
    this->testSubject.append(TruncateToCapacity, "");
    EXPECT_THAT(this->testSubject.capacity(), Eq(STRINGCAP));
    EXPECT_THAT(this->testSubject.size(), Eq(1U));
    EXPECT_THAT(this->testSubject.c_str(), StrEq("M"));
}

TYPED_TEST(stringTyped_test, AppendStringLiteralToEmptyStringResultsInConcatenatedString)
{
    ::testing::Test::RecordProperty("TEST_ID", "8ebaa145-0a91-4853-9320-2cab77a7f99f");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    this->testSubject.append(TruncateToCapacity, "M");
    EXPECT_THAT(this->testSubject.capacity(), Eq(STRINGCAP));
    EXPECT_THAT(this->testSubject.size(), Eq(1U));
    EXPECT_THAT(this->testSubject.c_str(), StrEq("M"));
}

TEST(String10, AppendStringLiteralResultsInConcatenatedString)
{
    ::testing::Test::RecordProperty("TEST_ID", "f620c504-8d0a-4088-9074-4a1be80ff077");
    constexpr uint64_t STRINGCAP = 10U;
    string<STRINGCAP> testString("Picar");
    testString.append(TruncateToCapacity, "d");
    EXPECT_THAT(testString.capacity(), Eq(STRINGCAP));
    EXPECT_THAT(testString.size(), Eq(6U));
    EXPECT_THAT(testString.c_str(), StrEq("Picard"));
}

TEST(String10, AppendTooLargeStringLiteralResultsInTruncatedString)
{
    ::testing::Test::RecordProperty("TEST_ID", "524a544a-0275-4549-a6dd-7c40e31af91c");
    constexpr uint64_t STRINGCAP = 10U;
    string<STRINGCAP> testString("Live long");
    testString.append(TruncateToCapacity, " and prosper");
    EXPECT_THAT(testString.capacity(), Eq(STRINGCAP));
    EXPECT_THAT(testString.size(), Eq(STRINGCAP));
    EXPECT_THAT(testString.c_str(), StrEq("Live long "));
}

TYPED_TEST(stringTyped_test, AppendStringContainingNullWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "ed16b243-ddb3-46be-8bd8-2ed93fc7184d");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    constexpr auto RESULT_CAPACITY = STRINGCAP + 10U;
    const std::string expectedString{"ice\0ryx", 7};

    string<RESULT_CAPACITY> sut("i");
    const string<RESULT_CAPACITY> testCxxString(TruncateToCapacity, expectedString.substr(1).c_str(), 6U);

    // append iox::string
    sut.append(TruncateToCapacity, testCxxString);
    EXPECT_THAT(sut.capacity(), Eq(RESULT_CAPACITY));
    EXPECT_THAT(sut.size(), Eq(7U));
    EXPECT_THAT(std::memcmp(sut.c_str(), expectedString.c_str(), static_cast<size_t>(sut.size())), Eq(0));
}

/// @note string& append(TruncateToCapacity_t, char c) noexcept
TYPED_TEST(stringTyped_test, AppendNullCharWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "c20f1797-2797-4f4a-9acf-7bd1b9518c3d");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    string<STRINGCAP + 1U> sut = "M";
    sut.append(TruncateToCapacity, '\0');
    EXPECT_THAT(sut.capacity(), Eq(STRINGCAP + 1U));
    EXPECT_THAT(sut.size(), Eq(2U));
    EXPECT_THAT(sut.c_str(), StrEq("M"));
    EXPECT_THAT(sut[1U], Eq('\0'));
}

TYPED_TEST(stringTyped_test, AppendCharToEmptyStringResultsInConcatenatedString)
{
    ::testing::Test::RecordProperty("TEST_ID", "7cf74a7a-a468-4efa-b54a-61d265de8808");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    this->testSubject.append(TruncateToCapacity, 'M');
    EXPECT_THAT(this->testSubject.capacity(), Eq(STRINGCAP));
    EXPECT_THAT(this->testSubject.size(), Eq(1U));
    EXPECT_THAT(this->testSubject.c_str(), StrEq("M"));
}

TYPED_TEST(stringTyped_test, AppendCharResultsInConcatenatedString)
{
    ::testing::Test::RecordProperty("TEST_ID", "853b6d2f-4e48-43f4-b167-53f076747ab2");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    string<STRINGCAP + 5U> testString("Picar");
    testString.append(TruncateToCapacity, 'd');
    EXPECT_THAT(testString.capacity(), Eq(STRINGCAP + 5U));
    EXPECT_THAT(testString.size(), Eq(6U));
    EXPECT_THAT(testString.c_str(), StrEq("Picard"));
}

TYPED_TEST(stringTyped_test, AppendCharDoesNotChangeStringWhenCapacityIsExceeded)
{
    ::testing::Test::RecordProperty("TEST_ID", "3005b5af-28b5-452a-98dc-603f6973230f");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    std::string temp(STRINGCAP, 'M');
    EXPECT_THAT(this->testSubject.unsafe_assign(temp.c_str()), Eq(true));

    this->testSubject.append(TruncateToCapacity, 'L');
    EXPECT_THAT(this->testSubject.capacity(), Eq(STRINGCAP));
    EXPECT_THAT(this->testSubject.size(), Eq(STRINGCAP));
    EXPECT_THAT(this->testSubject.c_str(), StrEq(temp.substr(0, STRINGCAP)));
}

/// @note template <typename T>
/// constexpr bool insert(const uint64_t pos, const T& str, const uint64_t count) noexcept
TEST(String10, InsertStringLiteralAtTheBeginningOfTheStringSucceeds)
{
    ::testing::Test::RecordProperty("TEST_ID", "8f3c5bf8-2367-46c0-b803-570a2a640c93");
    constexpr uint64_t STRINGCAP = 10U;
    const string<STRINGCAP> expectedString("Hypnotoad");
    string<STRINGCAP> sut("toad");
    ASSERT_TRUE(sut.insert(0, "Hypno", 5));
    EXPECT_THAT(sut.size(), Eq(expectedString.size()));
    EXPECT_THAT(sut, Eq(expectedString));
}

TEST(String10, InsertStringLiteralInTheMiddleOfTheStringSucceeds)
{
    ::testing::Test::RecordProperty("TEST_ID", "861eec68-3e78-42cf-ac75-30c23a3e60d2");
    constexpr uint64_t STRINGCAP = 10U;
    const string<STRINGCAP> expectedString("Hypnotoad");
    string<STRINGCAP> sut("Hypoad");
    ASSERT_TRUE(sut.insert(3, "not", 3));
    EXPECT_THAT(sut.size(), Eq(expectedString.size()));
    EXPECT_THAT(sut, Eq(expectedString));
}

TEST(String10, InsertStringLiteralAtTheEndOfTheStringSucceeds)
{
    ::testing::Test::RecordProperty("TEST_ID", "91f733e0-5ded-4f85-9727-241b649bd7dc");
    constexpr uint64_t STRINGCAP = 10U;
    const string<STRINGCAP> expectedString("Hypnotoad");
    string<STRINGCAP> sut("Hypno");
    ASSERT_TRUE(sut.insert(sut.size(), "toad", 4));
    EXPECT_THAT(sut.size(), Eq(expectedString.size()));
    EXPECT_THAT(sut, Eq(expectedString));
}

TYPED_TEST(stringTyped_test, InsertStringLiteralToEmptyStringWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "eba320fc-9909-4043-a944-7a5c9f2a4272");
    const string<1> expectedString("M");
    ASSERT_TRUE(this->testSubject.insert(0, "M", 1));
    EXPECT_THAT(this->testSubject.size(), Eq(expectedString.size()));
    EXPECT_THAT(this->testSubject, Eq(expectedString));
}

TYPED_TEST(stringTyped_test, InsertEmptyStringLiteralDoesNotChangeTheString)
{
    ::testing::Test::RecordProperty("TEST_ID", "37d723b7-5b35-4f1d-ac33-23dd483121a1");
    this->testSubject = "M";
    ASSERT_TRUE(this->testSubject.insert(0, "", 0));
    EXPECT_THAT(this->testSubject.size(), Eq(1));
    EXPECT_THAT(this->testSubject.c_str(), StrEq("M"));
}

TYPED_TEST(stringTyped_test, InsertStringLiteralWithCountGreaterThanSizeOfStringLiteralFails)
{
    ::testing::Test::RecordProperty("TEST_ID", "3d2951f8-d868-4998-bb33-58424aba7a65");
    ASSERT_FALSE(this->testSubject.insert(0, "M", 2));
    EXPECT_THAT(this->testSubject.size(), Eq(0));
    EXPECT_THAT(this->testSubject.c_str(), StrEq(""));
}

TEST(String10, InsertTooLargeStringLiteralFails)
{
    ::testing::Test::RecordProperty("TEST_ID", "0f11f387-f437-4a39-8207-259df4807c70");
    constexpr uint64_t STRINGCAP = 10U;
    const string<STRINGCAP> expectedString("Ferdinand");
    string<STRINGCAP> sut(expectedString);
    ASSERT_FALSE(sut.insert(sut.size(), "Spitzschnueffler", 16));
    EXPECT_THAT(sut.size(), Eq(expectedString.size()));
    EXPECT_THAT(sut, Eq(expectedString));
}

TEST(String10, InsertTooLargeStringLiteralWithSmallCountSucceeds)
{
    ::testing::Test::RecordProperty("TEST_ID", "2763b3d8-3023-4841-80d1-798acd3b71d7");
    constexpr uint64_t STRINGCAP = 10U;
    const string<STRINGCAP> expectedString("FerdinandS");
    string<STRINGCAP> sut("Ferdinand");
    ASSERT_TRUE(sut.insert(sut.size(), "Spitzschnueffler", 1));
    EXPECT_THAT(sut.size(), Eq(expectedString.size()));
    EXPECT_THAT(sut, Eq(expectedString));
}

TYPED_TEST(stringTyped_test, InsertStringLiteralWithCount0DoesntChangeTheString)
{
    ::testing::Test::RecordProperty("TEST_ID", "90e80302-1c14-48bb-b9e9-0f8fca017fea");
    ASSERT_TRUE(this->testSubject.insert(0, "Ferdinand", 0));
    EXPECT_THAT(this->testSubject.size(), Eq(0));
    EXPECT_THAT(this->testSubject.c_str(), StrEq(""));
}

TEST(String10, InsertStringLiteralAtPositionGreaterStringSizeFails)
{
    ::testing::Test::RecordProperty("TEST_ID", "a3a30ea5-f228-48f4-9497-08502e4f2c9a");
    constexpr uint64_t STRINGCAP = 10U;
    const string<STRINGCAP> expectedString("Muesli");
    string<STRINGCAP> sut(expectedString);
    ASSERT_FALSE(sut.insert(sut.size() + 1U, "s", 1));
    EXPECT_THAT(sut.size(), Eq(expectedString.size()));
    EXPECT_THAT(sut, Eq(expectedString));
}

TEST(String10, InsertCxxStringAtTheBeginningOfTheStringSucceeds)
{
    ::testing::Test::RecordProperty("TEST_ID", "df62df30-0838-4282-91f7-b39b1c861861");
    constexpr uint64_t STRINGCAP = 10U;
    const string<STRINGCAP> expectedString("Hypnotoad");
    string<STRINGCAP> sut("toad");
    string<STRINGCAP> string_to_insert("Hypno");
    ASSERT_TRUE(sut.insert(0, string_to_insert, string_to_insert.size()));
    EXPECT_THAT(sut.size(), Eq(expectedString.size()));
    EXPECT_THAT(sut, Eq(expectedString));
}

TEST(String10, InsertCxxStringInTheMiddleOfTheStringSucceeds)
{
    ::testing::Test::RecordProperty("TEST_ID", "b5a94b5a-c734-4654-a6ba-08819b4b7eac");
    constexpr uint64_t STRINGCAP = 10U;
    const string<STRINGCAP> expectedString("Hypnotoad");
    string<STRINGCAP> sut("Hypoad");
    string<STRINGCAP> string_to_insert("not");
    ASSERT_TRUE(sut.insert(3, string_to_insert, string_to_insert.size()));
    EXPECT_THAT(sut.size(), Eq(expectedString.size()));
    EXPECT_THAT(sut, Eq(expectedString));
}

TEST(String10, InsertCxxStringAtTheEndOfTheStringSucceeds)
{
    ::testing::Test::RecordProperty("TEST_ID", "f77dfb73-77ce-471f-b90b-8fcf0ab1a5de");
    constexpr uint64_t STRINGCAP = 10U;
    const string<STRINGCAP> expectedString("Hypnotoad");
    string<STRINGCAP> sut("Hypno");
    string<STRINGCAP> string_to_insert("toad");
    ASSERT_TRUE(sut.insert(sut.size(), string_to_insert, string_to_insert.size()));
    EXPECT_THAT(sut.size(), Eq(expectedString.size()));
    EXPECT_THAT(sut, Eq(expectedString));
}

TYPED_TEST(stringTyped_test, InsertCxxStringToEmptyStringWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "95ce194c-62a7-420c-9f37-9a2d7c5b17b2");
    string<1> string_to_insert("M");
    ASSERT_TRUE(this->testSubject.insert(0, string_to_insert, string_to_insert.size()));
    EXPECT_THAT(this->testSubject.size(), Eq(string_to_insert.size()));
    EXPECT_THAT(this->testSubject, Eq(string_to_insert));
}

TYPED_TEST(stringTyped_test, InsertEmptyCxxStringDoesNotChangeTheString)
{
    ::testing::Test::RecordProperty("TEST_ID", "b566efaf-4a32-41a1-970f-4d51675e594f");
    this->testSubject = "M";
    string<1> string_to_insert("");
    ASSERT_TRUE(this->testSubject.insert(0, string_to_insert, string_to_insert.size()));
    EXPECT_THAT(this->testSubject.size(), Eq(1));
    EXPECT_THAT(this->testSubject.c_str(), StrEq("M"));
}

TYPED_TEST(stringTyped_test, InsertCxxStringWithCountGreaterThanSizeOfStringLiteralFails)
{
    ::testing::Test::RecordProperty("TEST_ID", "9bd29af2-12c8-4dc1-8ee8-82610f1bb9a6");
    string<1> string_to_insert("M");
    ASSERT_FALSE(this->testSubject.insert(0, string_to_insert, string_to_insert.size() + 1));
    EXPECT_THAT(this->testSubject.size(), Eq(0));
    EXPECT_THAT(this->testSubject.c_str(), StrEq(""));
}

TEST(String10, InsertTooLargeCxxStringFails)
{
    ::testing::Test::RecordProperty("TEST_ID", "e9d88dd2-9da5-4f5f-a798-10b3931e6516");
    constexpr uint64_t STRINGCAP = 10U;
    const string<STRINGCAP> expectedString("Ferdinand");
    string<STRINGCAP> sut(expectedString);
    string<STRINGCAP + 6U> string_to_insert("Spitzschnueffler");
    ASSERT_FALSE(sut.insert(sut.size(), string_to_insert, string_to_insert.size()));
    EXPECT_THAT(sut.size(), Eq(expectedString.size()));
    EXPECT_THAT(sut, Eq(expectedString));
}

TEST(String10, InsertTooLargeCxxStringWithSmallCountSucceeds)
{
    ::testing::Test::RecordProperty("TEST_ID", "e99348ac-87a4-4678-b33a-2ddac3e21cb4");
    constexpr uint64_t STRINGCAP = 10U;
    constexpr uint64_t INSERT_STRINGCAP = STRINGCAP + 6U;
    const string<STRINGCAP> expectedString("FerdinandS");
    string<STRINGCAP> sut("Ferdinand");
    string<INSERT_STRINGCAP> string_to_insert("Spitzschnueffler");
    ASSERT_TRUE(sut.insert(sut.size(), string_to_insert, 1));
    EXPECT_THAT(sut.size(), Eq(expectedString.size()));
    EXPECT_THAT(sut, Eq(expectedString));
}

TYPED_TEST(stringTyped_test, InsertCxxStringWithCount0DoesntChangeTheString)
{
    ::testing::Test::RecordProperty("TEST_ID", "340c595e-b0b1-43a8-a08e-ebb62dc0d306");
    const string<10> string_to_insert("Ferdinand");
    ASSERT_TRUE(this->testSubject.insert(0, string_to_insert, 0));
    EXPECT_THAT(this->testSubject.size(), Eq(0));
    EXPECT_THAT(this->testSubject.c_str(), StrEq(""));
}

TEST(String10, InsertCxxStringAtPositionGreaterStringSizeFails)
{
    ::testing::Test::RecordProperty("TEST_ID", "a2517ea0-2842-4f56-a89a-b7fd01c6c6ba");
    constexpr uint64_t STRINGCAP = 10U;
    const string<STRINGCAP> expectedString("Muesli");
    string<STRINGCAP> sut(expectedString);
    string<1> string_to_insert("s");
    ASSERT_FALSE(sut.insert(sut.size() + 1U, string_to_insert, 1));
    EXPECT_THAT(sut.size(), Eq(expectedString.size()));
    EXPECT_THAT(sut, Eq(expectedString));
}
} // namespace
