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

/// @note string() noexcept
TYPED_TEST(stringTyped_test, EmptyInitializationResultsInSize0)
{
    ::testing::Test::RecordProperty("TEST_ID", "4a1a3850-b151-4f11-8208-b286e403847d");
    EXPECT_THAT(this->testSubject.size(), Eq(0U));
}

TYPED_TEST(stringTyped_test, EmptyInitializationResultsInEmptyString)
{
    ::testing::Test::RecordProperty("TEST_ID", "b4833ccc-814c-408e-a469-9950550703b9");
    EXPECT_THAT(this->testSubject.c_str(), StrEq(""));
}

/// @note string(const string& other) noexcept
TYPED_TEST(stringTyped_test, CopyConstructEmptyStringResultsInSize0)
{
    ::testing::Test::RecordProperty("TEST_ID", "8a88b069-4c7c-425a-b515-35deeaacaae5");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    string<STRINGCAP> fuu(this->testSubject);
    EXPECT_THAT(fuu.capacity(), Eq(STRINGCAP));
    EXPECT_THAT(fuu.size(), Eq(0U));
    EXPECT_THAT(fuu.c_str(), StrEq(""));
}

TYPED_TEST(stringTyped_test, CopyConstructStringOfSizeCapaResultsInSizeCapa)
{
    ::testing::Test::RecordProperty("TEST_ID", "0d6bc778-8a1e-4eb5-8f87-0ebf45c5116e");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    std::string testString(STRINGCAP, 'M');
    EXPECT_THAT(this->testSubject.unsafe_assign(testString.c_str()), Eq(true));
    string<STRINGCAP> fuu(this->testSubject);
    EXPECT_THAT(fuu.capacity(), Eq(STRINGCAP));
    EXPECT_THAT(fuu.size(), Eq(STRINGCAP));
    EXPECT_THAT(fuu.c_str(), StrEq(testString));
}

/// @note string(string&& other) noexcept
TYPED_TEST(stringTyped_test, MoveConstructionWithStringOfSize0Works)
{
    ::testing::Test::RecordProperty("TEST_ID", "1857ab80-bb27-4e4d-8f11-6ddf94b0cfaf");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    string<STRINGCAP> testString(std::move(this->testSubject));
    EXPECT_THAT(testString.size(), Eq(0));
    EXPECT_THAT(this->testSubject.size(), Eq(0U));
    EXPECT_THAT(testString.c_str(), StrEq(""));
    EXPECT_THAT(this->testSubject.c_str(), StrEq(""));
}

TYPED_TEST(stringTyped_test, MoveConstructionWithStringOfSizeSmallerCapaWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "cb79ff56-d10f-4bb3-85a5-08c1ccc6947a");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    std::string testString(STRINGCAP - 1U, 'M');
    EXPECT_THAT(this->testSubject.unsafe_assign(testString.c_str()), Eq(true));
    string<STRINGCAP> fuu(std::move(this->testSubject));
    EXPECT_THAT(this->testSubject.size(), Eq(0U));
    EXPECT_THAT(fuu.size(), Eq(STRINGCAP - 1U));
    EXPECT_THAT(fuu.c_str(), Eq(testString));
    EXPECT_THAT(this->testSubject.c_str(), StrEq(""));
}

TYPED_TEST(stringTyped_test, MoveConstructionWithStringOfSizeCapaWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "f313dd8a-7dc4-40e3-8203-e099e2a05305");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    std::string testString(STRINGCAP, 'M');
    EXPECT_THAT(this->testSubject.unsafe_assign(testString.c_str()), Eq(true));
    string<STRINGCAP> fuu(std::move(this->testSubject));
    EXPECT_THAT(this->testSubject.size(), Eq(0U));
    EXPECT_THAT(fuu.size(), Eq(STRINGCAP));
    EXPECT_THAT(fuu.c_str(), Eq(testString));
    EXPECT_THAT(this->testSubject.c_str(), StrEq(""));
}

/// @note string& operator=(const string& rhs) noexcept
TYPED_TEST(stringTyped_test, SelfCopyAssignmentExcluded)
{
    ::testing::Test::RecordProperty("TEST_ID", "51b70520-fc11-4948-8fdf-643c638a81b9");
    this->testSubject = "M";
    this->testSubject = this->testSubject;
    EXPECT_THAT(this->testSubject.size(), Eq(1U));
    EXPECT_THAT(this->testSubject.c_str(), StrEq("M"));
}

TYPED_TEST(stringTyped_test, CopyAssignmentWithStringOfSize0Works)
{
    ::testing::Test::RecordProperty("TEST_ID", "fdfca1c5-8293-4e14-beef-519729e71420");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    string<STRINGCAP> fuu;
    fuu = this->testSubject;
    EXPECT_THAT(this->testSubject.size(), Eq(0U));
    EXPECT_THAT(this->testSubject.c_str(), StrEq(""));
    EXPECT_THAT(fuu.size(), Eq(0U));
    EXPECT_THAT(fuu.c_str(), StrEq(""));
}

