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

// NOTE: To speed up compilation and the clang-tidy check, the tests for 'iox::string' are split into multiple files

namespace
{
using namespace ::testing;
using namespace iox;
using namespace iox::testing;

TYPED_TEST_SUITE(stringTyped_test, StringImplementations, );

TEST(string_test, CapacityReturnsSpecifiedCapacity)
{
    ::testing::Test::RecordProperty("TEST_ID", "6eed7f1a-c8d9-4902-ac22-405a3bd62d28");
    constexpr uint16_t CAPACITY_ONE{1};
    constexpr uint16_t CAPACITY_FOURTYTWO{42};

    EXPECT_THAT(string<CAPACITY_ONE>::capacity(), Eq(CAPACITY_ONE));
    EXPECT_THAT(string<CAPACITY_FOURTYTWO>::capacity(), Eq(CAPACITY_FOURTYTWO));
}

/// @note void unsafe_raw_access(const std::function<void(char*, const uint64_t, const uint64_t)>& func) noexcept
TYPED_TEST(stringTyped_test, UnsafeRawAccessOfCStringOfSize0ResultsInSize0)
{
    ::testing::Test::RecordProperty("TEST_ID", "43e10399-445d-42af-80b1-25071590de0a");
    this->testSubject.unsafe_raw_access([this](char* str, const auto info) -> uint64_t {
        //NOLINTNEXTLINE(clang-analyzer-security.insecureAPI.strcpy,-warnings-as-errors)
        strcpy(str, "");
        EXPECT_THAT(info.used_size, this->testSubject.size());
        using MyString = typename TestFixture::stringType;
        EXPECT_THAT(info.total_size, MyString::capacity() + 1); // real buffer size
        return 0U;
    });
    EXPECT_THAT(this->testSubject.size(), Eq(0U));
    EXPECT_THAT(this->testSubject.c_str(), StrEq(""));
}

TYPED_TEST(stringTyped_test, UnsafeRawAccessOfCStringOfSize1ResultsInSize1)
{
    ::testing::Test::RecordProperty("TEST_ID", "a3a3395e-2b69-400c-876a-1fdf70cf2d4a");
    this->testSubject.unsafe_raw_access([this](char* str, const auto info) -> uint64_t {
        //NOLINTNEXTLINE(clang-analyzer-security.insecureAPI.strcpy,-warnings-as-errors)
        strcpy(str, "M");
        EXPECT_THAT(info.used_size, this->testSubject.size());
        using MyString = typename TestFixture::stringType;
        EXPECT_THAT(info.total_size, MyString::capacity() + 1); // real buffer size
        return 1U;
    });
    EXPECT_THAT(this->testSubject.size(), Eq(1U));
    EXPECT_THAT(this->testSubject.c_str(), StrEq("M"));
}

TYPED_TEST(stringTyped_test, UnsafeRawAccessCStringOfSizeCapaResultsInSizeCapa)
{
    ::testing::Test::RecordProperty("TEST_ID", "49faad68-52fa-4024-993c-49b05e7cb971");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    std::vector<char> testCharstring(STRINGCAP, 'M');
#if (defined(__GNUC__) && __GNUC__ >= 7 && !defined(__clang__))
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Warray-bounds"
#endif
    testCharstring.emplace_back('\0');
#if (defined(__GNUC__) && __GNUC__ >= 7 && !defined(__clang__))
#pragma GCC diagnostic pop
#endif
    this->testSubject.unsafe_raw_access([&](char* str, const auto) -> uint64_t {
        //NOLINTNEXTLINE(clang-analyzer-security.insecureAPI.strcpy,-warnings-as-errors)
        strcpy(str, testCharstring.data());
        return STRINGCAP;
    });
    EXPECT_THAT(this->testSubject.unsafe_assign(testCharstring.data()), Eq(true));
    EXPECT_THAT(this->testSubject.size(), Eq(STRINGCAP));
}

TYPED_TEST(stringTyped_test, UnsafeRawAccessCStringOutOfBoundFail)
{
    ::testing::Test::RecordProperty("TEST_ID", "b25c35db-1c0d-4f0e-b4bc-b9430a6696f1");

    runInTestThread([this] {
        this->testSubject.unsafe_raw_access([](char* str, const auto info) -> uint64_t {
            //NOLINTNEXTLINE(clang-analyzer-security.insecureAPI.strcpy,-warnings-as-errors)
            strcpy(str, "M");
            return info.total_size + 1U;
        });
    });
    IOX_TESTING_EXPECT_PANIC();
}

TYPED_TEST(stringTyped_test, UnsafeRawAccessCStringWrongLenghtFail)
{
    ::testing::Test::RecordProperty("TEST_ID", "411f5db1-18b8-45c3-9ad6-3c886fb12a26");

    runInTestThread([this] {
        this->testSubject.unsafe_raw_access([](char* str, const auto) -> uint64_t {
            //NOLINTNEXTLINE(clang-analyzer-security.insecureAPI.strcpy,-warnings-as-errors)
            strcpy(str, "M");
            return 0U;
        });
    });
    IOX_TESTING_EXPECT_PANIC();
}

/// @note constexpr bool empty() const noexcept
TYPED_TEST(stringTyped_test, NewlyCreatedStringIsEmpty)
{
    ::testing::Test::RecordProperty("TEST_ID", "b76d10c1-46e2-4bc3-ab79-af187b85d584");
    using MyString = typename TestFixture::stringType;
    MyString sut;
    EXPECT_THAT(sut.empty(), Eq(true));
}

TYPED_TEST(stringTyped_test, StringWithContentIsNotEmtpy)
{
    ::testing::Test::RecordProperty("TEST_ID", "bb7fd1ff-82dc-4ae2-af70-9b080eb2265d");
    using MyString = typename TestFixture::stringType;
    MyString sut(TruncateToCapacity, "Dr.SchluepferStrikesAgain!");
    EXPECT_THAT(sut.empty(), Eq(false));
}

/// @note void clear() noexcept
TYPED_TEST(stringTyped_test, ClearEmptyStringDoesNotChangeString)
{
    ::testing::Test::RecordProperty("TEST_ID", "5109413a-6067-4f7f-ac35-8dd3a33a641f");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString().capacity();

    this->testSubject.clear();
    EXPECT_THAT(this->testSubject.empty(), Eq(true));
    EXPECT_THAT(this->testSubject.capacity(), Eq(STRINGCAP));
}

TYPED_TEST(stringTyped_test, ClearNotEmptyStringResultsInEmptyStringWithUnchangedCapacity)
{
    ::testing::Test::RecordProperty("TEST_ID", "d25d2a74-6ea0-4892-b072-02177b31309e");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString().capacity();
    this->testSubject = "M";
    ASSERT_THAT(this->testSubject.empty(), Eq(false));

    this->testSubject.clear();
    EXPECT_THAT(this->testSubject.empty(), Eq(true));
    EXPECT_THAT(this->testSubject.capacity(), Eq(STRINGCAP));
}

TYPED_TEST(stringTyped_test, ChangeStringAfterClearWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "58444d00-ec85-464f-8269-7838823f04c2");
    this->testSubject.clear();
    this->testSubject = "M";
    EXPECT_THAT(this->testSubject.c_str(), StrEq("M"));
}

