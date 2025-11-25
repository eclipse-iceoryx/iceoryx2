// Copyright (c) 2023 by Apex.AI Inc. All rights reserved.
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

#include "iceoryx_hoofs/testing/fatal_failure.hpp"
#include "iceoryx_platform/platform_settings.hpp"
#include "iox/detail/hoofs_error_reporting.hpp"
#include "iox/file_name.hpp"
#include "iox/file_path.hpp"
#include "iox/group_name.hpp"
#include "iox/path.hpp"
#include "iox/semantic_string.hpp"
#include "iox/user_name.hpp"
#include "test.hpp"

#include <string>
#include <vector>

namespace
{
using namespace ::testing;
using namespace iox;
using namespace iox::testing;

template <typename T>
struct TestValues
{
    static const uint64_t CAPACITY;
    static const std::vector<std::string> VALID_VALUES;
    static const std::vector<std::string> INVALID_CHARACTER_VALUES;
    static const std::vector<std::string> INVALID_CONTENT_VALUES;
    static const std::vector<std::string> TOO_LONG_CONTENT_VALUES;
    static const std::string GREATER_VALID_VALUE;
    static const std::string SMALLER_VALID_VALUE;
    static const std::string MAX_CAPACITY_VALUE;
    static const std::vector<std::string> ADD_VALID_CHARS_TO_CREATE_INVALID_CONTENT_AT_BEGIN;
    static const std::vector<std::string> ADD_VALID_CHARS_TO_CREATE_INVALID_CONTENT_AT_END;
};

///////////////////
// START: UserName
///////////////////
template <>
const uint64_t TestValues<UserName>::CAPACITY = platform::MAX_USER_NAME_LENGTH;
template <>
const std::vector<std::string> TestValues<UserName>::VALID_VALUES{{"some-user"}, {"user2"}};
template <>
const std::vector<std::string> TestValues<UserName>::INVALID_CHARACTER_VALUES{
    {"some-!user"}, {"*kasjd"}, {"_fuuuas"}, {"asd/asd"}, {";'1'fuuuu"}, {"argh/"}, {"fuu/arg/bla"}};
template <>
const std::vector<std::string> TestValues<UserName>::INVALID_CONTENT_VALUES{
    {""}, {"-do-not-start-with-dash"}, {"5do-not-start-with-a-number"}};
template <>
const std::vector<std::string> TestValues<UserName>::TOO_LONG_CONTENT_VALUES{{"i-am-waaaaay-toooooooo-loooooooong"}};
template <>
const std::string TestValues<UserName>::GREATER_VALID_VALUE{"zebra-zusel"};
template <>
const std::string TestValues<UserName>::SMALLER_VALID_VALUE{"alfons-alf"};
template <>
const std::string TestValues<UserName>::MAX_CAPACITY_VALUE{"all-glory-to-the-incredible-and-legendary-hypno-toad"};
template <>
const std::vector<std::string> TestValues<UserName>::ADD_VALID_CHARS_TO_CREATE_INVALID_CONTENT_AT_BEGIN{"-bla",
                                                                                                        "81923"};
template <>
const std::vector<std::string> TestValues<UserName>::ADD_VALID_CHARS_TO_CREATE_INVALID_CONTENT_AT_END{};
///////////////////
// END: UserName
///////////////////

///////////////////
// START: GroupName
///////////////////
template <>
const uint64_t TestValues<GroupName>::CAPACITY = platform::MAX_GROUP_NAME_LENGTH;
template <>
const std::vector<std::string> TestValues<GroupName>::VALID_VALUES{{"a-group"}, {"group2"}};
template <>
const std::vector<std::string> TestValues<GroupName>::INVALID_CHARACTER_VALUES{
    {"se-!ur"}, {"*kad"}, {"_fus"}, {"a/sd"}, {";'1'fu"}, {"ah/"}, {"fuu/bla"}};
template <>
const std::vector<std::string> TestValues<GroupName>::INVALID_CONTENT_VALUES{{""}, {"-no-dash"}, {"5no-number"}};
template <>
const std::vector<std::string> TestValues<GroupName>::TOO_LONG_CONTENT_VALUES{{"i-am-waaaaay-toooooooo-loooooooong"}};
template <>
const std::string TestValues<GroupName>::GREATER_VALID_VALUE{"zebra-zusel"};
template <>
const std::string TestValues<GroupName>::SMALLER_VALID_VALUE{"alfons-alf"};
template <>
const std::string TestValues<GroupName>::MAX_CAPACITY_VALUE{"all-glory-to-the-incredible-and-legendary-hypno-toad"};
template <>
const std::vector<std::string> TestValues<GroupName>::ADD_VALID_CHARS_TO_CREATE_INVALID_CONTENT_AT_BEGIN{"-fuu",
                                                                                                         "8number"};
template <>
const std::vector<std::string> TestValues<GroupName>::ADD_VALID_CHARS_TO_CREATE_INVALID_CONTENT_AT_END{};
///////////////////
// END: GroupName
///////////////////

///////////////////
// START: FileName
///////////////////
template <>
const uint64_t TestValues<FileName>::CAPACITY = platform::IOX_MAX_FILENAME_LENGTH;
template <>
const std::vector<std::string> TestValues<FileName>::VALID_VALUES{
    {"file"}, {"another_file.bla"}, {"123.456"}, {".hidden_me"}};
template <>
const std::vector<std::string> TestValues<FileName>::INVALID_CHARACTER_VALUES{
    {"some-!user"}, {"*kasjd"}, {"$_fuuuas"}, {"asd/asd"}, {";'1'fuuuu"}, {"argh/"}, {"fuu/arg/bla"}};
template <>
const std::vector<std::string> TestValues<FileName>::INVALID_CONTENT_VALUES{{""}, {"."}, {".."}};
template <>
const std::vector<std::string> TestValues<FileName>::TOO_LONG_CONTENT_VALUES{
    std::string(platform::IOX_MAX_FILENAME_LENGTH + 2, 'a')};
template <>
const std::string TestValues<FileName>::GREATER_VALID_VALUE{"9-i-am-a-file"};
template <>
const std::string TestValues<FileName>::SMALLER_VALID_VALUE{"0.me.too.be.file"};
template <>
const std::string TestValues<FileName>::MAX_CAPACITY_VALUE{std::string(platform::IOX_MAX_FILENAME_LENGTH, 'b')};
template <>
const std::vector<std::string> TestValues<FileName>::ADD_VALID_CHARS_TO_CREATE_INVALID_CONTENT_AT_BEGIN{};
template <>
const std::vector<std::string> TestValues<FileName>::ADD_VALID_CHARS_TO_CREATE_INVALID_CONTENT_AT_END{};
///////////////////
// END: FileName
///////////////////

///////////////////
// START: FilePath
///////////////////
template <>
const uint64_t TestValues<FilePath>::CAPACITY = platform::IOX_MAX_PATH_LENGTH;
template <>
const std::vector<std::string> TestValues<FilePath>::VALID_VALUES{{"file"},
                                                                  {"another_file.bla"},
                                                                  {"123.456"},
                                                                  {".hidden_me"},
                                                                  {"/some/file/path"},
                                                                  {"./relative/path"},
                                                                  {"another/../../relative/path"},
                                                                  {"another/../...bla"},
                                                                  {"not/yet/another/path/../fuu"}};
template <>
const std::vector<std::string> TestValues<FilePath>::INVALID_CHARACTER_VALUES{{"some-!user"},
                                                                              {"*kasjd"},
                                                                              {"$_fuuuas"},
                                                                              {";'1'fuuuu"},
                                                                              {"so*me/path/to/."},
                                                                              {"/some/pa)th/to/."},
                                                                              {"another/relative/pa]th/at/the/end/.."}};
template <>
const std::vector<std::string> TestValues<FilePath>::INVALID_CONTENT_VALUES{
    {""}, {"."}, {".."}, {"stop/with/relative/.."}, "another/relative/part/at/the/end/."};
template <>
const std::vector<std::string> TestValues<FilePath>::TOO_LONG_CONTENT_VALUES{
    std::string(platform::IOX_MAX_PATH_LENGTH + 2, 'a')};
template <>
const std::string TestValues<FilePath>::GREATER_VALID_VALUE{"9-i-am-a-file"};
template <>
const std::string TestValues<FilePath>::SMALLER_VALID_VALUE{"0.me.too.be.file"};
template <>
const std::string TestValues<FilePath>::MAX_CAPACITY_VALUE{std::string(platform::IOX_MAX_PATH_LENGTH, 'b')};
template <>
const std::vector<std::string> TestValues<FilePath>::ADD_VALID_CHARS_TO_CREATE_INVALID_CONTENT_AT_BEGIN{};
template <>
const std::vector<std::string> TestValues<FilePath>::ADD_VALID_CHARS_TO_CREATE_INVALID_CONTENT_AT_END{};
///////////////////
// END: FilePath
///////////////////

///////////////////
// START: Path
///////////////////
template <>
const uint64_t TestValues<Path>::CAPACITY = platform::IOX_MAX_PATH_LENGTH;
template <>
const std::vector<std::string> TestValues<Path>::VALID_VALUES{{"file"},
                                                              {"another_file.bla"},
                                                              {"123.456"},
                                                              {".hidden_me"},
                                                              {"/some/file/path"},
                                                              {"./relative/path"},
                                                              {"another/../../relative/path"},
                                                              {"another/../...bla"},
                                                              {"not/yet/another/path/../fuu"},
                                                              {"/slash/at/the/end/"},
                                                              {"../relative/path/at/the/end/.."},
                                                              "relative_path/at/end2/."};
template <>
const std::vector<std::string> TestValues<Path>::INVALID_CHARACTER_VALUES{
    {"some-!user"},
    {"*kasjd"},
    {"$_fuuuas"},
    {";'1'fuuuu"},
    {"so*me/path/to/.*"},
    {"another/relative/character]th/at/the/end/#$!*"}};
template <>
const std::vector<std::string> TestValues<Path>::INVALID_CONTENT_VALUES{};
template <>
const std::vector<std::string> TestValues<Path>::TOO_LONG_CONTENT_VALUES{
    std::string(platform::IOX_MAX_PATH_LENGTH + 2, 'a')};
template <>
const std::string TestValues<Path>::GREATER_VALID_VALUE{"9-i-am-a-file/blubb/di/whoop"};
template <>
const std::string TestValues<Path>::SMALLER_VALID_VALUE{"0.me.too.be.file/whoop/whoop"};
template <>
const std::string TestValues<Path>::MAX_CAPACITY_VALUE{std::string(platform::IOX_MAX_PATH_LENGTH, 'b')};
template <>
const std::vector<std::string> TestValues<Path>::ADD_VALID_CHARS_TO_CREATE_INVALID_CONTENT_AT_BEGIN{};
template <>
const std::vector<std::string> TestValues<Path>::ADD_VALID_CHARS_TO_CREATE_INVALID_CONTENT_AT_END{};
///////////////////
// END: Path
///////////////////


template <typename T>
class SemanticString_test : public Test
{
  protected:
    void SetUp() override
    {
        EXPECT_FALSE(TestValues<T>::VALID_VALUES.empty());
        EXPECT_FALSE(TestValues<T>::TOO_LONG_CONTENT_VALUES.empty());
        EXPECT_FALSE(TestValues<T>::GREATER_VALID_VALUE.empty());
        EXPECT_FALSE(TestValues<T>::SMALLER_VALID_VALUE.empty());
        // Greater since not all platforms have the same capacity. The value will be truncated when the
        // capacity is smaller.
        EXPECT_TRUE(TestValues<T>::MAX_CAPACITY_VALUE >= TestValues<T>::MAX_CAPACITY_VALUE);
        // we left out INVALID_CHARACTER_VALUES & INVALID_CONTENT_VALUES since a SemanticString can
        // have only invalid characters or only invalid content or neither of both
    }