TYPED_TEST(stringTyped_test, CopyAssignmentWithStringOfSizeSmallerCapaWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "b9b030db-e760-42ab-adbd-311cb5293a2e");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    std::string testString(STRINGCAP - 1U, 'M');
    EXPECT_THAT(this->testSubject.unsafe_assign(testString.c_str()), Eq(true));
    string<STRINGCAP> fuu;
    fuu = this->testSubject;
    EXPECT_THAT(this->testSubject.size(), Eq(STRINGCAP - 1U));
    EXPECT_THAT(this->testSubject.c_str(), StrEq(testString.substr(0U, STRINGCAP - 1U)));
    EXPECT_THAT(fuu.size(), Eq(STRINGCAP - 1U));
    EXPECT_THAT(fuu.c_str(), StrEq(testString.substr(0U, STRINGCAP - 1U)));
}

TYPED_TEST(stringTyped_test, CopyAssignmentWithStringOfSizeCapaWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "10fa4f3d-089c-4d10-83e9-5a37f7d5f0ec");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    std::string testString(STRINGCAP, 'M');
    EXPECT_THAT(this->testSubject.unsafe_assign(testString.c_str()), Eq(true));
    string<STRINGCAP> fuu;
    fuu = this->testSubject;
    EXPECT_THAT(this->testSubject.size(), Eq(STRINGCAP));
    EXPECT_THAT(this->testSubject.c_str(), Eq(testString));
    EXPECT_THAT(fuu.size(), Eq(STRINGCAP));
    EXPECT_THAT(fuu.c_str(), Eq(testString));
}

/// @note string& operator=(string&& rhs) noexcept
TYPED_TEST(stringTyped_test, SelfMoveAssignmentExcluded)
{
    ::testing::Test::RecordProperty("TEST_ID", "0ad45975-b68b-465a-b8c5-83dd8d8290d5");
    this->testSubject = "M";
#if (defined(__GNUC__) && (__GNUC__ >= 13 && __GNUC__ <= 15) && !defined(__clang__))
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wself-move"
#endif
    this->testSubject = std::move(this->testSubject);
#if (defined(__GNUC__) && (__GNUC__ >= 13 && __GNUC__ <= 15) && !defined(__clang__))
#pragma GCC diagnostic pop
#endif
    EXPECT_THAT(this->testSubject.size(), Eq(1U));
    EXPECT_THAT(this->testSubject.c_str(), StrEq("M"));
}

TYPED_TEST(stringTyped_test, MoveAssignmentOfStringWithSize0ResultsInSize0)
{
    ::testing::Test::RecordProperty("TEST_ID", "7170b9d6-ff73-4e3f-a6d1-b66f6de21dcd");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    string<STRINGCAP> fuu;
    fuu = std::move(this->testSubject);
    EXPECT_THAT(this->testSubject.size(), Eq(0U));
    EXPECT_THAT(fuu.size(), Eq(0U));
    EXPECT_THAT(this->testSubject.c_str(), StrEq(""));
    EXPECT_THAT(fuu.c_str(), StrEq(""));
}

TYPED_TEST(stringTyped_test, MoveAssignmentOfStringWithSmallerSizeResultsInSmallerSize)
{
    ::testing::Test::RecordProperty("TEST_ID", "ca455516-5ca6-4882-933b-082769eb975a");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    std::string testString(STRINGCAP - 1U, 'M');
    EXPECT_THAT(this->testSubject.unsafe_assign(testString.c_str()), Eq(true));
    string<STRINGCAP> fuu;
    fuu = std::move(this->testSubject);
    EXPECT_THAT(fuu.size(), Eq(STRINGCAP - 1U));
    EXPECT_THAT(this->testSubject.size(), Eq(0U));
    EXPECT_THAT(fuu.c_str(), Eq(testString));
    EXPECT_THAT(this->testSubject.c_str(), StrEq(""));
}

TYPED_TEST(stringTyped_test, MoveAssignmentOfStringWithSizeCapaResultsInSizeCapa)
{
    ::testing::Test::RecordProperty("TEST_ID", "375944b8-6817-44d4-8a05-de9e0235c503");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    std::string testString(STRINGCAP, 'M');
    EXPECT_THAT(this->testSubject.unsafe_assign(testString.c_str()), Eq(true));
    string<STRINGCAP> fuu;
    fuu = std::move(this->testSubject);
    EXPECT_THAT(fuu.size(), Eq(STRINGCAP));
    EXPECT_THAT(this->testSubject.size(), Eq(0U));
    EXPECT_THAT(fuu.c_str(), Eq(testString));
    EXPECT_THAT(this->testSubject.c_str(), StrEq(""));
}