/// @note iox::optional<string<Capacity>> substr(uint64_t pos = 0) const noexcept;
TYPED_TEST(stringTyped_test, SubstrWithDefaultPosAndSizeResultsInWholeString)
{
    ::testing::Test::RecordProperty("TEST_ID", "da66bb36-2a1c-435b-8a47-874eb12315ef");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    std::string testStdString(STRINGCAP, 'M');
    EXPECT_THAT(this->testSubject.unsafe_assign(testStdString.c_str()), Eq(true));
    auto res = this->testSubject.substr();
    ASSERT_THAT(res.has_value(), Eq(true));

    auto testSubstring = res.value();
    std::string testStdSubstring = testStdString.substr();
    EXPECT_THAT(testSubstring.capacity(), Eq(STRINGCAP));
    EXPECT_THAT(testSubstring.size(), Eq(testStdSubstring.size()));
    EXPECT_THAT(testSubstring.c_str(), StrEq(testStdSubstring));
    EXPECT_THAT(this->testSubject.capacity(), Eq(STRINGCAP));
    EXPECT_THAT(this->testSubject.size(), Eq(STRINGCAP));
    EXPECT_THAT(this->testSubject.c_str(), StrEq(testStdString));
}

TEST(String100, SubstrWithDefaultSizeWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "10b1244f-7245-449d-b6d0-5c94faa7d274");
    constexpr uint64_t STRINGCAP = 100U;
    constexpr uint64_t SUBSTR_POS = 8U;
    std::string testStdString = "Mueslimaedchen";
    std::string testStdSubstring = testStdString.substr(SUBSTR_POS);
    string<STRINGCAP> testCxxString(TruncateToCapacity, testStdString.c_str(), testStdString.size());
    auto res = testCxxString.substr(SUBSTR_POS);
    ASSERT_THAT(res.has_value(), Eq(true));

    auto testSubstring = res.value();
    EXPECT_THAT(testSubstring.capacity(), Eq(STRINGCAP));
    EXPECT_THAT(testSubstring.size(), Eq(testStdSubstring.size()));
    EXPECT_THAT(testSubstring.c_str(), StrEq(testStdSubstring));
}

