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

#include "iox/std_string_support.hpp"

#include "test.hpp"

namespace
{
using namespace ::testing;
using namespace iox;


template <typename T>
class StdString_test : public Test
{
  public:
    T testSubject;

    using stringType = T;
};

using Implementations = Types<string<1>, string<15>, string<100>, string<1000>>;

TYPED_TEST_SUITE(StdString_test, Implementations, );

TYPED_TEST(StdString_test, STDStringToLossyStringConvConstrWithSize0ResultsInSize0)
{
    ::testing::Test::RecordProperty("TEST_ID", "83e1b7b2-8487-4c71-ac86-f4d5d98c1918");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    std::string testString;
    string<STRINGCAP> fuu = into<lossy<MyString>>(testString);
    EXPECT_THAT(fuu.capacity(), Eq(STRINGCAP));
    EXPECT_THAT(fuu.size(), Eq(0U));
    EXPECT_THAT(fuu.c_str(), StrEq(""));
}

TYPED_TEST(StdString_test, STDStringToLossyStringConvConstrWithSizeSmallerCapaResultsInSizeSmallerCapa)
{
    ::testing::Test::RecordProperty("TEST_ID", "1bd6cd60-0487-4ba2-9e51-3a9297078454");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    std::string testString(STRINGCAP - 1U, 'M');
    string<STRINGCAP> fuu = into<lossy<MyString>>(testString);
    EXPECT_THAT(fuu.capacity(), Eq(STRINGCAP));
    EXPECT_THAT(fuu.size(), Eq(STRINGCAP - 1U));
    EXPECT_THAT(fuu.c_str(), Eq(testString));
}

TYPED_TEST(StdString_test, STDStringToLossyStringConvConstrWithSizeCapaResultsInSizeCapa)
{
    ::testing::Test::RecordProperty("TEST_ID", "afa37f19-fde0-40ab-b1bd-10862f623ae7");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    std::string testString(STRINGCAP, 'M');
    string<STRINGCAP> fuu = into<lossy<MyString>>(testString);
    EXPECT_THAT(fuu.capacity(), Eq(STRINGCAP));
    EXPECT_THAT(fuu.size(), Eq(STRINGCAP));
    EXPECT_THAT(fuu.c_str(), Eq(testString));
}

TYPED_TEST(StdString_test, STDStringToLossyStringConvConstrWithSizeGreaterCapaResultsInSizeCapa)
{
    ::testing::Test::RecordProperty("TEST_ID", "67cba3f0-30ed-415d-8232-8e8b5898fe04");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    std::string testString(STRINGCAP + 1U, 'M');
    string<STRINGCAP> fuu = into<lossy<MyString>>(testString);
    EXPECT_THAT(fuu.capacity(), Eq(STRINGCAP));
    EXPECT_THAT(fuu.size(), Eq(STRINGCAP));
    EXPECT_THAT(fuu.c_str(), Eq(testString.substr(0U, STRINGCAP)));
}

TYPED_TEST(StdString_test, STDStringToOptionalStringConvConstrWithSize0ResultsInSize0)
{
    ::testing::Test::RecordProperty("TEST_ID", "357f6fbf-7848-4ba7-9de6-dfbf185d8c4b");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    std::string testString;
    into<optional<MyString>>(testString)
        .and_then([&](const auto& fuu) {
            EXPECT_THAT(fuu.capacity(), Eq(STRINGCAP));
            EXPECT_THAT(fuu.size(), Eq(0U));
            EXPECT_THAT(fuu.c_str(), StrEq(""));
        })
        .or_else([&] { GTEST_FAIL() << "Expected successful string conversion!"; });
}

TYPED_TEST(StdString_test, STDStringToOptionalStringConvConstrWithSizeSmallerCapaResultsInSizeSmallerCapa)
{
    ::testing::Test::RecordProperty("TEST_ID", "964223ae-aa70-4bf9-ab22-f3761b211ce4");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    std::string testString(STRINGCAP - 1U, 'M');
    into<optional<MyString>>(testString)
        .and_then([&](const auto& fuu) {
            EXPECT_THAT(fuu.capacity(), Eq(STRINGCAP));
            EXPECT_THAT(fuu.size(), Eq(STRINGCAP - 1U));
            EXPECT_THAT(fuu.c_str(), Eq(testString));
        })
        .or_else([&] { GTEST_FAIL() << "Expected successful string conversion!"; });
}

TYPED_TEST(StdString_test, STDStringToOptionalStringConvConstrWithSizeCapaResultsInSizeCapa)
{
    ::testing::Test::RecordProperty("TEST_ID", "dcfe0e07-4e3c-41a2-bf5f-c74497544701");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    std::string testString(STRINGCAP, 'M');
    into<optional<MyString>>(testString)
        .and_then([&](const auto& fuu) {
            EXPECT_THAT(fuu.capacity(), Eq(STRINGCAP));
            EXPECT_THAT(fuu.size(), Eq(STRINGCAP));
            EXPECT_THAT(fuu.c_str(), Eq(testString));
        })
        .or_else([&] { GTEST_FAIL() << "Expected successful string conversion!"; });
}

TYPED_TEST(StdString_test, STDStringToOptionalStringConvConstrWithSizeGreaterCapaResultsInSizeCapa)
{
    ::testing::Test::RecordProperty("TEST_ID", "fd99374b-49ec-46a1-870a-52a13efdd283");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    std::string testString(STRINGCAP + 1U, 'M');
    into<optional<MyString>>(testString)
        .and_then([&](const auto& fuu) {
            GTEST_FAIL() << "Expected string conversion from '" << testString << "' to fixed string with capacity '"
                         << STRINGCAP << "' to fail but got '" << fuu << "'";
        })
        .or_else([&] { GTEST_SUCCEED() << "Size of source string exceeds capacity!"; });
}

TYPED_TEST(StdString_test, EmptyStringToSTDStringConvResultsInZeroSize)
{
    ::testing::Test::RecordProperty("TEST_ID", "753888b8-12e2-4534-a2fd-32b29b457803");
    using MyString = typename TestFixture::stringType;
    MyString sut;
    std::string testString = into<std::string>(sut);
    EXPECT_THAT(testString.size(), Eq(0U));
    EXPECT_THAT(testString.c_str(), StrEq(""));
}

TYPED_TEST(StdString_test, StringOfSizeCapaToSTDStringConvResultsInSizeCapa)
{
    ::testing::Test::RecordProperty("TEST_ID", "50e727f3-c855-4613-9e38-a56429fa5748");
    using MyString = typename TestFixture::stringType;
    MyString sut;
    constexpr auto STRINGCAP = MyString::capacity();
    std::string testString1(STRINGCAP, 'M');
    EXPECT_THAT(sut.unsafe_assign(testString1.c_str()), Eq(true));
    std::string testString2 = into<std::string>(sut);

    EXPECT_THAT(testString2.size(), Eq(STRINGCAP));
    EXPECT_THAT(testString2.c_str(), StrEq(testString1.substr(0, STRINGCAP)));
}

TYPED_TEST(StdString_test, UnsafeAppendEmptyStdStringWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "d2da56ce-c68b-4d66-9fc6-25564776b3a4");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    this->testSubject = "M";
    std::string testStdString;
    EXPECT_THAT(this->testSubject.unsafe_append(testStdString), Eq(true));
    EXPECT_THAT(this->testSubject.capacity(), Eq(STRINGCAP));
    EXPECT_THAT(this->testSubject.size(), Eq(1U));
    EXPECT_THAT(this->testSubject.c_str(), StrEq("M"));
}