/// @note template <uint64_t N>
/// string(const string<N>& other) noexcept;
TYPED_TEST(stringTyped_test, ConstrWithEmptyStringWithSmallerCapaWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "94bb789d-a98e-4eaa-8255-029a6d837d7c");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    string<STRINGCAP + 1U> testString(this->testSubject);
    EXPECT_THAT(testString.c_str(), StrEq(""));
    EXPECT_THAT(testString.size(), Eq(0U));
    EXPECT_THAT(testString.capacity(), Eq(STRINGCAP + 1U));
    EXPECT_THAT(this->testSubject.c_str(), StrEq(""));
    EXPECT_THAT(this->testSubject.size(), Eq(0U));
    EXPECT_THAT(this->testSubject.capacity(), Eq(STRINGCAP));
}

TYPED_TEST(stringTyped_test, ConstrWithStringWithSmallerCapaWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "cebe491f-abaa-4473-8417-6c3264620f61");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    this->testSubject = "M";
    string<STRINGCAP + 1U> testString(this->testSubject);
    EXPECT_THAT(testString.c_str(), StrEq("M"));
    EXPECT_THAT(testString.size(), Eq(1U));
    EXPECT_THAT(testString.capacity(), Eq(STRINGCAP + 1U));
    EXPECT_THAT(this->testSubject.c_str(), StrEq("M"));
    EXPECT_THAT(this->testSubject.size(), Eq(1U));
    EXPECT_THAT(this->testSubject.capacity(), Eq(STRINGCAP));
}

/// @note template <uint64_t N>
/// string(string<N>&& other) noexcept;
TYPED_TEST(stringTyped_test, MoveConstrWithEmptyStringWithSmallerCapaWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "76527963-8483-4689-816e-01e6803e078a");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    string<STRINGCAP + 30U> testString(std::move(this->testSubject));
    EXPECT_THAT(testString.size(), Eq(0U));
    EXPECT_THAT(this->testSubject.size(), Eq(0U));
    EXPECT_THAT(testString.c_str(), StrEq(""));
    EXPECT_THAT(this->testSubject.c_str(), StrEq(""));
}

TYPED_TEST(stringTyped_test, MoveConstrWithStringSmallerCapaWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "4bc0b719-080b-4345-9875-a74d0b5cf77d");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    std::string testString(STRINGCAP, 'M');
    EXPECT_THAT(this->testSubject.unsafe_assign(testString.c_str()), Eq(true));
    string<STRINGCAP + 11U> fuu(std::move(this->testSubject));
    EXPECT_THAT(this->testSubject.size(), Eq(0U));
    EXPECT_THAT(fuu.size(), Eq(STRINGCAP));
    EXPECT_THAT(fuu.c_str(), Eq(testString));
    EXPECT_THAT(this->testSubject.c_str(), StrEq(""));
}

/// @note template <uint64_t N>
/// string& operator=(const string<N>& rhs) noexcept;
TYPED_TEST(stringTyped_test, AssignmentOfStringWithSmallerCapaWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "983c0fd2-6224-4712-9704-01367f1cd2c6");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    string<STRINGCAP + 1U> testString;
    testString = this->testSubject;
    EXPECT_THAT(testString.c_str(), StrEq(""));
    EXPECT_THAT(testString.size(), Eq(0U));
    EXPECT_THAT(this->testSubject.c_str(), StrEq(""));
    EXPECT_THAT(this->testSubject.size(), Eq(0U));
}

TYPED_TEST(stringTyped_test, AssignmentOfEmptyStringWithSmallerCapaWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "c2361b47-cc22-458e-b9bc-e295324c3216");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    string<STRINGCAP + 1U> testString("M");
    testString = this->testSubject;
    EXPECT_THAT(testString.c_str(), StrEq(""));
    EXPECT_THAT(testString.size(), Eq(0U));
    EXPECT_THAT(this->testSubject.c_str(), StrEq(""));
    EXPECT_THAT(this->testSubject.size(), Eq(0U));
}

TYPED_TEST(stringTyped_test, AssignmentOfNotEmptyStringWithSmallerCapaWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "66eaa759-fe92-4b94-9c1c-371f2c4fc2bf");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    this->testSubject = "M";
    string<STRINGCAP + 30U> testString("Ferdinand Spitzschnueffler");
    testString = this->testSubject;
    EXPECT_THAT(testString.c_str(), StrEq("M"));
    EXPECT_THAT(testString.size(), Eq(1U));
    EXPECT_THAT(this->testSubject.c_str(), StrEq("M"));
    EXPECT_THAT(this->testSubject.size(), Eq(1U));
}

/// @note template <uint64_t N>
/// string& operator=(string<N>&& rhs) noexcept;
TYPED_TEST(stringTyped_test, MoveAssignmentOfEmptyStringWithSmallerCapaWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "0703614e-88a6-4f5f-80c7-f0d7e51736f6");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    string<STRINGCAP + 63U> fuu;
    fuu = std::move(this->testSubject);
    EXPECT_THAT(this->testSubject.size(), Eq(0U));
    EXPECT_THAT(fuu.size(), Eq(0U));
    EXPECT_THAT(this->testSubject.c_str(), StrEq(""));
    EXPECT_THAT(fuu.c_str(), StrEq(""));
}

