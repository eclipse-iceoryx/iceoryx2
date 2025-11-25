// Copyright (c) 2020 by Robert Bosch GmbH. All rights reserved.
// Copyright (c) 2021 - 2022 by Apex.AI Inc. All rights reserved.
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

#include "iox/string.hpp"
#include "iox/type_traits.hpp"
#include "test.hpp"


namespace iox
{
namespace test
{
template <typename, typename = void>
struct has_mytype_as_member : std::false_type
{
};

template <typename T>
struct has_mytype_as_member<T, void_t<typename T::myType>> : std::true_type
{
};
} // namespace test
} // namespace iox

namespace
{
using namespace ::testing;
using namespace iox;

TEST(TypeTraitsTest, IsInvocableResolvesToTrue)
{
    ::testing::Test::RecordProperty("TEST_ID", "802f0044-ee40-47b7-9b83-519866c63508");
    auto lambda = [](int) -> void {};
    auto sut = is_invocable<decltype(lambda), int>::value;
    EXPECT_TRUE(sut);
}

TEST(TypeTraitsTest, IsInvocableResolvesToFalse)
{
    ::testing::Test::RecordProperty("TEST_ID", "c862c84f-d31b-4060-9e11-3a4d850c59f2");
    int beeblebrox{42};
    auto sut = is_invocable<decltype(beeblebrox), void>::value;
    EXPECT_FALSE(sut);
}

TEST(TypeTraitsTest, IsInvocableRResolvesToTrue)
{
    ::testing::Test::RecordProperty("TEST_ID", "15f2d85e-a68f-4a3c-93bd-8b30e87903dc");
    auto lambda = [](int foo) -> int { return foo++; };
    auto sut = is_invocable_r<int, decltype(lambda), int>::value;
    EXPECT_TRUE(sut);
}

TEST(TypeTraitsTest, IsInvocableRResolvesToFalse)
{
    ::testing::Test::RecordProperty("TEST_ID", "ae967e0c-7f55-435a-8161-bd0bc7ada6f7");
    auto lambda = [](float foo) -> float { return foo++; };
    auto sut = is_invocable_r<void, decltype(lambda), int>::value;
    EXPECT_FALSE(sut);
}

TEST(TypeTraitsTest, NoTypeAsMemberIsFalse)
{
    ::testing::Test::RecordProperty("TEST_ID", "244b424c-98da-4da5-a793-3bd3606acc01");
    struct Sut
    {
    };

    EXPECT_FALSE(test::has_mytype_as_member<Sut>::value);
}

TEST(TypeTraitsTest, MyTypeAsMemberIsTrue)
{
    ::testing::Test::RecordProperty("TEST_ID", "8b233e3a-f9c2-4f6a-8ed4-0ace56894576");
    struct Sut
    {
        using myType = int;
    };

    EXPECT_TRUE(test::has_mytype_as_member<Sut>::value);
}

TEST(TypeTraitsTest, AddConstConditionallyAddsConstIfConditionTypeIsConst)
{
    ::testing::Test::RecordProperty("TEST_ID", "021cb188-8d85-46e2-8e35-5916daf43ad3");
    using SutType = uint8_t;
    using ConditionType = bool;

    using SutTypeResult = iox::add_const_conditionally<SutType, const ConditionType>::type;

    EXPECT_TRUE(std::is_const<SutTypeResult>::value);
    // EXPECT_TRUE macro is broken when std::is_same is used directly
    auto typesIsNotChanged = std::is_same<SutType, std::remove_const_t<SutTypeResult>>::value;
    EXPECT_TRUE(typesIsNotChanged);
}

TEST(TypeTraitsTest, AddConstConditionallyDoesNotAddsConstIfConditionTypeIsNotConst)
{
    ::testing::Test::RecordProperty("TEST_ID", "01a7a26f-e988-4cd1-867b-88002623097c");
    using SutType = uint8_t;
    using ConditionType = bool;

    using SutTypeResult = iox::add_const_conditionally<SutType, ConditionType>::type;

    EXPECT_FALSE(std::is_const<SutTypeResult>::value);
    // EXPECT_TRUE macro is broken when std::is_same is used directly
    auto typesIsNotChanged = std::is_same<SutType, SutTypeResult>::value;
    EXPECT_TRUE(typesIsNotChanged);
}

TEST(TypeTraitsTest, AddConstConditionallyTypeAliasWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "0034c4c7-80d1-45c0-bc02-a3e89ea13d45");
    using SutType = uint8_t;
    using ConditionType = bool;