TYPED_TEST(StdString_test, UnsafeAppendFittingStdStringWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "32beaa61-3282-4964-af1f-b185b7cc50ee");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    string<5U * STRINGCAP> sut("R2-D");
    std::string testStdString = "2";
    EXPECT_THAT(sut.unsafe_append(testStdString), Eq(true));
    EXPECT_THAT(sut.capacity(), Eq(5U * STRINGCAP));
    EXPECT_THAT(sut.size(), Eq(5U));
    EXPECT_THAT(sut.c_str(), StrEq("R2-D2"));
}

TYPED_TEST(StdString_test, UnsafeAppendTooLargeStdStringFails)
{
    ::testing::Test::RecordProperty("TEST_ID", "ea5ed2f4-e7a5-4417-af30-8cec5af2d8d4");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    std::string testStdString(STRINGCAP + 1U, 'M');

    EXPECT_THAT(this->testSubject.unsafe_append(testStdString), Eq(false));
    EXPECT_THAT(this->testSubject.capacity(), Eq(STRINGCAP));
    EXPECT_THAT(this->testSubject.empty(), Eq(true));
}

TYPED_TEST(StdString_test, UnsafeAppendWithStdStringToEmptyStringWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "8f30ed18-c15c-4252-91b2-9506ca5a998c");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    std::string testStdString = "d";
    EXPECT_THAT(this->testSubject.unsafe_append(testStdString), Eq(true));
    EXPECT_THAT(this->testSubject.capacity(), Eq(STRINGCAP));
    EXPECT_THAT(this->testSubject.size(), Eq(1U));
    EXPECT_THAT(this->testSubject.c_str(), StrEq(testStdString));
}