TYPED_TEST(stringTyped_test, MoveAssignmentOfStringWithSmallerCapaWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "7c8713cb-d018-448b-9df7-880f1e765474");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    std::string testString(STRINGCAP, 'M');
    EXPECT_THAT(this->testSubject.unsafe_assign(testString.c_str()), Eq(true));
    string<STRINGCAP + 36U> fuu;
    fuu = std::move(this->testSubject);
    EXPECT_THAT(fuu.size(), Eq(STRINGCAP));
    EXPECT_THAT(this->testSubject.size(), Eq(0U));
    EXPECT_THAT(fuu.c_str(), Eq(testString));
    EXPECT_THAT(this->testSubject.c_str(), StrEq(""));
}

TYPED_TEST(stringTyped_test, MoveAssignmentOfNotEmptyStringWithSmallerCapaWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "74811de5-eab2-44b7-9a80-6022e5d90ce0");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    this->testSubject = "M";
    string<STRINGCAP + 30U> testString("Jean-Luc Picard");
    testString = std::move(this->testSubject);
    EXPECT_THAT(testString.c_str(), StrEq("M"));
    EXPECT_THAT(testString.size(), Eq(1U));
    EXPECT_THAT(this->testSubject.c_str(), StrEq(""));
    EXPECT_THAT(this->testSubject.size(), Eq(0U));
}

/// @note template <uint64_t N>
/// string(const char (&other)[N]) noexcept
TYPED_TEST(stringTyped_test, CharToStringConvConstrWithSize0ResultsInSize0)
{
    ::testing::Test::RecordProperty("TEST_ID", "29e9f7b8-a1c7-4462-81d0-52ef34a8b883");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    string<STRINGCAP> fuu("");
    EXPECT_THAT(fuu.capacity(), Eq(STRINGCAP));
    EXPECT_THAT(fuu.size(), Eq(0U));
    EXPECT_THAT(fuu.c_str(), StrEq(""));
}

TYPED_TEST(stringTyped_test, CharToStringConvConstrWithSizeCapaResultsInSizeCapa)
{
    ::testing::Test::RecordProperty("TEST_ID", "de81475a-527e-43e0-97b8-faf7a9300204");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    // required to verify string literal functionality of iox::string
    // NOLINTNEXTLINE(hicpp-avoid-c-arrays, cppcoreguidelines-avoid-c-arrays)
    char testChar[STRINGCAP];
    for (uint64_t i = 0U; i < STRINGCAP - 1U; i++)
    {
        // NOLINTJUSTIFICATION no other way to populate testCharArray
        // NOLINTNEXTLINE(cppcoreguidelines-pro-bounds-constant-array-index)
        testChar[i] = 'M';
    }
    testChar[STRINGCAP - 1U] = '\0';
    string<STRINGCAP> testSubject(testChar);
    EXPECT_THAT(testSubject.capacity(), Eq(STRINGCAP));
    EXPECT_THAT(testSubject.size(), Eq(STRINGCAP - 1U));
    EXPECT_THAT(testSubject.c_str(), StrEq(&testChar[0]));
}

/// @note string(TruncateToCapacity_t, const char* const other) noexcept
TYPED_TEST(stringTyped_test, UnsafeCharToStringConvConstrWithSize0ResultsInSize0)
{
    ::testing::Test::RecordProperty("TEST_ID", "3ed6360e-c4a0-474b-8ed7-e6b4e129a6c0");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    string<STRINGCAP> fuu(TruncateToCapacity, "");
    EXPECT_THAT(fuu.capacity(), Eq(STRINGCAP));
    EXPECT_THAT(fuu.size(), Eq(0U));
    EXPECT_THAT(fuu.c_str(), StrEq(""));
}

TYPED_TEST(stringTyped_test, UnsafeCharToStringConvConstrWithSizeCapaResultsInSizeCapa)
{
    ::testing::Test::RecordProperty("TEST_ID", "5c417b37-ee9d-42f9-bb25-c59c6929d4ca");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    // increase capacity by one to circumvent gcc -Werror=array-bounds
    // required to verify string literal functionality of iox::string
    // NOLINTNEXTLINE(hicpp-avoid-c-arrays, cppcoreguidelines-avoid-c-arrays)
    char testChar[STRINGCAP + 1];
    for (uint64_t i = 0U; i < STRINGCAP - 1U; i++)
    {
        // NOLINTJUSTIFICATION no other way to populate testCharArray
        // NOLINTNEXTLINE(cppcoreguidelines-pro-bounds-constant-array-index)
        testChar[i] = 'M';
    }
    testChar[STRINGCAP - 1U] = '\0';
    string<STRINGCAP> testSubject(TruncateToCapacity, &testChar[0]);
    EXPECT_THAT(testSubject.capacity(), Eq(STRINGCAP));
    EXPECT_THAT(testSubject.size(), Eq(STRINGCAP - 1U));
    EXPECT_THAT(testSubject.c_str(), StrEq(&testChar[0]));
}