    using SutType = T;
    string<SutType::capacity()> greater_value_str =
        string<SutType::capacity()>(TruncateToCapacity, TestValues<SutType>::GREATER_VALID_VALUE.c_str());
    string<SutType::capacity()> smaller_value_str =
        string<SutType::capacity()>(TruncateToCapacity, TestValues<SutType>::SMALLER_VALID_VALUE.c_str());

    SutType greater_value = SutType::create(greater_value_str).expect("Failed to create test string.");
    SutType smaller_value = SutType::create(smaller_value_str).expect("Failed to create test string.");
};

using Implementations = Types<UserName, FileName, GroupName, FilePath, Path>;

TYPED_TEST_SUITE(SemanticString_test, Implementations, );

TYPED_TEST(SemanticString_test, InitializeWithValidStringLiteralWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "31a2cd17-ca02-486a-b173-3f1f219d8ca3");
    using SutType = typename TestFixture::SutType;

    auto sut = SutType::create("alwaysvalid");

    ASSERT_THAT(sut.has_error(), Eq(false));
    EXPECT_THAT(sut->size(), Eq(11));
    EXPECT_THAT(sut->capacity(), Eq(TestValues<SutType>::CAPACITY));
    EXPECT_TRUE(sut->as_string() == "alwaysvalid");
}