TYPED_TEST(StdString_test, AppendEmptyStdStringWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "6b3fb31a-a92b-4013-ba61-12b4846e8593");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    this->testSubject = "M";
    std::string testStdString;
    this->testSubject.append(TruncateToCapacity, testStdString);
    EXPECT_THAT(this->testSubject.capacity(), Eq(STRINGCAP));
    EXPECT_THAT(this->testSubject.size(), Eq(1U));
    EXPECT_THAT(this->testSubject.c_str(), StrEq("M"));
}

TYPED_TEST(StdString_test, AppendStdStringToEmptyStringResultsInConcatenatedString)
{
    ::testing::Test::RecordProperty("TEST_ID", "2eebdc68-c495-4531-bb66-ee4ede8d86e3");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    std::string testStdString = "M";
    this->testSubject.append(TruncateToCapacity, testStdString);
    EXPECT_THAT(this->testSubject.capacity(), Eq(STRINGCAP));
    EXPECT_THAT(this->testSubject.size(), Eq(1U));
    EXPECT_THAT(this->testSubject.c_str(), StrEq("M"));
}

TYPED_TEST(StdString_test, AppendStdStringResultsInConcatenatedString)
{
    ::testing::Test::RecordProperty("TEST_ID", "7159ef46-f441-4cc7-8eff-46b5f3d33597");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    string<STRINGCAP + 5U> sut = "P";
    std::string testStdString("icard");
    sut.append(TruncateToCapacity, testStdString);
    EXPECT_THAT(sut.capacity(), Eq(STRINGCAP + 5U));
    EXPECT_THAT(sut.size(), Eq(6U));
    EXPECT_THAT(sut.c_str(), StrEq("Picard"));
}

TYPED_TEST(StdString_test, AppendTooLargeStdStringResultsInTruncatedString)
{
    ::testing::Test::RecordProperty("TEST_ID", "76a4752d-282f-47ad-87eb-7c8aab982c0c");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    this->testSubject = "M";
    std::string testStdString(STRINGCAP + 1U, 'M');
    this->testSubject.append(TruncateToCapacity, testStdString);
    EXPECT_THAT(this->testSubject.capacity(), Eq(STRINGCAP));
    EXPECT_THAT(this->testSubject.size(), Eq(STRINGCAP));
    EXPECT_THAT(this->testSubject.c_str(), StrEq(testStdString.substr(0, STRINGCAP)));
}

/// @note constexpr int64_t compare(const std::string& other) const noexcept
TYPED_TEST(StdString_test, CompareWithStdStringResultPositiveWithDifferentSize)
{
    ::testing::Test::RecordProperty("TEST_ID", "08891d54-3db7-47cd-8e42-9beb7035c044");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    std::string testString(STRINGCAP + 6U, 'M');
    string<STRINGCAP + 6U> sut;
    ASSERT_THAT(sut.unsafe_assign(testString.c_str()), Eq(true));
    std::string foo(STRINGCAP, 'M');
    EXPECT_THAT(sut.compare(foo), Gt(0));
}