TYPED_TEST(stringTyped_test, UnsafeCharToStringConvConstrWithSizeGreaterCapaResultsInSizeCapa)
{
    ::testing::Test::RecordProperty("TEST_ID", "5e0a2023-ea15-43d5-aae8-980a75be6122");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    // required to verify string literal functionality of iox::string
    // NOLINTNEXTLINE(hicpp-avoid-c-arrays, cppcoreguidelines-avoid-c-arrays)
    char testChar[STRINGCAP + 1U];
    for (uint64_t i = 0U; i < STRINGCAP; i++)
    {
        // NOLINTJUSTIFICATION no other way to populate testCharArray
        // NOLINTNEXTLINE(cppcoreguidelines-pro-bounds-constant-array-index)
        testChar[i] = 'M';
    }
    testChar[STRINGCAP] = '\0';
    string<STRINGCAP> testSubject(TruncateToCapacity, &testChar[0]);
    EXPECT_THAT(testSubject.capacity(), Eq(STRINGCAP));
    EXPECT_THAT(testSubject.size(), Eq(STRINGCAP));
}

TYPED_TEST(stringTyped_test, UnsafeCharToStringConvConstrWithNullPtrResultsEmptyString)
{
    ::testing::Test::RecordProperty("TEST_ID", "c6bbcbc6-e049-4c2c-bf84-8d89dcf42ce8");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    string<STRINGCAP> fuu(TruncateToCapacity, nullptr);
    EXPECT_THAT(fuu.capacity(), Eq(STRINGCAP));
    EXPECT_THAT(fuu.size(), Eq(0U));
    EXPECT_THAT(fuu.c_str(), StrEq(""));
}


/// @note string(TruncateToCapacity_t, const char* const other, const uint64_t count) noexcept
TYPED_TEST(stringTyped_test, UnsafeCharToStringConstrWithCount0ResultsInSize0)
{
    ::testing::Test::RecordProperty("TEST_ID", "6cbd9ee8-f015-41ce-8a2c-b8c023b0722a");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    string<STRINGCAP> fuu(TruncateToCapacity, "Yoda", 0U);
    EXPECT_THAT(fuu.capacity(), Eq(STRINGCAP));
    EXPECT_THAT(fuu.size(), Eq(0U));
    EXPECT_THAT(fuu.c_str(), StrEq(""));
}

TYPED_TEST(stringTyped_test, UnsafeCharToStringConstrWithCountEqCapaResultsInSizeCapa)
{
    ::testing::Test::RecordProperty("TEST_ID", "0e62c9fe-78e7-4b6d-8c31-27270dca7581");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    std::string testString(STRINGCAP, 'M');
    string<STRINGCAP> testSubject(TruncateToCapacity, testString.c_str(), STRINGCAP);
    EXPECT_THAT(testSubject.capacity(), Eq(STRINGCAP));
    EXPECT_THAT(testSubject.size(), Eq(STRINGCAP));
    EXPECT_THAT(testSubject.c_str(), StrEq(testString));
}

TYPED_TEST(stringTyped_test, UnsafeCharToStringConstrWithCountGreaterCapaResultsInSizeCapa)
{
    ::testing::Test::RecordProperty("TEST_ID", "d2d321ed-8ee3-4337-a7c9-4785b5ae9a67");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    std::string testString(STRINGCAP + 1U, 'M');
    string<STRINGCAP> testSubject(TruncateToCapacity, testString.c_str(), STRINGCAP + 1U);
    EXPECT_THAT(testSubject.capacity(), Eq(STRINGCAP));
    EXPECT_THAT(testSubject.size(), Eq(STRINGCAP));
    EXPECT_THAT(testSubject.c_str(), StrEq(testString.substr(0U, STRINGCAP)));
}

TEST(String100, UnsafeCharToStringConstrIncludingNullCharWithCountResultsInSizeCount)
{
    ::testing::Test::RecordProperty("TEST_ID", "548eb26e-39b0-4c35-ad80-7665cde80361");
    constexpr uint64_t STRING_CAPACITY = 100U;
    constexpr uint64_t STRING_SIZE = 7U;
    std::string testString{"ice\0ryx", STRING_SIZE};
    string<STRING_CAPACITY> testSubject(TruncateToCapacity, testString.c_str(), STRING_SIZE);
    EXPECT_THAT(testSubject.capacity(), Eq(STRING_CAPACITY));
    EXPECT_THAT(testSubject.size(), Eq(STRING_SIZE));
    // NOLINTNEXTLINE(bugprone-string-literal-with-embedded-nul) this shall be explicitly tested
    EXPECT_THAT(testSubject.c_str(), StrEq("ice\0ryx"));
}