/// @note iox::optional<string<Capacity>> substr(uint64_t pos, uint64_t count) const noexcept
TEST(String100, SubstrWithValidPosAndSizeWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "3cd90af2-97f4-4767-854d-d3ca726bd348");
    constexpr uint64_t STRINGCAP = 100U;
    std::string testStdString = "Ferdinand Spitzschnueffler";
    string<STRINGCAP> testCxxString(TruncateToCapacity, testStdString.c_str(), testStdString.size());

    std::string testStdSubstring = testStdString.substr(0, 19);
    auto res1 = testCxxString.substr(0U, 19U);
    ASSERT_THAT(res1.has_value(), Eq(true));
    auto testSubstring1 = res1.value();
    EXPECT_THAT(testSubstring1.capacity(), Eq(STRINGCAP));
    EXPECT_THAT(testSubstring1.size(), Eq(testStdSubstring.size()));
    EXPECT_THAT(testSubstring1.c_str(), StrEq(testStdSubstring));

    testStdSubstring = testStdString.substr(20, 5);
    auto res2 = testCxxString.substr(20U, 5U);
    EXPECT_THAT(res2.has_value(), Eq(true));
    auto testSubstring2 = res2.value();
    EXPECT_THAT(testSubstring2.capacity(), Eq(STRINGCAP));
    EXPECT_THAT(testSubstring2.size(), Eq(testStdSubstring.size()));
    EXPECT_THAT(testSubstring2.c_str(), StrEq(testStdSubstring));

    testStdSubstring = testStdString.substr(0, 26);
    auto res3 = testCxxString.substr(0U, 26U);
    ASSERT_THAT(res3.has_value(), Eq(true));
    auto testSubstring3 = res3.value();
    EXPECT_THAT(testSubstring3.capacity(), Eq(STRINGCAP));
    EXPECT_THAT(testSubstring3.size(), Eq(testStdSubstring.size()));
    EXPECT_THAT(testSubstring3.c_str(), StrEq(testStdSubstring));

    testStdSubstring = testStdString.substr(11, 8);
    auto res4 = testCxxString.substr(11U, 8U);
    ASSERT_THAT(res4.has_value(), Eq(true));
    auto testSubstring4 = res4.value();
    EXPECT_THAT(testSubstring4.capacity(), Eq(STRINGCAP));
    EXPECT_THAT(testSubstring4.size(), Eq(testStdSubstring.size()));
    EXPECT_THAT(testSubstring4.c_str(), StrEq(testStdSubstring));

    testStdSubstring = testStdString.substr(13, 98);
    auto res5 = testCxxString.substr(13U, 98U);
    ASSERT_THAT(res5.has_value(), Eq(true));
    auto testSubstring5 = res5.value();
    EXPECT_THAT(testSubstring5.capacity(), Eq(STRINGCAP));
    EXPECT_THAT(testSubstring5.size(), Eq(testStdSubstring.size()));
    EXPECT_THAT(testSubstring5.c_str(), StrEq(testStdSubstring));
}

TYPED_TEST(stringTyped_test, SubstrWithInvalidPosFails)
{
    ::testing::Test::RecordProperty("TEST_ID", "2298f577-2974-4a01-82fb-a11d1f6faf6e");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    auto res = this->testSubject.substr(STRINGCAP + 1U, STRINGCAP + 2U);
    EXPECT_THAT(res.has_value(), Eq(false));
}

/// @note template <typename T>
/// iox::optional<uint64_t> find(const T& t, uint64_t pos = 0) const noexcept
TYPED_TEST(stringTyped_test, FindEmptyStringInEmptyStringWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "3ceb4ed6-1395-445e-afe4-94f6c9b2cee8");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    string<STRINGCAP + 5U> testString;
    auto res = this->testSubject.find(testString);
    ASSERT_THAT(res.has_value(), Eq(true));
    EXPECT_THAT(res.value(), Eq(0U));

    res = this->testSubject.find("");
    ASSERT_THAT(res.has_value(), Eq(true));
    EXPECT_THAT(res.value(), Eq(0U));
}

TYPED_TEST(stringTyped_test, FindStringInEmptyStringFails)
{
    ::testing::Test::RecordProperty("TEST_ID", "feb48bb6-c6f0-4f8b-8ce5-bf881fd876cb");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    string<STRINGCAP + 5U> testString("a");
    auto res = this->testSubject.find(testString);
    EXPECT_THAT(res.has_value(), Eq(false));

    res = this->testSubject.find("a");
    EXPECT_THAT(res.has_value(), Eq(false));
}

TEST(String100, FindStringInNotEmptyStringWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "b487b6bc-1c40-46f1-8285-dd1660d2e882");
    string<10U> testString("R2-D2");
    string<100U> substring("2");
    auto res = testString.find(substring);
    ASSERT_THAT(res.has_value(), Eq(true));
    EXPECT_THAT(res.value(), Eq(1U));

    res = testString.find(substring, 1U);
    ASSERT_THAT(res.has_value(), Eq(true));
    EXPECT_THAT(res.value(), Eq(1U));

    res = testString.find(substring, 2U);
    ASSERT_THAT(res.has_value(), Eq(true));
    EXPECT_THAT(res.value(), Eq(4U));
}

TEST(String100, FindNotIncludedStringFails)
{
    ::testing::Test::RecordProperty("TEST_ID", "bb0ca1b6-3dbd-491a-acac-e1f57ad2eb4f");
    constexpr uint64_t STRINGCAP = 100U;
    string<STRINGCAP> testString("Kernfusionsbaby");
    string<STRINGCAP> substring("abc");
    auto res = testString.find(substring);
    EXPECT_THAT(res.has_value(), Eq(false));

    res = testString.find(substring, 0U);
    EXPECT_THAT(res.has_value(), Eq(false));

    res = testString.find(substring, 50U);
    EXPECT_THAT(res.has_value(), Eq(false));
}

TEST(String100, FindStringLiteralInNotEmptyStringWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "b9a3018f-10d5-4f57-bfeb-f536a6ea9642");
    constexpr uint64_t STRINGCAP = 100U;

    string<STRINGCAP> testString1("Mueslimaedchen");
    auto res = testString1.find("lima");
    ASSERT_THAT(res.has_value(), Eq(true));
    EXPECT_THAT(res.value(), Eq(4U));

    res = testString1.find("lima", 2U);
    ASSERT_THAT(res.has_value(), Eq(true));
    EXPECT_THAT(res.value(), Eq(4U));

    res = testString1.find("e", 10U);
    ASSERT_THAT(res.has_value(), Eq(true));
    EXPECT_THAT(res.value(), Eq(12U));

    constexpr uint64_t STRING_COUNT = 7U;
    std::string testStdString{"ice\0ryx", STRING_COUNT};
    string<STRINGCAP> testString2(TruncateToCapacity, testStdString.c_str(), STRING_COUNT);
    res = testString2.find("e\0ry", 0U);
    ASSERT_THAT(res.has_value(), Eq(true));
    EXPECT_THAT(res.value(), Eq(2U));
}

TEST(String100, FindNotIncludedStringLiteralFails)
{
    ::testing::Test::RecordProperty("TEST_ID", "bd1ef66b-3698-4049-9d62-131b6de51b76");
    string<100U> testString("Kernfusionsbaby");
    auto res = testString.find("abc");
    EXPECT_THAT(res.has_value(), Eq(false));

    res = testString.find("abc", 50U);
    EXPECT_THAT(res.has_value(), Eq(false));
}

/// @note template <typename T>
/// iox::optional<uint64_t> find_first_of(const T& t, uint64_t pos = 0) const noexcept
TYPED_TEST(stringTyped_test, FindFirstOfFailsForEmptyStringInEmptyString)
{
    ::testing::Test::RecordProperty("TEST_ID", "21f90f13-6b15-4ce1-9258-c21154b6043c");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    string<STRINGCAP + 5U> testString;
    auto res = this->testSubject.find_first_of(testString);
    EXPECT_THAT(res.has_value(), Eq(false));

    res = this->testSubject.find_first_of("");
    EXPECT_THAT(res.has_value(), Eq(false));
}

TYPED_TEST(stringTyped_test, FindFirstOfForStringInEmptyStringFails)
{
    ::testing::Test::RecordProperty("TEST_ID", "1a1b1272-fc5b-431e-980d-39357f66e882");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    string<STRINGCAP + 5U> testString("a");
    auto res = this->testSubject.find_first_of(testString);
    EXPECT_THAT(res.has_value(), Eq(false));

    res = this->testSubject.find_first_of("a");
    EXPECT_THAT(res.has_value(), Eq(false));
}