TYPED_TEST(StdString_test, CompareWithStdStringResultNegativeWithDifferentSize)
{
    ::testing::Test::RecordProperty("TEST_ID", "fbfa1376-8474-4cee-8c83-5adb6dc115a8");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    std::string testString(STRINGCAP, 'M');
    ASSERT_THAT(this->testSubject.unsafe_assign(testString.c_str()), Eq(true));
    std::string foo(STRINGCAP + 4, 'M');
    EXPECT_THAT(this->testSubject.compare(foo), Lt(0));
}

/// @note template <uint64_t Capacity>
/// inline std::ostream& operator<<(std::ostream& stream, const string<Capacity>& str)
TYPED_TEST(StdString_test, EmptyStreamInputWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "88c68194-9a9c-4f2f-a0e0-90bd72f9b102");
    std::ostringstream testStream;
    testStream << "";
    EXPECT_THAT(testStream.str(), StrEq(""));
}

TYPED_TEST(StdString_test, StreamInputOfSizeCapacityWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "c9b3dff3-008d-4189-818f-3534767e7ee4");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    std::string testString(STRINGCAP, 'M');
    string<STRINGCAP> testFixedString(TruncateToCapacity, testString.c_str(), testString.size());
    std::ostringstream testStream;
    testStream << testFixedString;
    EXPECT_THAT(testStream.str(), Eq(testFixedString.c_str()));
}

/// @note int64_t compare(const T& other) const noexcept
/// with T = {std::string}
TYPED_TEST(StdString_test, CompareEqStdStringResultsInZero)
{
    ::testing::Test::RecordProperty("TEST_ID", "b9e8ba17-5de8-4a23-b310-e98c1fb8ecb9");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    std::string testStdString(STRINGCAP, 'M');
    ASSERT_THAT(this->testSubject.unsafe_assign(testStdString.c_str()), Eq(true));

    EXPECT_THAT(this->testSubject.compare(testStdString), Eq(0));
}

TYPED_TEST(StdString_test, CompareWithStdStringResultNegative)
{
    ::testing::Test::RecordProperty("TEST_ID", "3448a602-886a-4857-b5c3-d216bf01c004");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    std::string temp(STRINGCAP, 'L');
    ASSERT_THAT(this->testSubject.unsafe_assign(temp.c_str()), Eq(true));

    const std::string testStdString(STRINGCAP, 'M');
    EXPECT_THAT(this->testSubject.compare(testStdString), Lt(0));
}

TYPED_TEST(StdString_test, CompareWithStdStringResultPositive)
{
    ::testing::Test::RecordProperty("TEST_ID", "652304a8-cd72-438b-aa04-cd962fafe65e");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    std::string temp(STRINGCAP, 'M');
    ASSERT_THAT(this->testSubject.unsafe_assign(temp.c_str()), Eq(true));

    const std::string testStdString(STRINGCAP, 'L');
    EXPECT_THAT(this->testSubject.compare(testStdString), Gt(0));
}

TYPED_TEST(StdString_test, CompareWithEmptyStdStringResultsInPositive)
{
    ::testing::Test::RecordProperty("TEST_ID", "7e9efc73-8fe2-49e8-8436-f3f7f12e641b");
    this->testSubject = "M";

    const std::string testStdString;
    EXPECT_THAT(this->testSubject.compare(testStdString), Gt(0));
}

TYPED_TEST(StdString_test, CompareEqStringAndStdStringWithDifferentCapaResultsInZero)
{
    ::testing::Test::RecordProperty("TEST_ID", "8a42e8e1-88c4-43bb-ae7f-1f49ddf3161b");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    std::string temp(STRINGCAP, 'M');
    ASSERT_THAT(this->testSubject.unsafe_assign(temp.c_str()), Eq(true));

    std::string testStdString(STRINGCAP, 'M');
    testStdString.reserve(STRINGCAP + 13U);
    EXPECT_THAT(this->testSubject.compare(testStdString), Eq(0));
}