TEST(CharArrayAssignment, AssignCharArrayWithStringSizeLessThanArraySize)
{
    ::testing::Test::RecordProperty("TEST_ID", "886f580d-e57c-4685-90bf-2399737779be");
    constexpr uint64_t STRING_CAPACITY = 20U;
    // required to verify string literal functionality of iox::string
    // NOLINTNEXTLINE(hicpp-avoid-c-arrays, cppcoreguidelines-avoid-c-arrays)
    char testString[STRING_CAPACITY] = "iceoryx";
    string<STRING_CAPACITY> testSubject(testString);
    EXPECT_THAT(testSubject.size(), Eq(7U));
    EXPECT_THAT(testSubject.c_str(), StrEq("iceoryx"));
}

TEST(CharArrayAssignment, AssignZeroTerminatedCharArrayWithSizeForFullCapa)
{
    ::testing::Test::RecordProperty("TEST_ID", "884e724a-f5d3-41d1-89fa-96f55ce99a96");
    constexpr uint64_t STRING_CAPACITY = 7U;
    // required to verify string literal functionality of iox::string
    // NOLINTNEXTLINE(hicpp-avoid-c-arrays, cppcoreguidelines-avoid-c-arrays)
    char testString[STRING_CAPACITY + 1U] = "iceoryx";
    string<STRING_CAPACITY> testSubject(testString);
    EXPECT_THAT(testSubject.size(), Eq(STRING_CAPACITY));
    EXPECT_THAT(testSubject.c_str(), StrEq("iceoryx"));
}

TEST(CharArrayAssignment, AssignNonZeroTerminatedCharArrayOfSizeForFullCapa)
{
    ::testing::Test::RecordProperty("TEST_ID", "2a43553f-4358-4c41-a885-1495de0d7f4f");
    constexpr uint64_t STRING_CAPACITY = 7U;
    // required to verify string literal functionality of iox::string
    // NOLINTNEXTLINE(hicpp-avoid-c-arrays, cppcoreguidelines-avoid-c-arrays)
    char testString[STRING_CAPACITY + 1U] = "iceoryx";
    testString[STRING_CAPACITY] = 'x'; // overwrite the 0 termination
    string<STRING_CAPACITY> testSubject(testString);
    EXPECT_THAT(testSubject.size(), Eq(STRING_CAPACITY));
    EXPECT_THAT(testSubject.c_str(), StrEq("iceoryx"));
}

TYPED_TEST(stringTyped_test, UnsafeCharToStringConstrWithNullPtrResultsEmptyString)
{
    ::testing::Test::RecordProperty("TEST_ID", "cd772d2a-b6db-4b9d-b90d-f2a0aca4aaf6");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    string<STRINGCAP> fuu(TruncateToCapacity, nullptr, STRINGCAP);
    EXPECT_THAT(fuu.capacity(), Eq(STRINGCAP));
    EXPECT_THAT(fuu.size(), Eq(0U));
    EXPECT_THAT(fuu.c_str(), StrEq(""));
}

/// @note template <uint64_t N>
/// string& operator=(const char (&rhs)[N]) noexcept
TYPED_TEST(stringTyped_test, AssignCStringOfSize0WithOperatorResultsInSize0)
{
    ::testing::Test::RecordProperty("TEST_ID", "49d08010-7a31-472d-bd99-6c6e7c49aad5");
    this->testSubject = "";
    EXPECT_THAT(this->testSubject.size(), Eq(0U));
    EXPECT_THAT(this->testSubject.c_str(), StrEq(""));
}

TYPED_TEST(stringTyped_test, AssignCStringOfSizeCapaWithOperatorResultsInSizeCapa)
{
    ::testing::Test::RecordProperty("TEST_ID", "19b0a4af-acfa-4d9b-b432-145ab1e7f59d");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    // required to verify string literal functionality of iox::string
    // NOLINTNEXTLINE(hicpp-avoid-c-arrays, cppcoreguidelines-avoid-c-arrays)
    char testChar[STRINGCAP];
    for (uint64_t i = 0U; i < STRINGCAP - 1U; i++)
    {
        // NOLINTJUSTIFICATION no other way to populate testCharArray
        // NOLINTNEXTLINE(cppcoreguidelines-pro-bounds-constant-array-index)
        testChar[i] = 'M';
    }
    testChar[STRINGCAP - 1U] = '\0';
    string<STRINGCAP> testSubject;
    testSubject = testChar;
    EXPECT_THAT(testSubject.size(), Eq(STRINGCAP - 1U));
    EXPECT_THAT(testSubject.c_str(), StrEq(&testChar[0]));
}