TYPED_TEST(SemanticString_test, SizeWorksCorrectly)
{
    ::testing::Test::RecordProperty("TEST_ID", "26cc39ac-84c6-45cf-b221-b6db7d210c44");
    using SutType = typename TestFixture::SutType;

    auto test_string =
        string<SutType::capacity()>(TruncateToCapacity, TestValues<SutType>::GREATER_VALID_VALUE.c_str());
    auto sut = SutType::create(test_string);

    ASSERT_THAT(sut.has_error(), Eq(false));
    EXPECT_THAT(sut->size(), Eq(test_string.size()));
}

TYPED_TEST(SemanticString_test, AsStringWorksCorrectly)
{
    ::testing::Test::RecordProperty("TEST_ID", "c4d721d2-0cf8-41d6-a3fe-fbc4b19e9b10");
    using SutType = typename TestFixture::SutType;

    auto test_string =
        string<SutType::capacity()>(TruncateToCapacity, TestValues<SutType>::SMALLER_VALID_VALUE.c_str());
    auto sut = SutType::create(test_string);

    ASSERT_THAT(sut.has_error(), Eq(false));
    EXPECT_THAT(sut->as_string().c_str(), StrEq(test_string.c_str()));
}

TYPED_TEST(SemanticString_test, CapacityWorksCorrectly)
{
    ::testing::Test::RecordProperty("TEST_ID", "d8f6eb13-8f2c-496f-901d-734ee22d85e3");
    using SutType = typename TestFixture::SutType;

    ASSERT_THAT(SutType::capacity(), Eq(TestValues<SutType>::CAPACITY));
}