    using SutTypeResult = iox::add_const_conditionally_t<SutType, const ConditionType>;

    EXPECT_TRUE(std::is_const<SutTypeResult>::value);
}

TEST(TypeTraitsTest, AlwaysFalseWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "41ba2959-d7ed-45fa-b2bb-467bbf7cbb38");
    struct Foo
    {
    };
    EXPECT_FALSE(always_false_v<Foo>);
}

TEST(TypeTraitsTest, IsFunctionPointerResolvesToTrue)
{
    ::testing::Test::RecordProperty("TEST_ID", "d2106163-92c3-4263-a706-b2f5bd17866a");
    auto result = is_function_pointer<void (*)(double)>::value;
    EXPECT_TRUE(result);

    result = is_function_pointer<int* (*)(double)>::value;
    EXPECT_TRUE(result);

    result = is_function_pointer<void (*)(int, double)>::value;
    EXPECT_TRUE(result);
}

TEST(TypeTraitsTest, IsFunctionPointerResolvesToFalse)
{
    ::testing::Test::RecordProperty("TEST_ID", "9801a871-f27e-4c96-831c-f826b62feac3");
    auto result = is_function_pointer<int*>::value;
    EXPECT_FALSE(result);

    result = is_function_pointer<void*>::value;
    EXPECT_FALSE(result);

    result = is_function_pointer<int>::value;
    EXPECT_FALSE(result);
}

TEST(TypeTraitsTest, TypeInfo_StringTypeTranslatesCorrectly)
{
    ::testing::Test::RecordProperty("TEST_ID", "e20e6698-3c0c-4b28-a8bb-f5c5dd05a107");
    EXPECT_THAT(TypeInfo<iox::string<1>>::NAME, StrEq("string"));
    EXPECT_THAT(TypeInfo<iox::string<123>>::NAME, StrEq("string"));
}

TEST(TypeTraitsTest, TypeInfo_int8_tTranslatesCorrectly)
{
    ::testing::Test::RecordProperty("TEST_ID", "50fca418-002f-43c6-8899-61377b43b96a");
    EXPECT_THAT(TypeInfo<int8_t>::NAME, StrEq("int8_t"));
}

TEST(TypeTraitsTest, TypeInfo_int16_tTranslatesCorrectly)
{
    ::testing::Test::RecordProperty("TEST_ID", "bd0e39dc-7950-4f55-a284-1265570e3e46");
    EXPECT_THAT(TypeInfo<int16_t>::NAME, StrEq("int16_t"));
}

TEST(TypeTraitsTest, TypeInfo_int32_tTranslatesCorrectly)
{
    ::testing::Test::RecordProperty("TEST_ID", "0c988179-dd46-4d42-98cc-b12ca3702518");
    EXPECT_THAT(TypeInfo<int32_t>::NAME, StrEq("int32_t"));
}

TEST(TypeTraitsTest, TypeInfo_int64_tTranslatesCorrectly)
{
    ::testing::Test::RecordProperty("TEST_ID", "2c5fceb1-2dd3-4133-b968-d0509d04e3d7");
    EXPECT_THAT(TypeInfo<int64_t>::NAME, StrEq("int64_t"));
}

TEST(TypeTraitsTest, TypeInfo_uint8_tTranslatesCorrectly)
{
    ::testing::Test::RecordProperty("TEST_ID", "a14d787b-ca2c-4e14-b61d-4c7dea7e7c7a");
    EXPECT_THAT(TypeInfo<uint8_t>::NAME, StrEq("uint8_t"));
}