/// @note bool operator==(const T& rhs) const noexcept
/// bool operator!=(const T& rhs) const noexcept
/// bool operator==(const T& lhs, const string<Capacity>& rhs) noexcept
/// bool operator!=(const T& lhs, const string<Capacity>& rhs) noexcept
/// with T = {std::string}
TYPED_TEST(StdString_test, CheckForEqualityWithEqualStdStringsWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "f59dac18-3ee4-46de-86e1-e7838bd25d5a");
    this->testSubject = "M";

    const std::string testStdString{"M"};
    EXPECT_THAT(this->testSubject == testStdString, Eq(true));
    EXPECT_THAT(testStdString == this->testSubject, Eq(true));
    EXPECT_THAT(this->testSubject != testStdString, Eq(false));
    EXPECT_THAT(testStdString != this->testSubject, Eq(false));
}

TYPED_TEST(StdString_test, CheckForEqualityWithUnequalStdStringsWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "8851f836-aa00-416a-9461-77f0909bcf1a");
    this->testSubject = "M";

    const std::string testStdString{"L"};
    EXPECT_THAT(this->testSubject == testStdString, Eq(false));
    EXPECT_THAT(testStdString == this->testSubject, Eq(false));
    EXPECT_THAT(this->testSubject != testStdString, Eq(true));
    EXPECT_THAT(testStdString != this->testSubject, Eq(true));
}

TYPED_TEST(StdString_test, CheckForEqualityWithEqualStdStringWithDifferentCapaWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "6a7f74bf-a81d-421c-80d9-39ab48d5086c");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    this->testSubject = "M";

    std::string testStdString = "M";
    testStdString.reserve(STRINGCAP + 5U);
    EXPECT_THAT(this->testSubject == testStdString, Eq(true));
    EXPECT_THAT(testStdString == this->testSubject, Eq(true));
    EXPECT_THAT(this->testSubject != testStdString, Eq(false));
    EXPECT_THAT(testStdString != this->testSubject, Eq(false));
}

TYPED_TEST(StdString_test, CheckForEqualityWithUnequalStdStringWithDifferentSizeWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "6aa3b1ab-29b5-456a-a27d-c531462c7a6e");
    this->testSubject = "M";

    const std::string testStdString = "ML";
    EXPECT_THAT(this->testSubject == testStdString, Eq(false));
    EXPECT_THAT(testStdString == this->testSubject, Eq(false));
    EXPECT_THAT(this->testSubject != testStdString, Eq(true));
    EXPECT_THAT(testStdString != this->testSubject, Eq(true));
}

/// @note bool operator<(const T& rhs) const noexcept
/// bool operator<=(const T& rhs) const noexcept
/// bool operator>(const T& rhs) const noexcept
/// bool operator>=(const T& rhs) const noexcept
/// bool operator<(const T& lhs, const string<Capacity>& rhs) noexcept
/// bool operator<=(const T& lhs, const string<Capacity>& rhs) noexcept
/// bool operator>(const T& lhs, const string<Capacity>& rhs) noexcept
/// bool operator>=(const T& lhs, const string<Capacity>& rhs) noexcept
/// with T = {std::string}
TYPED_TEST(StdString_test, CompareOperatorsWithStdString)
{
    ::testing::Test::RecordProperty("TEST_ID", "a7ecd8c4-0aa4-41ce-a49a-b1dfdb22b9f1");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();

    std::string testStdString = "L";

    // compare with greater string
    string<STRINGCAP> sutGreater("M");

    EXPECT_THAT(sutGreater < testStdString, Eq(false));
    EXPECT_THAT(sutGreater <= testStdString, Eq(false));
    EXPECT_THAT(sutGreater > testStdString, Eq(true));
    EXPECT_THAT(sutGreater >= testStdString, Eq(true));
    EXPECT_THAT(testStdString < sutGreater, Eq(true));
    EXPECT_THAT(testStdString <= sutGreater, Eq(true));
    EXPECT_THAT(testStdString > sutGreater, Eq(false));
    EXPECT_THAT(testStdString >= sutGreater, Eq(false));

    // compare with less string
    string<STRINGCAP> sutLess("F");

    EXPECT_THAT(sutLess < testStdString, Eq(true));
    EXPECT_THAT(sutLess <= testStdString, Eq(true));
    EXPECT_THAT(sutLess > testStdString, Eq(false));
    EXPECT_THAT(sutLess >= testStdString, Eq(false));
    EXPECT_THAT(testStdString < sutLess, Eq(false));
    EXPECT_THAT(testStdString <= sutLess, Eq(false));
    EXPECT_THAT(testStdString > sutLess, Eq(true));
    EXPECT_THAT(testStdString >= sutLess, Eq(true));
}