TYPED_TEST(SemanticString_test, CanBeFilledUpToMaxCapacity)
{
    ::testing::Test::RecordProperty("TEST_ID", "c5ed0595-380c-4caa-a392-a8d2933646d9");
    using SutType = typename TestFixture::SutType;

    auto test_string = string<SutType::capacity()>(TruncateToCapacity, TestValues<SutType>::MAX_CAPACITY_VALUE.c_str());
    auto sut = SutType::create(test_string);

    ASSERT_THAT(sut.has_error(), Eq(false));
    EXPECT_THAT(sut->as_string().c_str(), StrEq(test_string.c_str()));
}

TYPED_TEST(SemanticString_test, InitializeWithValidStringValueWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "0100d764-628c-44ad-9af7-fe7a4540491a");
    using SutType = typename TestFixture::SutType;

    for (const auto& value : TestValues<UserName>::VALID_VALUES)
    {
        auto sut = SutType::create(string<SutType::capacity()>(TruncateToCapacity, value.c_str()));

        ASSERT_THAT(sut.has_error(), Eq(false));
        EXPECT_THAT(sut->size(), Eq(value.size()));
        EXPECT_THAT(sut->capacity(), Eq(TestValues<SutType>::CAPACITY));
        EXPECT_THAT(sut->as_string().c_str(), StrEq(value));
    }
}

TYPED_TEST(SemanticString_test, InitializeWithStringContainingIllegalCharactersFails)
{
    ::testing::Test::RecordProperty("TEST_ID", "14483f4e-d556-4770-89df-84d873428eee");
    using SutType = typename TestFixture::SutType;

    for (auto& value : TestValues<SutType>::INVALID_CHARACTER_VALUES)
    {
        auto sut = SutType::create(string<SutType::capacity()>(TruncateToCapacity, value.c_str()));

        ASSERT_THAT(sut.has_error(), Eq(true));
        ASSERT_THAT(sut.error(), Eq(SemanticStringError::ContainsInvalidCharacters));
    }
}