TEST(String100, FindFirstOfForStringInNotEmptyStringWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "b24dbd99-3595-4b04-9e6e-177d2a6e4ad0");
    constexpr uint64_t STRINGCAP = 10U;
    constexpr uint64_t SUB_STRINGCAP = 100U;
    string<STRINGCAP> testString("R2-D2");
    string<SUB_STRINGCAP> substring1("2");
    auto res = testString.find_first_of(substring1);
    ASSERT_THAT(res.has_value(), Eq(true));
    EXPECT_THAT(res.value(), Eq(1U));

    res = testString.find_first_of(substring1, 1U);
    ASSERT_THAT(res.has_value(), Eq(true));
    EXPECT_THAT(res.value(), Eq(1U));

    res = testString.find_first_of(substring1, 2U);
    ASSERT_THAT(res.has_value(), Eq(true));
    EXPECT_THAT(res.value(), Eq(4U));

    string<SUB_STRINGCAP> substring2("D3R");
    res = testString.find_first_of(substring2);
    ASSERT_THAT(res.has_value(), Eq(true));
    EXPECT_THAT(res.value(), Eq(0U));

    res = testString.find_first_of(substring2, 1U);
    ASSERT_THAT(res.has_value(), Eq(true));
    EXPECT_THAT(res.value(), Eq(3U));
}

TEST(String100, FindFirstOfForNotIncludedStringFails)
{
    ::testing::Test::RecordProperty("TEST_ID", "f42e026a-063b-43dd-8a6a-157330f3f426");
    constexpr uint64_t STRINGCAP = 100U;
    string<STRINGCAP> testString("Kernfusionsbaby");
    string<STRINGCAP> substring("cdG");
    auto res = testString.find_first_of(substring);
    EXPECT_THAT(res.has_value(), Eq(false));

    res = testString.find_first_of(substring, 0U);
    EXPECT_THAT(res.has_value(), Eq(false));

    res = testString.find_first_of(substring, 50U);
    EXPECT_THAT(res.has_value(), Eq(false));
}

TEST(String100, FindFirstOfForStringLiteralInNotEmptyStringWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "cfb3c539-c442-4a80-82b2-2da4ea37f1cd");
    constexpr uint64_t STRINGCAP = 100U;
    string<STRINGCAP> testString1("Mueslimaedchen");
    auto res = testString1.find_first_of("lima");
    ASSERT_THAT(res.has_value(), Eq(true));
    EXPECT_THAT(res.value(), Eq(4U));

    res = testString1.find_first_of("mali", 2U);
    ASSERT_THAT(res.has_value(), Eq(true));
    EXPECT_THAT(res.value(), Eq(4U));

    res = testString1.find_first_of("e", 10U);
    ASSERT_THAT(res.has_value(), Eq(true));
    EXPECT_THAT(res.value(), Eq(12U));

    res = testString1.find_first_of("U3M");
    ASSERT_THAT(res.has_value(), Eq(true));
    EXPECT_THAT(res.value(), Eq(0U));

    constexpr uint64_t STRING_COUNT = 7U;
    std::string testStdString{"ice\0ryx", STRING_COUNT};
    string<STRINGCAP> testString2(TruncateToCapacity, testStdString.c_str(), STRING_COUNT);
    res = testString2.find_first_of("e\0ry", 0U);
    ASSERT_THAT(res.has_value(), Eq(true));
    EXPECT_THAT(res.value(), Eq(2U));
}

TEST(String100, FindFirstOfForNotIncludedStringLiteralFails)
{
    ::testing::Test::RecordProperty("TEST_ID", "bc574b5b-496b-4c81-acf9-a81fe3d2abb0");
    string<100U> testString("Kernfusionsbaby");
    auto res = testString.find_first_of("cd");
    EXPECT_THAT(res.has_value(), Eq(false));

    res = testString.find_first_of("cd", 0U);
    EXPECT_THAT(res.has_value(), Eq(false));

    res = testString.find_first_of("cd", 50U);
    EXPECT_THAT(res.has_value(), Eq(false));
}

/// @note template <typename T>
/// iox::optional<uint64_t> find_last_of(const T& t, uint64_t pos = 0) const noexcept
TYPED_TEST(stringTyped_test, FindLastOfFailsForEmptyStringInEmptyString)
{
    ::testing::Test::RecordProperty("TEST_ID", "7e09947d-762f-4d7f-a1ab-65696877da06");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    string<STRINGCAP + 5U> testString;
    auto res = this->testSubject.find_last_of(testString);
    EXPECT_THAT(res.has_value(), Eq(false));

    res = this->testSubject.find_last_of("");
    EXPECT_THAT(res.has_value(), Eq(false));
}

TYPED_TEST(stringTyped_test, FindLastOfForStringInEmptyStringFails)
{
    ::testing::Test::RecordProperty("TEST_ID", "ab1650d8-b3f1-445a-b53a-b339f9f37830");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    string<STRINGCAP + 5U> testString("a");
    auto res = this->testSubject.find_last_of(testString);
    EXPECT_THAT(res.has_value(), Eq(false));

    res = this->testSubject.find_last_of("a");
    EXPECT_THAT(res.has_value(), Eq(false));
}