TYPED_TEST(StdString_test, CompareOperatorsWithEqualStdString)
{
    ::testing::Test::RecordProperty("TEST_ID", "106afc4c-7dcb-4579-bbbb-feb3cadabe0a");
    this->testSubject = "M";

    const std::string testStdString = "M";
    EXPECT_THAT(this->testSubject < testStdString, Eq(false));
    EXPECT_THAT(this->testSubject <= testStdString, Eq(true));
    EXPECT_THAT(this->testSubject > testStdString, Eq(false));
    EXPECT_THAT(this->testSubject >= testStdString, Eq(true));

    EXPECT_THAT(testStdString < this->testSubject, Eq(false));
    EXPECT_THAT(testStdString <= this->testSubject, Eq(true));
    EXPECT_THAT(testStdString > this->testSubject, Eq(false));
    EXPECT_THAT(testStdString >= this->testSubject, Eq(true));
}

TYPED_TEST(StdString_test, CompareOperatorsWithDifferentStdStringWithDifferentSize)
{
    ::testing::Test::RecordProperty("TEST_ID", "844fc675-25a2-415f-99c3-5eece0f2315d");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();

    const std::string testStdString(STRINGCAP, 'L');

    // compare with greater string
    std::string temp1(STRINGCAP + 5U, 'M');
    string<STRINGCAP + 5U> sutGreater;
    ASSERT_THAT(sutGreater.unsafe_assign(temp1.c_str()), Eq(true));

    EXPECT_THAT(sutGreater < testStdString, Eq(false));
    EXPECT_THAT(sutGreater <= testStdString, Eq(false));
    EXPECT_THAT(sutGreater > testStdString, Eq(true));
    EXPECT_THAT(sutGreater >= testStdString, Eq(true));
    EXPECT_THAT(testStdString < sutGreater, Eq(true));
    EXPECT_THAT(testStdString <= sutGreater, Eq(true));
    EXPECT_THAT(testStdString > sutGreater, Eq(false));
    EXPECT_THAT(testStdString >= sutGreater, Eq(false));

    // compare with less string
    std::string temp2(STRINGCAP + 5U, 'F');
    string<STRINGCAP + 5U> sutLess;
    ASSERT_THAT(sutLess.unsafe_assign(temp2.c_str()), Eq(true));

    EXPECT_THAT(sutLess < testStdString, Eq(true));
    EXPECT_THAT(sutLess <= testStdString, Eq(true));
    EXPECT_THAT(sutLess > testStdString, Eq(false));
    EXPECT_THAT(sutLess >= testStdString, Eq(false));
    EXPECT_THAT(testStdString < sutLess, Eq(false));
    EXPECT_THAT(testStdString <= sutLess, Eq(false));
    EXPECT_THAT(testStdString > sutLess, Eq(true));
    EXPECT_THAT(testStdString >= sutLess, Eq(true));
}