TYPED_TEST(SemanticString_test, InitializeWithStringContainingIllegalContentFails)
{
    ::testing::Test::RecordProperty("TEST_ID", "9380f932-527f-4116-bd4f-dc8078b63330");
    using SutType = typename TestFixture::SutType;

    for (auto& value : TestValues<SutType>::INVALID_CONTENT_VALUES)
    {
        auto sut = SutType::create(string<SutType::capacity()>(TruncateToCapacity, value.c_str()));

        ASSERT_THAT(sut.has_error(), Eq(true));
        ASSERT_THAT(sut.error(), Eq(SemanticStringError::ContainsInvalidContent));
    }
}

TYPED_TEST(SemanticString_test, InitializeWithTooLongContentFails)
{
    ::testing::Test::RecordProperty("TEST_ID", "b5597825-c559-48e7-96f3-5136fffc55d7");
    using SutType = typename TestFixture::SutType;

    for (auto& value : TestValues<SutType>::TOO_LONG_CONTENT_VALUES)
    {
        auto sut = SutType::create(string<SutType::capacity() * 2>(TruncateToCapacity, value.c_str()));

        ASSERT_THAT(sut.has_error(), Eq(true));
        ASSERT_THAT(sut.error(), Eq(SemanticStringError::ExceedsMaximumLength));
    }
}

TYPED_TEST(SemanticString_test, AppendValidContentToValidStringWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "0994fccc-5baa-4408-b17e-e2955439608d");
    using SutType = typename TestFixture::SutType;

    for (auto& value : TestValues<SutType>::VALID_VALUES)
    {
        for (auto& add_value : TestValues<SutType>::VALID_VALUES)
        {
            auto sut = SutType::create(string<SutType::capacity()>(TruncateToCapacity, value.c_str()));
            ASSERT_THAT(sut.has_error(), Eq(false));

            EXPECT_THAT(sut->append(string<SutType::capacity()>(TruncateToCapacity, add_value.c_str())).has_error(),
                        Eq(false));
            auto result_size = value.size() + add_value.size();
            EXPECT_THAT(sut->size(), result_size);
            EXPECT_THAT(sut->capacity(), Eq(TestValues<SutType>::CAPACITY));

            auto result = value + add_value;
            EXPECT_THAT(sut->as_string().c_str(), StrEq(result));
        }
    }
}

TYPED_TEST(SemanticString_test, AppendInvalidCharactersToValidStringFails)
{
    ::testing::Test::RecordProperty("TEST_ID", "fddf4a56-c368-4ff0-8727-e732d6ebc87f");
    using SutType = typename TestFixture::SutType;

    for (auto& value : TestValues<SutType>::VALID_VALUES)
    {
        for (auto& invalid_value : TestValues<SutType>::INVALID_CHARACTER_VALUES)
        {
            auto sut = SutType::create(string<SutType::capacity()>(TruncateToCapacity, value.c_str()));
            ASSERT_THAT(sut.has_error(), Eq(false));

            auto result = sut->append(string<SutType::capacity()>(TruncateToCapacity, invalid_value.c_str()));
            ASSERT_TRUE(result.has_error());
            EXPECT_THAT(result.error(), Eq(SemanticStringError::ContainsInvalidCharacters));
            EXPECT_THAT(sut->size(), value.size());
            EXPECT_THAT(sut->capacity(), Eq(TestValues<SutType>::CAPACITY));

            EXPECT_THAT(sut->as_string().c_str(), StrEq(value));
        }
    }
}

TYPED_TEST(SemanticString_test, GenerateInvalidContentWithAppend)
{
    ::testing::Test::RecordProperty("TEST_ID", "a416c7c6-eaff-4e5e-8945-fe9f2d06ee6d");
    using SutType = typename TestFixture::SutType;

    for (auto& value : TestValues<SutType>::VALID_VALUES)
    {
        for (auto& invalid_value : TestValues<SutType>::ADD_VALID_CHARS_TO_CREATE_INVALID_CONTENT_AT_END)
        {
            auto sut = SutType::create(string<SutType::capacity()>(TruncateToCapacity, value.c_str()));
            ASSERT_THAT(sut.has_error(), Eq(false));

            auto result = sut->append(string<SutType::capacity()>(TruncateToCapacity, invalid_value.c_str()));
            ASSERT_TRUE(result.has_error());
            EXPECT_THAT(result.error(), Eq(SemanticStringError::ContainsInvalidContent));
            EXPECT_THAT(sut->size(), value.size());
            EXPECT_THAT(sut->capacity(), Eq(TestValues<SutType>::CAPACITY));

            EXPECT_THAT(sut->as_string().c_str(), StrEq(value));
        }
    }
}