/// @note template <uint64_t N>
/// string& assign(const string& str) noexcept
TYPED_TEST(stringTyped_test, SelfAssignmentIsExcluded)
{
    ::testing::Test::RecordProperty("TEST_ID", "32c07349-3924-4b86-9ce8-21247c8148b9");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    string<STRINGCAP> testSubject;
    testSubject.assign(testSubject);
    EXPECT_THAT(testSubject.size(), Eq(0U));
}

TYPED_TEST(stringTyped_test, AssignStringOfSize0ResultsInSize0)
{
    ::testing::Test::RecordProperty("TEST_ID", "ee981dcb-80ab-439e-a3bf-a52a591b7dbb");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    string<STRINGCAP> fuu;
    this->testSubject.assign(fuu);
    EXPECT_THAT(this->testSubject.size(), Eq(0U));
    EXPECT_THAT(this->testSubject.c_str(), StrEq(""));
    EXPECT_THAT(fuu.size(), Eq(0U));
    EXPECT_THAT(fuu.c_str(), StrEq(""));
}

TYPED_TEST(stringTyped_test, AssignStringOfSizeCapaResultsInSizeCapa)
{
    ::testing::Test::RecordProperty("TEST_ID", "75803631-93e8-4555-84d1-cdb44d88adc4");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    std::string testString(STRINGCAP, 'M');
    string<STRINGCAP> fuu(TruncateToCapacity, testString.c_str(), testString.size());
    this->testSubject.assign(fuu);
    EXPECT_THAT(this->testSubject.size(), Eq(STRINGCAP));
    EXPECT_THAT(this->testSubject.c_str(), StrEq(testString));
    EXPECT_THAT(fuu.size(), Eq(STRINGCAP));
    EXPECT_THAT(fuu.c_str(), StrEq(testString));
}

TYPED_TEST(stringTyped_test, AssignStringOfSize0AndSmallerCapaResultsInSize0)
{
    ::testing::Test::RecordProperty("TEST_ID", "d66ee5d5-03ac-4485-ac73-b8b6f044d38a");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    string<STRINGCAP + 1U> testString;
    testString.assign(this->testSubject);
    EXPECT_THAT(testString.size(), Eq(0U));
    EXPECT_THAT(testString.c_str(), StrEq(""));
    EXPECT_THAT(this->testSubject.size(), Eq(0U));
    EXPECT_THAT(this->testSubject.c_str(), StrEq(""));
}

TYPED_TEST(stringTyped_test, AssignStringWithSmallerCapaWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "d7bdad39-fd7f-4b32-9e3f-60e6cd246bdc");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    std::string testStdString(STRINGCAP, 'M');
    EXPECT_THAT(this->testSubject.unsafe_assign(testStdString.c_str()), Eq(true));
    string<STRINGCAP + 1U> testString;
    testString.assign(this->testSubject);
    EXPECT_THAT(testString.size(), Eq(STRINGCAP));
    EXPECT_THAT(testString.c_str(), StrEq(testStdString));
    EXPECT_THAT(this->testSubject.size(), Eq(STRINGCAP));
    EXPECT_THAT(this->testSubject.c_str(), StrEq(testStdString));
}

/// @note template <uint64_t N>
/// string& assign(const char (&str)[N]) noexcept
TYPED_TEST(stringTyped_test, FreshlyAssignNothingResultsInZeroSize)
{
    ::testing::Test::RecordProperty("TEST_ID", "c0df23ee-86f8-4821-b577-38bad21c2e77");
    this->testSubject.assign("");
    EXPECT_THAT(this->testSubject.size(), Eq(0U));
}

TYPED_TEST(stringTyped_test, ReassignNothingResultsInZeroSize)
{
    ::testing::Test::RecordProperty("TEST_ID", "854089ee-bf0f-432c-96ce-20e83e09181c");
    this->testSubject.assign("M");
    this->testSubject.assign("");
    EXPECT_THAT(this->testSubject.size(), Eq(0U));
}

TYPED_TEST(stringTyped_test, AssignCStringOfSizeCapaResultsInSizeCapa)
{
    ::testing::Test::RecordProperty("TEST_ID", "25f4f306-2303-4cc8-a42b-d0cbb600d833");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    // required to verify string literal functionality of iox::string
    // NOLINTNEXTLINE(hicpp-avoid-c-arrays, cppcoreguidelines-avoid-c-arrays)
    char testChar[STRINGCAP];
    for (uint64_t i = 0U; i < STRINGCAP - 1U; i++)
    {
        // NOLINTJUSTIFICATION no other way to populate testCharArray
        // NOLINTNEXTLINE(cppcoreguidelines-pro-bounds-constant-array-index)
        testChar[i] = 'M';
    }
    testChar[STRINGCAP - 1U] = '\0';
    string<STRINGCAP> testSubject;
    testSubject.assign(testChar);
    EXPECT_THAT(testSubject.size(), Eq(STRINGCAP - 1U));
    EXPECT_THAT(testSubject.c_str(), StrEq(&testChar[0]));
}