TEST(String100, FindLastOfForStringInNotEmptyStringWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "384b1457-ccaa-4801-a7be-625ef9b7a27a");
    constexpr uint64_t STRINGCAP = 10U;
    constexpr uint64_t SUB_STRINGCAP = 100U;
    string<STRINGCAP> testString("R2-D2");
    string<SUB_STRINGCAP> substring1("2");
    auto res = testString.find_last_of(substring1);
    ASSERT_THAT(res.has_value(), Eq(true));
    EXPECT_THAT(res.value(), Eq(4U));

    res = testString.find_last_of(substring1, 1U);
    ASSERT_THAT(res.has_value(), Eq(true));
    EXPECT_THAT(res.value(), Eq(1U));

    res = testString.find_last_of(substring1, 5U);
    ASSERT_THAT(res.has_value(), Eq(true));
    EXPECT_THAT(res.value(), Eq(4U));

    string<SUB_STRINGCAP> substring2("D3R");
    res = testString.find_last_of(substring2);
    ASSERT_THAT(res.has_value(), Eq(true));
    EXPECT_THAT(res.value(), Eq(3U));

    res = testString.find_last_of(substring2, 1U);
    ASSERT_THAT(res.has_value(), Eq(true));
    EXPECT_THAT(res.value(), Eq(0U));
}

TEST(String100, FindLastOfForNotIncludedStringFails)
{
    ::testing::Test::RecordProperty("TEST_ID", "c0aad222-061f-42ab-8d58-7c7fb5dc1296");
    constexpr uint64_t STRINGCAP = 100U;
    string<STRINGCAP> testString("Kernfusionsbaby");
    string<STRINGCAP> substring("cdG");
    auto res = testString.find_last_of(substring);
    EXPECT_THAT(res.has_value(), Eq(false));

    res = testString.find_last_of(substring, 0U);
    EXPECT_THAT(res.has_value(), Eq(false));

    res = testString.find_last_of(substring, 50U);
    EXPECT_THAT(res.has_value(), Eq(false));
}

TEST(String100, FindLastOfForStringLiteralInNotEmptyStringWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "cac2ed11-6f33-4e32-ac3f-fd725c068f42");
    string<100U> testString1("Mueslimaedchen");
    auto res = testString1.find_last_of("lima");
    ASSERT_THAT(res.has_value(), Eq(true));
    EXPECT_THAT(res.value(), Eq(7U));

    res = testString1.find_last_of("lima", 5U);
    ASSERT_THAT(res.has_value(), Eq(true));
    EXPECT_THAT(res.value(), Eq(5U));

    res = testString1.find_last_of("e", 7U);
    ASSERT_THAT(res.has_value(), Eq(true));
    EXPECT_THAT(res.value(), Eq(2U));

    res = testString1.find_last_of("U3M");
    ASSERT_THAT(res.has_value(), Eq(true));
    EXPECT_THAT(res.value(), Eq(0U));
}

TEST(String100, FindLastOfForNotIncludedStringLiteralFails)
{
    ::testing::Test::RecordProperty("TEST_ID", "ec2e64cf-91ef-4c9c-801d-4a27d26db240");
    string<100U> testString("Kernfusionsbaby");
    auto res = testString.find_last_of("cd");
    EXPECT_THAT(res.has_value(), Eq(false));

    res = testString.find_last_of("cd", 0U);
    EXPECT_THAT(res.has_value(), Eq(false));

    res = testString.find_last_of("cd", 50U);
    EXPECT_THAT(res.has_value(), Eq(false));
}

/// @note constexpr char& at(const uint64_t pos) noexcept
TYPED_TEST(stringTyped_test, AccessPositionOfEmptyStringViaAtFails)
{
    ::testing::Test::RecordProperty("TEST_ID", "89817818-f05a-4ceb-8663-9727d227048c");

    IOX_EXPECT_FATAL_FAILURE([&] { this->testSubject.at(0U); }, iox::er::ENFORCE_VIOLATION);
}

TYPED_TEST(stringTyped_test, AccessPositionOutOfBoundsViaAtFails)
{
    ::testing::Test::RecordProperty("TEST_ID", "68035709-5f8d-4bcb-80ce-ad5619aba84a");

    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString().capacity();

    IOX_EXPECT_FATAL_FAILURE([&] { this->testSubject.at(STRINGCAP); }, iox::er::ENFORCE_VIOLATION);
}