TYPED_TEST(SemanticString_test, GenerateInvalidContentWithInsert)
{
    ::testing::Test::RecordProperty("TEST_ID", "e7db87d3-2574-4b5f-9c3e-c103e05a6b46");
    using SutType = typename TestFixture::SutType;

    for (auto& value : TestValues<SutType>::VALID_VALUES)
    {
        for (auto& invalid_value : TestValues<SutType>::ADD_VALID_CHARS_TO_CREATE_INVALID_CONTENT_AT_BEGIN)
        {
            auto sut = SutType::create(string<SutType::capacity()>(TruncateToCapacity, value.c_str()));
            ASSERT_THAT(sut.has_error(), Eq(false));

            auto result = sut->insert(
                0, string<SutType::capacity()>(TruncateToCapacity, invalid_value.c_str()), invalid_value.size());
            ASSERT_TRUE(result.has_error());
            EXPECT_THAT(result.error(), Eq(SemanticStringError::ContainsInvalidContent));
            EXPECT_THAT(sut->size(), value.size());
            EXPECT_THAT(sut->capacity(), Eq(TestValues<SutType>::CAPACITY));

            EXPECT_THAT(sut->as_string().c_str(), StrEq(value));
        }
    }
}

TYPED_TEST(SemanticString_test, AppendTooLongContentToValidStringFails)
{
    ::testing::Test::RecordProperty("TEST_ID", "b8616fbf-601d-43b9-b4a3-f6b96acdf555");
    using SutType = typename TestFixture::SutType;

    for (auto& value : TestValues<SutType>::VALID_VALUES)
    {
        for (auto& invalid_value : TestValues<SutType>::TOO_LONG_CONTENT_VALUES)
        {
            auto sut = SutType::create(string<SutType::capacity()>(TruncateToCapacity, value.c_str()));
            ASSERT_THAT(sut.has_error(), Eq(false));

            EXPECT_THAT(sut->append(string<SutType::capacity()>(TruncateToCapacity, invalid_value.c_str())).has_error(),
                        Eq(true));
            EXPECT_THAT(sut->size(), Eq(value.size()));
            EXPECT_THAT(sut->capacity(), Eq(TestValues<SutType>::CAPACITY));

            EXPECT_THAT(sut->as_string().c_str(), StrEq(value));
        }
    }
}

TYPED_TEST(SemanticString_test, InsertValidContentToValidStringWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "56ea499f-5ac3-4ffe-abea-b56194cfd728");
    using SutType = typename TestFixture::SutType;

    for (auto& value : TestValues<SutType>::VALID_VALUES)
    {
        for (auto& add_value : TestValues<SutType>::VALID_VALUES)
        {
            for (size_t insert_position = 0; insert_position < value.size(); ++insert_position)
            {
                auto sut = SutType::create(string<SutType::capacity()>(TruncateToCapacity, value.c_str()));
                ASSERT_THAT(sut.has_error(), Eq(false));

                EXPECT_THAT(sut->insert(insert_position,
                                        string<SutType::capacity()>(TruncateToCapacity, add_value.c_str()),
                                        add_value.size())
                                .has_error(),
                            Eq(false));


                EXPECT_THAT(sut->size(), Eq(value.size() + add_value.size()));
                EXPECT_THAT(sut->capacity(), Eq(TestValues<SutType>::CAPACITY));

                auto result = value;
                result.insert(insert_position, add_value);
                EXPECT_THAT(sut->as_string().c_str(), StrEq(result));
            }
        }
    }
}