/// @note bool unsafe_assign(const char* const str) noexcept
TYPED_TEST(stringTyped_test, UnsafeAssignOfCStringOfSize0ResultsInSize0)
{
    ::testing::Test::RecordProperty("TEST_ID", "861aa3a2-4fa3-401a-b6af-2ddbb3139f69");
    EXPECT_THAT(this->testSubject.unsafe_assign(""), Eq(true));
    EXPECT_THAT(this->testSubject.size(), Eq(0U));
    EXPECT_THAT(this->testSubject.c_str(), StrEq(""));
}

TYPED_TEST(stringTyped_test, UnsafeAssignOfCStringOfSize1ResultsInSize1)
{
    ::testing::Test::RecordProperty("TEST_ID", "7b09a56c-1706-43b9-85d0-6c8bdf79f1b2");
    EXPECT_THAT(this->testSubject.unsafe_assign("M"), Eq(true));
    EXPECT_THAT(this->testSubject.size(), Eq(1U));
    EXPECT_THAT(this->testSubject.c_str(), StrEq("M"));
}

TYPED_TEST(stringTyped_test, UnsafeAssignCStringOfSizeCapaResultsInSizeCapa)
{
    ::testing::Test::RecordProperty("TEST_ID", "422a0400-0862-441f-a7d0-20272856800f");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    std::vector<char> testCharstring(STRINGCAP, 'M');
#if (defined(__GNUC__) && __GNUC__ >= 7 && !defined(__clang__))
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Warray-bounds"
#pragma GCC diagnostic ignored "-Wstringop-overflow"
#endif
    testCharstring.emplace_back('\0');
#if (defined(__GNUC__) && __GNUC__ >= 7 && !defined(__clang__))
#pragma GCC diagnostic pop
#endif
    EXPECT_THAT(this->testSubject.unsafe_assign(testCharstring.data()), Eq(true));
    EXPECT_THAT(this->testSubject.size(), Eq(STRINGCAP));
}

TYPED_TEST(stringTyped_test, UnsafeAssignCStringOfSizeGreaterCapaResultsInSize0)
{
    ::testing::Test::RecordProperty("TEST_ID", "6012209b-87f8-4796-b9ec-336881cf7e24");
    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    std::vector<char> testCharstring(STRINGCAP + 1U, 'M');
#if (defined(__GNUC__) && __GNUC__ >= 7 && !defined(__clang__))
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Warray-bounds"
#pragma GCC diagnostic ignored "-Wstringop-overflow"
#endif
    testCharstring.emplace_back('\0');
#if (defined(__GNUC__) && __GNUC__ >= 7 && !defined(__clang__))
#pragma GCC diagnostic pop
#endif
    EXPECT_THAT(this->testSubject.unsafe_assign(testCharstring.data()), Eq(false));
    EXPECT_THAT(this->testSubject.size(), Eq(0U));
    EXPECT_THAT(this->testSubject.c_str(), StrEq(""));
}

TYPED_TEST(stringTyped_test, UnsafeAssignOfInvalidCStringFails)
{
    ::testing::Test::RecordProperty("TEST_ID", "863d38e5-5c5f-4a78-a2ca-155205bb9ecc");
    this->testSubject = "L";

    using MyString = typename TestFixture::stringType;
    constexpr auto STRINGCAP = MyString::capacity();
    std::vector<char> testCharstring(STRINGCAP + 1U, 'M');
#if (defined(__GNUC__) && __GNUC__ >= 7 && !defined(__clang__))
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Warray-bounds"
#pragma GCC diagnostic ignored "-Wstringop-overflow"
#endif
    testCharstring.emplace_back('\0');
#if (defined(__GNUC__) && __GNUC__ >= 7 && !defined(__clang__))
#pragma GCC diagnostic pop
#endif

    EXPECT_THAT(this->testSubject.unsafe_assign(testCharstring.data()), Eq(false));
    EXPECT_THAT(this->testSubject.size(), Eq(1U));
    EXPECT_THAT(this->testSubject.c_str(), StrEq("L"));
}

TYPED_TEST(stringTyped_test, UnsafeAssignOfCharPointerPointingToSameAddress)
{
    ::testing::Test::RecordProperty("TEST_ID", "a129add2-4d8e-4953-ba28-b78ece5a2f02");
    this->testSubject = "M";
    const char* fuu = this->testSubject.c_str();
    EXPECT_THAT(this->testSubject.unsafe_assign(fuu), Eq(false));
}

TYPED_TEST(stringTyped_test, UnsafeAssignOfNullptrFails)
{
    ::testing::Test::RecordProperty("TEST_ID", "140e5402-c6b5-4a07-a0f7-2a10f6d307fb");
    EXPECT_THAT(this->testSubject.unsafe_assign(nullptr), Eq(false));
}
} // namespace