TYPED_TEST(stringTyped_test, AccessFirstPositionOfNonEmptyStringViaAtReturnsCorrectCharacter)
{
    ::testing::Test::RecordProperty("TEST_ID", "8f096ca8-6951-43d4-86a8-3cf94de2977a");
    this->testSubject = "M";
    EXPECT_THAT(this->testSubject.at(0U), Eq('M'));
}

TYPED_TEST(stringTyped_test, AccessAndAssignToMaxPositionOfNotEmptyStringViaAtSucceeds)
{
    ::testing::Test::RecordProperty("TEST_ID", "9b9d106d-bfc5-40fe-876d-5beb1725e3fa");
    constexpr char START_CHARACTER = 'M';
    constexpr char NEW_CHARACTER = 'L';
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString().capacity();

    std::string testSTDString(STRINGCAP, START_CHARACTER);
    ASSERT_THAT(this->testSubject.unsafe_assign(testSTDString.c_str()), Eq(true));

    this->testSubject.at(STRINGCAP - 1) = NEW_CHARACTER;
    testSTDString.at(STRINGCAP - 1) = NEW_CHARACTER;
    EXPECT_THAT(this->testSubject.c_str(), StrEq(testSTDString));
}

/// @note constexpr const char& at(const uint64_t pos) const noexcept
TYPED_TEST(stringTyped_test, AccessPositionOfEmptyStringViaConstAtFails)
{
    ::testing::Test::RecordProperty("TEST_ID", "5cf6d322-6ee9-41ce-bbf6-4e0d193fa938");

    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString().capacity();
    const string<STRINGCAP> sut;

    IOX_EXPECT_FATAL_FAILURE([&] { sut.at(0U); }, iox::er::ENFORCE_VIOLATION);
}

TYPED_TEST(stringTyped_test, AccessPositionOutOfBoundsViaConstAtFails)
{
    ::testing::Test::RecordProperty("TEST_ID", "90a986f4-b29b-4ce7-ad55-79cc4b7b2b29");

    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString().capacity();
    const string<STRINGCAP> sut;

    IOX_EXPECT_FATAL_FAILURE([&] { sut.at(STRINGCAP); }, iox::er::ENFORCE_VIOLATION);
}

TYPED_TEST(stringTyped_test, AccessFirstPositionOfNotEmptyStringViaConstAtReturnsCorrectCharacter)
{
    ::testing::Test::RecordProperty("TEST_ID", "661fb674-3edf-4ac3-adb7-c7767153040d");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString().capacity();
    const string<STRINGCAP> sut("M");
    EXPECT_THAT(sut.at(0U), Eq('M'));
}

TYPED_TEST(stringTyped_test, AccessMaxPositionOfNotEmptyStringViaConstAtSucceeds)
{
    ::testing::Test::RecordProperty("TEST_ID", "78f13867-8633-4d90-9f04-a9a47ab492b1");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString().capacity();

    std::string testSTDString(STRINGCAP, 'M');
    const string<STRINGCAP> sut(TruncateToCapacity, testSTDString.c_str(), testSTDString.size());

    EXPECT_THAT(sut.at(STRINGCAP - 1), Eq('M'));
}

/// @note constexpr char& operator[](const uint64_t pos) noexcept
TYPED_TEST(stringTyped_test, AccessPositionOfEmptyStringViaSubscriptOperatorFails)
{
    ::testing::Test::RecordProperty("TEST_ID", "95ced457-1aec-47e9-a496-0197ea3f4600");

    IOX_EXPECT_FATAL_FAILURE([&] { this->testSubject[0U]; }, iox::er::ENFORCE_VIOLATION);
}

TYPED_TEST(stringTyped_test, AccessPositionOutOfBoundsViaSubscriptOperatorFails)
{
    ::testing::Test::RecordProperty("TEST_ID", "ab52924e-1d6a-41e1-a8a9-8cfd9ab2120d");

    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString().capacity();

    IOX_EXPECT_FATAL_FAILURE([&] { this->testSubject[STRINGCAP]; }, iox::er::ENFORCE_VIOLATION);
}

TYPED_TEST(stringTyped_test, AccessFirstPositionOfNotEmptyStringViaSubscriptOperatorReturnsCorrectCharacter)
{
    ::testing::Test::RecordProperty("TEST_ID", "86a984b9-8345-4413-a25c-e9a1c74c5cef");
    this->testSubject = "L";
    EXPECT_THAT(this->testSubject[0U], Eq('L'));
}