TYPED_TEST(StdString_test, AppendStdStringContainingNullWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "f8814d78-449b-4c3a-b7c7-7c3ff2a0a62f");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    constexpr auto RESULT_CAPACITY = STRINGCAP + 10U;
    const std::string expectedString{"ice\0ryx", 7};

    string<RESULT_CAPACITY> sut("i");
    const string<RESULT_CAPACITY> testCxxString(TruncateToCapacity, expectedString.substr(1).c_str(), 6U);
    const std::string testStdString = expectedString.substr(1);

    // append std::string
    sut.append(TruncateToCapacity, testStdString);
    EXPECT_THAT(sut.capacity(), Eq(RESULT_CAPACITY));
    EXPECT_THAT(sut.size(), Eq(7U));
    EXPECT_THAT(std::memcmp(sut.c_str(), expectedString.c_str(), static_cast<size_t>(sut.size())), Eq(0));
}

TYPED_TEST(StdString_test, FindStdStringInEmptyStringFails)
{
    ::testing::Test::RecordProperty("TEST_ID", "75e34d61-be16-4892-a931-96a96bc1e45f");
    std::string testStdString = "a";

    auto res = this->testSubject.find(testStdString);

    EXPECT_THAT(res.has_value(), Eq(false));
}

TYPED_TEST(StdString_test, FindEmptyStdStringInEmptyStringWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "9991ab0c-61e9-44b5-817b-dae155317f0d");
    std::string testStdString;

    auto res = this->testSubject.find(testStdString);

    EXPECT_THAT(res.has_value(), Eq(true));
}

/// @note template <typename T>
/// iox::optional<uint64_t> find_first_of(const T& t, uint64_t pos = 0) const noexcept
TYPED_TEST(StdString_test, FindFirstOfFailsForEmptyStdStringInEmptyString)
{
    ::testing::Test::RecordProperty("TEST_ID", "207671e4-cef3-40d2-8984-e8ae5c2b42ec");
    std::string testStdString;

    auto res = this->testSubject.find_first_of(testStdString);

    EXPECT_THAT(res.has_value(), Eq(false));
}

TYPED_TEST(StdString_test, FindFirstOfForStdStringInEmptyStringFails)
{
    ::testing::Test::RecordProperty("TEST_ID", "cfd4a842-64e3-4a2c-afc9-d98f93f1f8f4");
    std::string testStdString = "a";

    auto res = this->testSubject.find_first_of(testStdString);

    EXPECT_THAT(res.has_value(), Eq(false));
}

TEST(String100, FindFirstOfForSTDStringInNotEmptyStringWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "1f44acab-aa37-4f45-a782-06ad02bd926b");
    string<100U> testString("R2-D2");
    std::string testStdString1 = "2";
    auto res = testString.find_first_of(testStdString1);
    ASSERT_THAT(res.has_value(), Eq(true));
    EXPECT_THAT(res.value(), Eq(1U));

    res = testString.find_first_of(testStdString1, 1U);
    ASSERT_THAT(res.has_value(), Eq(true));
    EXPECT_THAT(res.value(), Eq(1U));

    res = testString.find_first_of(testStdString1, 2U);
    ASSERT_THAT(res.has_value(), Eq(true));
    EXPECT_THAT(res.value(), Eq(4U));

    std::string testStdString2 = "D3R";
    res = testString.find_first_of(testStdString2);
    ASSERT_THAT(res.has_value(), Eq(true));
    EXPECT_THAT(res.value(), Eq(0U));

    res = testString.find_first_of(testStdString2, 1U);
    ASSERT_THAT(res.has_value(), Eq(true));
    EXPECT_THAT(res.value(), Eq(3U));
}

TEST(String100, FindFirstOfForNotIncludedStdStringFails)
{
    ::testing::Test::RecordProperty("TEST_ID", "700d9fd9-4039-490e-9dd4-3833fb9f5e08");
    string<100U> testString("Kernfusionsbaby");
    std::string testStdString = "cd";
    auto res = testString.find_first_of(testStdString);
    EXPECT_THAT(res.has_value(), Eq(false));

    res = testString.find_first_of(testStdString, 0U);
    EXPECT_THAT(res.has_value(), Eq(false));

    res = testString.find_first_of(testStdString, 50U);
    EXPECT_THAT(res.has_value(), Eq(false));
}