TYPED_TEST(SemanticString_test, InsertInvalidCharactersToValidStringFails)
{
    ::testing::Test::RecordProperty("TEST_ID", "35229fb8-e6e9-44d9-9d47-d00b71a4ce01");
    using SutType = typename TestFixture::SutType;

    for (auto& value : TestValues<SutType>::VALID_VALUES)
    {
        for (auto& add_value : TestValues<SutType>::INVALID_CHARACTER_VALUES)
        {
            for (uint64_t insert_position = 0; insert_position < value.size(); ++insert_position)
            {
                auto sut = SutType::create(string<SutType::capacity()>(TruncateToCapacity, value.c_str()));
                ASSERT_THAT(sut.has_error(), Eq(false));

                auto result = sut->insert(insert_position,
                                          string<SutType::capacity()>(TruncateToCapacity, add_value.c_str()),
                                          add_value.size());
                ASSERT_TRUE(result.has_error());
                EXPECT_THAT(result.error(), Eq(SemanticStringError::ContainsInvalidCharacters));


                EXPECT_THAT(sut->size(), value.size());
                EXPECT_THAT(sut->capacity(), Eq(TestValues<SutType>::CAPACITY));
                EXPECT_THAT(sut->as_string().c_str(), StrEq(value));
            }
        }
    }
}

TYPED_TEST(SemanticString_test, InsertTooLongContentToValidStringFails)
{
    ::testing::Test::RecordProperty("TEST_ID", "b6939126-a878-4d7f-9fea-c2b438226e65");
    using SutType = typename TestFixture::SutType;

    for (auto& value : TestValues<SutType>::VALID_VALUES)
    {
        for (auto& add_value : TestValues<SutType>::TOO_LONG_CONTENT_VALUES)
        {
            for (uint64_t insert_position = 0; insert_position < value.size(); ++insert_position)
            {
                auto sut = SutType::create(string<SutType::capacity()>(TruncateToCapacity, value.c_str()));
                ASSERT_THAT(sut.has_error(), Eq(false));

                EXPECT_THAT(sut->insert(insert_position,
                                        string<SutType::capacity()>(TruncateToCapacity, add_value.c_str()),
                                        add_value.size())
                                .has_error(),
                            Eq(true));


                EXPECT_THAT(sut->size(), Eq(value.size()));
                EXPECT_THAT(sut->capacity(), Eq(TestValues<SutType>::CAPACITY));

                EXPECT_THAT(sut->as_string().c_str(), StrEq(value));
            }
        }
    }
}

TYPED_TEST(SemanticString_test, EqualityOperatorWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "97889932-ac3b-4155-9958-34c843d2d323");

    EXPECT_TRUE(this->greater_value == this->greater_value);
    EXPECT_FALSE(this->greater_value == this->smaller_value);

    EXPECT_TRUE(this->greater_value == this->greater_value_str);
    EXPECT_FALSE(this->greater_value == this->smaller_value_str);
}

TYPED_TEST(SemanticString_test, InequalityOperatorWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "32903b0b-3594-4c00-9869-d18e1dfc773f");

    EXPECT_FALSE(this->greater_value != this->greater_value);
    EXPECT_TRUE(this->greater_value != this->smaller_value);

    EXPECT_FALSE(this->greater_value != this->greater_value_str);
    EXPECT_TRUE(this->greater_value != this->smaller_value_str);
}

TYPED_TEST(SemanticString_test, LessThanOrEqualOperatorWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "53f5b765-b462-4cc1-bab7-9b937fbbcecf");

    EXPECT_TRUE(this->greater_value <= this->greater_value);
    EXPECT_TRUE(this->smaller_value <= this->greater_value);
    EXPECT_FALSE(this->greater_value <= this->smaller_value);
}

TYPED_TEST(SemanticString_test, LessThanOperatorWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "cea977a4-ccb3-42a6-9d13-e09dce24c273");

    EXPECT_FALSE(this->greater_value < this->greater_value);
    EXPECT_TRUE(this->smaller_value < this->greater_value);
    EXPECT_FALSE(this->greater_value < this->smaller_value);
}

TYPED_TEST(SemanticString_test, GreaterThanOrEqualOperatorWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "5d731b17-f787-46fc-b64d-3d86c9102008");

    EXPECT_TRUE(this->greater_value >= this->greater_value);
    EXPECT_FALSE(this->smaller_value >= this->greater_value);
    EXPECT_TRUE(this->greater_value >= this->smaller_value);
}

TYPED_TEST(SemanticString_test, GreaterThanOperatorWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "8c046cff-fb69-43b4-9a45-e86f17c874db");

    EXPECT_FALSE(this->greater_value > this->greater_value);
    EXPECT_FALSE(this->smaller_value > this->greater_value);
    EXPECT_TRUE(this->greater_value > this->smaller_value);
}
} // namespace