TYPED_TEST(stringTyped_test, AccessAndAssignToMaxPositionOfNotEmptyStringViaSubscriptOperatorSucceeds)
{
    ::testing::Test::RecordProperty("TEST_ID", "9e2dd8ba-6053-4532-895b-f18a9ae0adf6");
    constexpr char START_CHARACTER = 'F';
    constexpr char NEW_CHARACTER = 'S';
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString().capacity();

    std::string testSTDString(STRINGCAP, START_CHARACTER);
    ASSERT_THAT(this->testSubject.unsafe_assign(testSTDString.c_str()), Eq(true));

    this->testSubject[STRINGCAP - 1] = NEW_CHARACTER;
    testSTDString[STRINGCAP - 1] = NEW_CHARACTER;
    EXPECT_THAT(this->testSubject.c_str(), StrEq(testSTDString));
}

/// @note constexpr const char& operator[](const uint64_t pos) const noexcept
TYPED_TEST(stringTyped_test, AccessPositionOfEmptyStringViaConstSubscriptOperatorFails)
{
    ::testing::Test::RecordProperty("TEST_ID", "7ca75e53-8e26-4451-8712-a86bfe5bd32c");

    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString().capacity();
    const string<STRINGCAP> sut;

    IOX_EXPECT_FATAL_FAILURE([&] { sut[0U]; }, iox::er::ENFORCE_VIOLATION);
}

TYPED_TEST(stringTyped_test, AccessPositionOutOfBoundsViaConstSubscriptOperatorFails)
{
    ::testing::Test::RecordProperty("TEST_ID", "5498e314-d321-464a-a667-400ee0c4d81f");

    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString().capacity();
    const string<STRINGCAP> sut;

    IOX_EXPECT_FATAL_FAILURE([&] { sut[STRINGCAP]; }, iox::er::ENFORCE_VIOLATION);
}

TYPED_TEST(stringTyped_test, AccessFirstPositionOfNotEmptyStringViaConstSubscriptOperatorReturnsCorrectCharacter)
{
    ::testing::Test::RecordProperty("TEST_ID", "7ea474cc-db27-43cd-a2f8-dfa2ba196404");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString().capacity();
    const string<STRINGCAP> sut("L");
    EXPECT_THAT(sut[0U], Eq('L'));
}

TYPED_TEST(stringTyped_test, AccessMaxPositionOfNotEmptyStringViaConstSubscriptOperatorSucceeds)
{
    ::testing::Test::RecordProperty("TEST_ID", "43ca0e71-7ac6-4039-a0fb-ebe84f7c0bad");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString().capacity();

    std::string testSTDString(STRINGCAP, 'L');
    const string<STRINGCAP> sut(TruncateToCapacity, testSTDString.c_str(), testSTDString.size());

    EXPECT_THAT(sut[STRINGCAP - 1], Eq('L'));
}

TEST(stringTyped_test, NonCxxStringsAreIdentifiedCorrectly)
{
    ::testing::Test::RecordProperty("TEST_ID", "898fdeb7-2b35-4d33-8db4-ed3b9447a1da");

    EXPECT_FALSE(is_iox_string<int>::value);
    // NOLINTJUSTIFICATION we want test explicitly the c arrays case
    // NOLINTBEGIN(hicpp-avoid-c-arrays,cppcoreguidelines-avoid-c-arrays)
    EXPECT_FALSE(is_iox_string<int[10]>::value);
    EXPECT_FALSE(is_iox_string<char[11]>::value);
    // NOLINTEND(hicpp-avoid-c-arrays,cppcoreguidelines-avoid-c-arrays)
    EXPECT_FALSE(is_iox_string<char>::value);
}

TEST(stringTyped_test, CxxStringsAreIdentifiedCorrectly)
{
    ::testing::Test::RecordProperty("TEST_ID", "778995dc-9be4-47f1-9490-cd111930d3d3");

    EXPECT_TRUE(is_iox_string<iox::string<1>>::value);
    EXPECT_TRUE(is_iox_string<iox::string<10>>::value);
}

TYPED_TEST(stringTyped_test, UncheckedAtWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "2197840d-af88-4fcd-9bb7-51a87d4bf10d");

    for (uint64_t i = 0; i < this->testSubject.capacity(); ++i)
    {
        // add a b c in alternating fashion
        EXPECT_THAT(this->testSubject.unsafe_append(static_cast<char>('a' + i % 3)), Eq(true));
    }

    for (uint64_t i = 0; i < this->testSubject.size(); ++i)
    {
        EXPECT_THAT(this->testSubject.unchecked_at(i), Eq(static_cast<char>('a' + i % 3)));
        // NOLINTJUSTIFICATION we explicitly test the const version
        // NOLINTNEXTLINE(cppcoreguidelines-pro-type-const-cast)
        EXPECT_THAT(const_cast<const decltype(this->testSubject)&>(this->testSubject).unchecked_at(i),
                    Eq(static_cast<char>('a' + i % 3)));
    }
}
} // namespace