TEST(TypeTraitsTest, TypeInfo_uint16_tTranslatesCorrectly)
{
    ::testing::Test::RecordProperty("TEST_ID", "581a078a-2541-47b7-a929-29a46e38cee9");
    EXPECT_THAT(TypeInfo<uint16_t>::NAME, StrEq("uint16_t"));
}

TEST(TypeTraitsTest, TypeInfo_uint32_tTranslatesCorrectly)
{
    ::testing::Test::RecordProperty("TEST_ID", "a0012e71-8f5b-4979-a56b-400533236c8a");
    EXPECT_THAT(TypeInfo<uint32_t>::NAME, StrEq("uint32_t"));
}

TEST(TypeTraitsTest, TypeInfo_uint64_tTranslatesCorrectly)
{
    ::testing::Test::RecordProperty("TEST_ID", "6c9d41b0-9e5a-45a1-830d-bf46507a0000");
    EXPECT_THAT(TypeInfo<uint64_t>::NAME, StrEq("uint64_t"));
}

TEST(TypeTraitsTest, TypeInfo_boolTranslatesCorrectly)
{
    ::testing::Test::RecordProperty("TEST_ID", "7506bc90-1447-48d9-ae91-621d0f4c1db2");
    EXPECT_THAT(TypeInfo<bool>::NAME, StrEq("bool"));
}

TEST(TypeTraitsTest, TypeInfo_charTranslatesCorrectly)
{
    ::testing::Test::RecordProperty("TEST_ID", "2aa53aa6-b2b0-4a78-bb58-1cabfb695e8b");
    EXPECT_THAT(TypeInfo<char>::NAME, StrEq("char"));
}

TEST(TypeTraitsTest, TypeInfo_floatTranslatesCorrectly)
{
    ::testing::Test::RecordProperty("TEST_ID", "b90f78d7-7b1f-49c1-ad90-4d5e706c63ae");
    EXPECT_THAT(TypeInfo<float>::NAME, StrEq("float"));
}

TEST(TypeTraitsTest, TypeInfo_doubleTranslatesCorrectly)
{
    ::testing::Test::RecordProperty("TEST_ID", "222baf0b-ae93-4c25-9244-18d4451a7e4f");
    EXPECT_THAT(TypeInfo<double>::NAME, StrEq("double"));
}

TEST(TypeTraitsTest, TypeInfo_long_doubleTranslatesCorrectly)
{
    ::testing::Test::RecordProperty("TEST_ID", "49fd3664-9d03-48b8-9c61-8f700c51194d");
    EXPECT_THAT(TypeInfo<long double>::NAME, StrEq("long double"));
}

TEST(TypeTraitsTest, NonCharArraysAreIdentifiedCorrectly)
{
    ::testing::Test::RecordProperty("TEST_ID", "40359de0-2ccd-422a-b1d4-da4b4f12a172");

    EXPECT_FALSE(is_char_array<int>::value);
    // NOLINTJUSTIFICATION we want test explicitly the c arrays case
    // NOLINTNEXTLINE(hicpp-avoid-c-arrays,cppcoreguidelines-avoid-c-arrays)
    EXPECT_FALSE(is_char_array<int[10]>::value);
    EXPECT_FALSE(is_char_array<iox::string<11>>::value);
    EXPECT_FALSE(is_char_array<char>::value);
}

TEST(TypeTraitsTest, CharArraysAreIdentifiedCorrectly)
{
    ::testing::Test::RecordProperty("TEST_ID", "e1c115d9-80c4-4bc9-97d0-338112dfe1d3");

    // NOLINTJUSTIFICATION we want test explicitly the c arrays case
    // NOLINTBEGIN(hicpp-avoid-c-arrays,cppcoreguidelines-avoid-c-arrays)
    EXPECT_TRUE(is_char_array<char[1]>::value);
    EXPECT_TRUE(is_char_array<char[10]>::value);
    // NOLINTEND(hicpp-avoid-c-arrays,cppcoreguidelines-avoid-c-arrays)
}
} // namespace