TEST(String100, FindNotIncludedStdStringFails)
{
    ::testing::Test::RecordProperty("TEST_ID", "8b2116c9-5f7d-48b4-8c26-cb3b71cf0ea2");
    string<100U> testString("Kernfusionsbaby");
    std::string testStdString = "abc";
    auto res = testString.find(testStdString);
    EXPECT_THAT(res.has_value(), Eq(false));

    res = testString.find(testStdString, 0U);
    EXPECT_THAT(res.has_value(), Eq(false));

    res = testString.find(testStdString, 50U);
    EXPECT_THAT(res.has_value(), Eq(false));
}

/// @note template <typename T>
/// iox::optional<uint64_t> find_last_of(const T& t, uint64_t pos = 0) const noexcept
TYPED_TEST(StdString_test, FindLastOfFailsForEmptyStdStringInEmptyString)
{
    ::testing::Test::RecordProperty("TEST_ID", "15f72273-8b90-407f-b7d0-07372f3cee29");
    std::string testStdString;

    auto res = this->testSubject.find_last_of(testStdString);

    EXPECT_THAT(res.has_value(), Eq(false));
}

TYPED_TEST(StdString_test, FindLastOfForStdStringInEmptyStringFails)
{
    ::testing::Test::RecordProperty("TEST_ID", "f8b48cdb-7165-41d6-9eb3-f148c6edd859");
    std::string testStdString = "a";

    auto res = this->testSubject.find_last_of(testStdString);

    EXPECT_THAT(res.has_value(), Eq(false));
}

TEST(String100, FindLastOfForStdStringInNotEmptyStringWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "f068fa78-1e97-4148-bbba-da9cc2cf022e");
    string<100U> testString("R2-D2");
    std::string testStdString1 = "2";
    auto res = testString.find_last_of(testStdString1);
    ASSERT_THAT(res.has_value(), Eq(true));
    EXPECT_THAT(res.value(), Eq(4U));

    res = testString.find_last_of(testStdString1, 1U);
    ASSERT_THAT(res.has_value(), Eq(true));
    EXPECT_THAT(res.value(), Eq(1U));

    res = testString.find_last_of(testStdString1, 5U);
    ASSERT_THAT(res.has_value(), Eq(true));
    EXPECT_THAT(res.value(), Eq(4U));

    std::string testStdString2 = "D3R";
    res = testString.find_last_of(testStdString2);
    ASSERT_THAT(res.has_value(), Eq(true));
    EXPECT_THAT(res.value(), Eq(3U));

    res = testString.find_last_of(testStdString2, 1U);
    ASSERT_THAT(res.has_value(), Eq(true));
    EXPECT_THAT(res.value(), Eq(0U));
}

TEST(String100, FindLastOfForNotIncludedSTDStringFails)
{
    ::testing::Test::RecordProperty("TEST_ID", "91df370f-38ee-41e8-8063-5f4c3010374f");
    string<100U> testString("Kernfusionsbaby");
    std::string testStdString = "cd";
    auto res = testString.find_last_of(testStdString);
    EXPECT_THAT(res.has_value(), Eq(false));

    res = testString.find_last_of(testStdString, 0U);
    EXPECT_THAT(res.has_value(), Eq(false));

    res = testString.find_last_of(testStdString, 50U);
    EXPECT_THAT(res.has_value(), Eq(false));
}

TEST(String100, FindSTDStringInNotEmptyStringWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "794f62c2-deca-4511-9529-3353ff9ee552");
    string<100U> testString("R2-D2");
    string<100U> testStdString = "2";
    auto res = testString.find(testStdString);
    ASSERT_THAT(res.has_value(), Eq(true));
    EXPECT_THAT(res.value(), Eq(1U));

    res = testString.find(testStdString, 1U);
    ASSERT_THAT(res.has_value(), Eq(true));
    EXPECT_THAT(res.value(), Eq(1U));

    res = testString.find(testStdString, 2U);
    ASSERT_THAT(res.has_value(), Eq(true));
    EXPECT_THAT(res.value(), Eq(4U));
}
} // namespace
