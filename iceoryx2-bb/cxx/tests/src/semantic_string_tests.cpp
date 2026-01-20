// Copyright (c) 2023 by Apex.AI Inc. All rights reserved.
// Copyright (c) 2025 Contributors to the Eclipse Foundation
//
// See the NOTICE file(s) distributed with this work for additional
// information regarding copyright ownership.
//
// This program and the accompanying materials are made available under the
// terms of the Apache Software License 2.0 which is available at
// https://www.apache.org/licenses/LICENSE-2.0, or the MIT license
// which is available at https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

#include "iox2/bb/file_name.hpp"
#include "iox2/bb/file_path.hpp"
#include "iox2/bb/path.hpp"
#include "iox2/bb/semantic_string.hpp"
#include "iox2/bb/static_string.hpp"

#include <gmock/gmock.h>
#include <gtest/gtest.h>

#include <string>
#include <vector>

namespace {
using namespace ::testing;
using namespace iox2::bb;
using namespace iox2::bb::platform;
using namespace iox2::legacy;

template <typename T>
struct TestValues {
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

// NOLINTBEGIN(cert-err58-cpp): Ignore "initialization with static storage duration may throw an exception that cannot be caught" in tests

///////////////////
// START: FileName
///////////////////
template <>
const uint64_t TestValues<FileName>::CAPACITY = IOX2_MAX_FILENAME_LENGTH; // NOLINT: false positive unused variable
template <>
const std::vector<std::string> TestValues<FileName>::VALID_VALUES {
    { "file" }, { "another_file.bla" }, { "123.456" }, { ".hidden_me" }
};
template <>
const std::vector<std::string> TestValues<FileName>::INVALID_CHARACTER_VALUES {
    { "some-!user" }, { "*kasjd" }, { "$_fuuuas" }, { "asd/asd" }, { ";'1'fuuuu" }, { "argh/" }, { "fuu/arg/bla" }
};
template <>
const std::vector<std::string> TestValues<FileName>::INVALID_CONTENT_VALUES { { "" }, { "." }, { ".." } };
template <>
const std::vector<std::string> TestValues<FileName>::TOO_LONG_CONTENT_VALUES { std::string(IOX2_MAX_FILENAME_LENGTH + 2,
                                                                                           'a') };
template <>
const std::string TestValues<FileName>::GREATER_VALID_VALUE { "9-i-am-a-file" };
template <>
const std::string TestValues<FileName>::SMALLER_VALID_VALUE { "0.me.too.be.file" };
template <>
const std::string TestValues<FileName>::MAX_CAPACITY_VALUE { std::string(IOX2_MAX_FILENAME_LENGTH, 'b') };
template <>
const std::vector<std::string> TestValues<FileName>::ADD_VALID_CHARS_TO_CREATE_INVALID_CONTENT_AT_BEGIN {};
template <>
const std::vector<std::string> TestValues<FileName>::ADD_VALID_CHARS_TO_CREATE_INVALID_CONTENT_AT_END {};
///////////////////
// END: FileName
///////////////////

///////////////////
// START: FilePath
///////////////////
template <>
const uint64_t TestValues<FilePath>::CAPACITY = IOX2_MAX_PATH_LENGTH; // NOLINT: false positive unused variable
template <>
const std::vector<std::string> TestValues<FilePath>::VALID_VALUES { { "file" },
                                                                    { "another_file.bla" },
                                                                    { "123.456" },
                                                                    { ".hidden_me" },
                                                                    { "/some/file/path" },
                                                                    { "another/../../relative/path" },
                                                                    { "another/../...bla" },
                                                                    { "not/yet/another/path/../fuu" } };
template <>
const std::vector<std::string> TestValues<FilePath>::INVALID_CHARACTER_VALUES {
    { "some-!user" },
    { "*kasjd" },
    { "$_fuuuas" },
    { ";'1'fuuuu" },
    { "so*me/path/to/." },
    { "/some/pa)th/to/." },
    { "another/relative/pa]th/at/the/end/.." }
};
template <>
const std::vector<std::string> TestValues<FilePath>::INVALID_CONTENT_VALUES {
    { "" }, { "." }, { ".." }, { "stop/with/relative/.." }, "another/relative/part/at/the/end/."
};
template <>
const std::vector<std::string> TestValues<FilePath>::TOO_LONG_CONTENT_VALUES { std::string(IOX2_MAX_PATH_LENGTH + 2,
                                                                                           'a') };
template <>
const std::string TestValues<FilePath>::GREATER_VALID_VALUE { "9-i-am-a-file" };
template <>
const std::string TestValues<FilePath>::SMALLER_VALID_VALUE { "0.me.too.be.file" };
template <>
const std::string TestValues<FilePath>::MAX_CAPACITY_VALUE { std::string(IOX2_MAX_PATH_LENGTH, 'b') };
template <>
const std::vector<std::string> TestValues<FilePath>::ADD_VALID_CHARS_TO_CREATE_INVALID_CONTENT_AT_BEGIN {};
template <>
const std::vector<std::string> TestValues<FilePath>::ADD_VALID_CHARS_TO_CREATE_INVALID_CONTENT_AT_END {};
///////////////////
// END: FilePath
///////////////////

///////////////////
// START: Path
///////////////////
template <>
const uint64_t TestValues<Path>::CAPACITY = IOX2_MAX_PATH_LENGTH; // NOLINT: false positive unused variable
template <>
const std::vector<std::string> TestValues<Path>::VALID_VALUES { { "file" },
                                                                { "another_file.bla" },
                                                                { "123.456" },
                                                                { ".hidden_me" },
                                                                { "/some/file/path" },
                                                                { "./relative/path" },
                                                                { "another/../../relative/path" },
                                                                { "another/../...bla" },
                                                                { "not/yet/another/path/../fuu" },
                                                                { "/slash/at/the/end/" },
                                                                { "../relative/path/at/the/end/.." },
                                                                "relative_path/at/end2/." };
template <>
const std::vector<std::string> TestValues<Path>::INVALID_CHARACTER_VALUES {
    { "some-!user" }, { "*kasjd" },           { "$_fuuuas" },
    { ";'1'fuuuu" },  { "so*me/path/to/.*" }, { "another/relative/character]th/at/the/end/#$!*" }
};
template <>
const std::vector<std::string> TestValues<Path>::INVALID_CONTENT_VALUES {};
template <>
const std::vector<std::string> TestValues<Path>::TOO_LONG_CONTENT_VALUES { std::string(IOX2_MAX_PATH_LENGTH + 2, 'a') };
template <>
const std::string TestValues<Path>::GREATER_VALID_VALUE { "9-i-am-a-file/blubb/di/whoop" };
template <>
const std::string TestValues<Path>::SMALLER_VALID_VALUE { "0.me.too.be.file/whoop/whoop" };
template <>
const std::string TestValues<Path>::MAX_CAPACITY_VALUE { std::string(IOX2_MAX_PATH_LENGTH, 'b') };
template <>
const std::vector<std::string> TestValues<Path>::ADD_VALID_CHARS_TO_CREATE_INVALID_CONTENT_AT_BEGIN {};
template <>
const std::vector<std::string> TestValues<Path>::ADD_VALID_CHARS_TO_CREATE_INVALID_CONTENT_AT_END {};
///////////////////
// END: Path
///////////////////

// NOLINTEND(cert-err58-cpp)

template <typename T>
class SemanticStringFixture : public Test {
  public:
    void SetUp() override {
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
    // NOLINTBEGIN(misc-non-private-member-variables-in-classes)
    StaticString<SutType::capacity()> greater_value_str =
        *StaticString<SutType::capacity()>::from_utf8_null_terminated_unchecked(
            TestValues<SutType>::GREATER_VALID_VALUE.c_str());
    StaticString<SutType::capacity()> smaller_value_str =
        *StaticString<SutType::capacity()>::from_utf8_null_terminated_unchecked(
            TestValues<SutType>::SMALLER_VALID_VALUE.c_str());

    SutType greater_value = SutType::create(greater_value_str).value();
    SutType smaller_value = SutType::create(smaller_value_str).value();
    // NOLINTEND(misc-non-private-member-variables-in-classes)
};

using Implementations = Types<FileName, FilePath, Path>;

TYPED_TEST_SUITE(SemanticStringFixture, Implementations, );

TYPED_TEST(SemanticStringFixture, initialize_with_valid_string_literal_works) {
    ::testing::Test::RecordProperty("TEST_ID", "31a2cd17-ca02-486a-b173-3f1f219d8ca3");
    using SutType = typename TestFixture::SutType;

    auto sut = SutType::create("alwaysvalid");

    ASSERT_THAT(sut.has_value(), Eq(true));
    EXPECT_THAT(sut->size(), Eq(11));
    EXPECT_THAT(sut->capacity(), Eq(TestValues<SutType>::CAPACITY));
    EXPECT_STREQ(sut->as_string().unchecked_access().c_str(), "alwaysvalid");
}

TYPED_TEST(SemanticStringFixture, size_works_correctly) {
    ::testing::Test::RecordProperty("TEST_ID", "26cc39ac-84c6-45cf-b221-b6db7d210c44");
    using SutType = typename TestFixture::SutType;

    auto test_string = *StaticString<SutType::capacity()>::from_utf8_null_terminated_unchecked(
        TestValues<SutType>::GREATER_VALID_VALUE.c_str());
    auto sut = SutType::create(test_string);

    ASSERT_THAT(sut.has_value(), Eq(true));
    EXPECT_THAT(sut->size(), Eq(test_string.size()));
}

TYPED_TEST(SemanticStringFixture, as_string_works_correctly) {
    ::testing::Test::RecordProperty("TEST_ID", "c4d721d2-0cf8-41d6-a3fe-fbc4b19e9b10");
    using SutType = typename TestFixture::SutType;

    auto test_string = *StaticString<SutType::capacity()>::from_utf8_null_terminated_unchecked(
        TestValues<SutType>::SMALLER_VALID_VALUE.c_str());
    auto sut = SutType::create(test_string);

    ASSERT_THAT(sut.has_value(), Eq(true));
    EXPECT_THAT(sut->as_string().unchecked_access().c_str(), StrEq(test_string.unchecked_access().c_str()));
}

TYPED_TEST(SemanticStringFixture, capacity_works_correctly) {
    ::testing::Test::RecordProperty("TEST_ID", "d8f6eb13-8f2c-496f-901d-734ee22d85e3");
    using SutType = typename TestFixture::SutType;

    ASSERT_THAT(SutType::capacity(), Eq(TestValues<SutType>::CAPACITY));
}

TYPED_TEST(SemanticStringFixture, can_be_filled_up_to_max_capacity) {
    ::testing::Test::RecordProperty("TEST_ID", "c5ed0595-380c-4caa-a392-a8d2933646d9");
    using SutType = typename TestFixture::SutType;

    auto test_string = *StaticString<SutType::capacity()>::from_utf8_null_terminated_unchecked(
        TestValues<SutType>::MAX_CAPACITY_VALUE.c_str());
    auto sut = SutType::create(test_string);

    ASSERT_THAT(sut.has_value(), Eq(true));
    EXPECT_THAT(sut->as_string().unchecked_access().c_str(), StrEq(test_string.unchecked_access().c_str()));
}

TYPED_TEST(SemanticStringFixture, initialize_with_valid_string_value_works) {
    ::testing::Test::RecordProperty("TEST_ID", "0100d764-628c-44ad-9af7-fe7a4540491a");
    using SutType = typename TestFixture::SutType;

    for (const auto& value : TestValues<SutType>::VALID_VALUES) {
        auto sut =
            SutType::create(*StaticString<SutType::capacity()>::from_utf8_null_terminated_unchecked(value.c_str()));

        ASSERT_THAT(sut.has_value(), Eq(true));
        EXPECT_THAT(sut->size(), Eq(value.size()));
        EXPECT_THAT(sut->capacity(), Eq(TestValues<SutType>::CAPACITY));
        EXPECT_THAT(sut->as_string().unchecked_access().c_str(), StrEq(value));
    }
}

TYPED_TEST(SemanticStringFixture, initialize_with_string_containing_illegal_characters_fails) {
    ::testing::Test::RecordProperty("TEST_ID", "14483f4e-d556-4770-89df-84d873428eee");
    using SutType = typename TestFixture::SutType;

    for (auto& value : TestValues<SutType>::INVALID_CHARACTER_VALUES) {
        auto sut =
            SutType::create(*StaticString<SutType::capacity()>::from_utf8_null_terminated_unchecked(value.c_str()));

        ASSERT_THAT(sut.has_value(), Eq(false));
        ASSERT_THAT(sut.error(), Eq(SemanticStringError::InvalidContent));
    }
}

TYPED_TEST(SemanticStringFixture, initialize_with_string_containing_illegal_content_fails) {
    ::testing::Test::RecordProperty("TEST_ID", "9380f932-527f-4116-bd4f-dc8078b63330");
    using SutType = typename TestFixture::SutType;

    for (auto& value : TestValues<SutType>::INVALID_CONTENT_VALUES) {
        auto sut =
            SutType::create(*StaticString<SutType::capacity()>::from_utf8_null_terminated_unchecked(value.c_str()));

        ASSERT_THAT(sut.has_value(), Eq(false));
        ASSERT_THAT(sut.error(), Eq(SemanticStringError::InvalidContent));
    }
}

TYPED_TEST(SemanticStringFixture, initialize_with_too_long_content_fails) {
    ::testing::Test::RecordProperty("TEST_ID", "b5597825-c559-48e7-96f3-5136fffc55d7");
    using SutType = typename TestFixture::SutType;

    for (auto& value : TestValues<SutType>::TOO_LONG_CONTENT_VALUES) {
        auto sut =
            SutType::create(*StaticString<SutType::capacity() * 2>::from_utf8_null_terminated_unchecked(value.c_str()));

        ASSERT_THAT(sut.has_value(), Eq(false));
        ASSERT_THAT(sut.error(), Eq(SemanticStringError::ExceedsMaximumLength));
    }
}

// NOLINTBEGIN(readability-function-cognitive-complexity)
TYPED_TEST(SemanticStringFixture, append_valid_content_to_valid_string_works) {
    ::testing::Test::RecordProperty("TEST_ID", "0994fccc-5baa-4408-b17e-e2955439608d");
    using SutType = typename TestFixture::SutType;

    for (auto& value : TestValues<SutType>::VALID_VALUES) {
        for (auto& add_value : TestValues<SutType>::VALID_VALUES) {
            auto sut =
                SutType::create(*StaticString<SutType::capacity()>::from_utf8_null_terminated_unchecked(value.c_str()));
            ASSERT_THAT(sut.has_value(), Eq(true));

            EXPECT_THAT(
                sut->append(*StaticString<SutType::capacity()>::from_utf8_null_terminated_unchecked(add_value.c_str()))
                    .has_value(),
                Eq(true));
            auto result_size = value.size() + add_value.size();
            EXPECT_THAT(sut->size(), result_size);
            EXPECT_THAT(sut->capacity(), Eq(TestValues<SutType>::CAPACITY));

            auto result = value + add_value;
            EXPECT_THAT(sut->as_string().unchecked_access().c_str(), StrEq(result));
        }
    }
}

TYPED_TEST(SemanticStringFixture, append_invalid_characters_to_valid_string_fails) {
    ::testing::Test::RecordProperty("TEST_ID", "fddf4a56-c368-4ff0-8727-e732d6ebc87f");
    using SutType = typename TestFixture::SutType;

    for (auto& value : TestValues<SutType>::VALID_VALUES) {
        for (auto& invalid_value : TestValues<SutType>::INVALID_CHARACTER_VALUES) {
            auto sut =
                SutType::create(*StaticString<SutType::capacity()>::from_utf8_null_terminated_unchecked(value.c_str()));
            ASSERT_THAT(sut.has_value(), Eq(true));

            auto result = sut->append(
                *StaticString<SutType::capacity()>::from_utf8_null_terminated_unchecked(invalid_value.c_str()));
            ASSERT_FALSE(result.has_value());
            EXPECT_THAT(result.error(), Eq(SemanticStringError::InvalidContent));
            EXPECT_THAT(sut->size(), value.size());
            EXPECT_THAT(sut->capacity(), Eq(TestValues<SutType>::CAPACITY));

            EXPECT_THAT(sut->as_string().unchecked_access().c_str(), StrEq(value));
        }
    }
}

TYPED_TEST(SemanticStringFixture, generate_invalid_content_with_append) {
    ::testing::Test::RecordProperty("TEST_ID", "a416c7c6-eaff-4e5e-8945-fe9f2d06ee6d");
    using SutType = typename TestFixture::SutType;

    for (auto& value : TestValues<SutType>::VALID_VALUES) {
        for (auto& invalid_value : TestValues<SutType>::ADD_VALID_CHARS_TO_CREATE_INVALID_CONTENT_AT_END) {
            auto sut =
                SutType::create(*StaticString<SutType::capacity()>::from_utf8_null_terminated_unchecked(value.c_str()));
            ASSERT_THAT(sut.has_value(), Eq(true));

            auto result = sut->append(
                *StaticString<SutType::capacity()>::from_utf8_null_terminated_unchecked(invalid_value.c_str()));
            ASSERT_FALSE(result.has_value());
            EXPECT_THAT(result.error(), Eq(SemanticStringError::InvalidContent));
            EXPECT_THAT(sut->size(), value.size());
            EXPECT_THAT(sut->capacity(), Eq(TestValues<SutType>::CAPACITY));

            EXPECT_THAT(sut->as_string().unchecked_access().c_str(), StrEq(value));
        }
    }
}

TYPED_TEST(SemanticStringFixture, generate_invalid_content_with_insert) {
    ::testing::Test::RecordProperty("TEST_ID", "e7db87d3-2574-4b5f-9c3e-c103e05a6b46");
    using SutType = typename TestFixture::SutType;

    for (auto& value : TestValues<SutType>::VALID_VALUES) {
        for (auto& invalid_value : TestValues<SutType>::ADD_VALID_CHARS_TO_CREATE_INVALID_CONTENT_AT_BEGIN) {
            auto sut =
                SutType::create(*StaticString<SutType::capacity()>::from_utf8_null_terminated_unchecked(value.c_str()));
            ASSERT_THAT(sut.has_value(), Eq(true));

            auto result = sut->insert(
                0,
                *StaticString<SutType::capacity()>::from_utf8_null_terminated_unchecked(invalid_value.c_str()),
                invalid_value.size());
            ASSERT_FALSE(result.has_value());
            EXPECT_THAT(result.error(), Eq(SemanticStringError::InvalidContent));
            EXPECT_THAT(sut->size(), value.size());
            EXPECT_THAT(sut->capacity(), Eq(TestValues<SutType>::CAPACITY));

            EXPECT_THAT(sut->as_string().unchecked_access().c_str(), StrEq(value));
        }
    }
}

TYPED_TEST(SemanticStringFixture, append_too_long_content_to_valid_string_fails) {
    ::testing::Test::RecordProperty("TEST_ID", "b8616fbf-601d-43b9-b4a3-f6b96acdf555");
    using SutType = typename TestFixture::SutType;

    for (auto& value : TestValues<SutType>::VALID_VALUES) {
        for (auto& invalid_value : TestValues<SutType>::TOO_LONG_CONTENT_VALUES) {
            auto sut =
                SutType::create(*StaticString<SutType::capacity()>::from_utf8_null_terminated_unchecked(value.c_str()));
            ASSERT_THAT(sut.has_value(), Eq(true));

            EXPECT_THAT(sut->append(*StaticString<SutType::capacity() + 2>::from_utf8_null_terminated_unchecked(
                                        invalid_value.c_str()))
                            .has_value(),
                        Eq(false));
            EXPECT_THAT(sut->size(), Eq(value.size()));
            EXPECT_THAT(sut->capacity(), Eq(TestValues<SutType>::CAPACITY));

            EXPECT_THAT(sut->as_string().unchecked_access().c_str(), StrEq(value));
        }
    }
}

TYPED_TEST(SemanticStringFixture, insert_valid_content_to_valid_string_works) {
    ::testing::Test::RecordProperty("TEST_ID", "56ea499f-5ac3-4ffe-abea-b56194cfd728");
    using SutType = typename TestFixture::SutType;

    // exclude FilePath because a dot at the end is invalid to be compatible with windows api
    if (!std::is_same<SutType, FilePath>::value) {
        for (auto& value : TestValues<SutType>::VALID_VALUES) {
            for (auto& add_value : TestValues<SutType>::VALID_VALUES) {
                for (size_t insert_position = 0; insert_position < value.size(); ++insert_position) {
                    auto sut = SutType::create(
                        *StaticString<SutType::capacity()>::from_utf8_null_terminated_unchecked(value.c_str()));
                    ASSERT_THAT(sut.has_value(), Eq(true));

                    EXPECT_THAT(sut->insert(insert_position,
                                            *StaticString<SutType::capacity()>::from_utf8_null_terminated_unchecked(
                                                add_value.c_str()),
                                            add_value.size())
                                    .has_value(),
                                Eq(true));


                    EXPECT_THAT(sut->size(), Eq(value.size() + add_value.size()));
                    EXPECT_THAT(sut->capacity(), Eq(TestValues<SutType>::CAPACITY));

                    auto result = value;
                    result.insert(insert_position, add_value);
                    EXPECT_THAT(sut->as_string().unchecked_access().c_str(), StrEq(result));
                }
            }
        }
    }
}

TYPED_TEST(SemanticStringFixture, insert_invalid_characters_to_valid_string_fails) {
    ::testing::Test::RecordProperty("TEST_ID", "35229fb8-e6e9-44d9-9d47-d00b71a4ce01");
    using SutType = typename TestFixture::SutType;

    for (auto& value : TestValues<SutType>::VALID_VALUES) {
        for (auto& add_value : TestValues<SutType>::INVALID_CHARACTER_VALUES) {
            for (uint64_t insert_position = 0; insert_position < value.size(); ++insert_position) {
                auto sut = SutType::create(
                    *StaticString<SutType::capacity()>::from_utf8_null_terminated_unchecked(value.c_str()));
                ASSERT_THAT(sut.has_value(), Eq(true));

                auto result = sut->insert(
                    insert_position,
                    *StaticString<SutType::capacity()>::from_utf8_null_terminated_unchecked(add_value.c_str()),
                    add_value.size());
                ASSERT_FALSE(result.has_value());
                EXPECT_THAT(result.error(), Eq(SemanticStringError::InvalidContent));


                EXPECT_THAT(sut->size(), value.size());
                EXPECT_THAT(sut->capacity(), Eq(TestValues<SutType>::CAPACITY));
                EXPECT_THAT(sut->as_string().unchecked_access().c_str(), StrEq(value));
            }
        }
    }
}

TYPED_TEST(SemanticStringFixture, insert_too_long_content_to_valid_string_fails) {
    ::testing::Test::RecordProperty("TEST_ID", "b6939126-a878-4d7f-9fea-c2b438226e65");
    using SutType = typename TestFixture::SutType;

    for (auto& value : TestValues<SutType>::VALID_VALUES) {
        for (auto& add_value : TestValues<SutType>::TOO_LONG_CONTENT_VALUES) {
            for (uint64_t insert_position = 0; insert_position < value.size(); ++insert_position) {
                auto sut = SutType::create(
                    *StaticString<SutType::capacity()>::from_utf8_null_terminated_unchecked(value.c_str()));
                ASSERT_THAT(sut.has_value(), Eq(true));

                EXPECT_THAT(sut->insert(insert_position,
                                        *StaticString<SutType::capacity() + 2>::from_utf8_null_terminated_unchecked(
                                            add_value.c_str()),
                                        add_value.size())
                                .has_value(),
                            Eq(false));


                EXPECT_THAT(sut->size(), Eq(value.size()));
                EXPECT_THAT(sut->capacity(), Eq(TestValues<SutType>::CAPACITY));

                EXPECT_THAT(sut->as_string().unchecked_access().c_str(), StrEq(value));
            }
        }
    }
}

// NOLINTEND(readability-function-cognitive-complexity)

TYPED_TEST(SemanticStringFixture, equality_operator_works) {
    ::testing::Test::RecordProperty("TEST_ID", "97889932-ac3b-4155-9958-34c843d2d323");

    EXPECT_TRUE(this->greater_value == this->greater_value);
    EXPECT_FALSE(this->greater_value == this->smaller_value);

    EXPECT_TRUE(this->greater_value == this->greater_value_str);
    EXPECT_FALSE(this->greater_value == this->smaller_value_str);
}

TYPED_TEST(SemanticStringFixture, inequality_operator_works) {
    ::testing::Test::RecordProperty("TEST_ID", "32903b0b-3594-4c00-9869-d18e1dfc773f");

    EXPECT_FALSE(this->greater_value != this->greater_value);
    EXPECT_TRUE(this->greater_value != this->smaller_value);

    EXPECT_FALSE(this->greater_value != this->greater_value_str);
    EXPECT_TRUE(this->greater_value != this->smaller_value_str);
}

TYPED_TEST(SemanticStringFixture, less_than_or_equal_operator_works) {
    ::testing::Test::RecordProperty("TEST_ID", "53f5b765-b462-4cc1-bab7-9b937fbbcecf");

    EXPECT_TRUE(this->greater_value <= this->greater_value);
    EXPECT_TRUE(this->smaller_value <= this->greater_value);
    EXPECT_FALSE(this->greater_value <= this->smaller_value);
}

TYPED_TEST(SemanticStringFixture, less_than_operator_works) {
    ::testing::Test::RecordProperty("TEST_ID", "cea977a4-ccb3-42a6-9d13-e09dce24c273");

    EXPECT_FALSE(this->greater_value < this->greater_value);
    EXPECT_TRUE(this->smaller_value < this->greater_value);
    EXPECT_FALSE(this->greater_value < this->smaller_value);
}

TYPED_TEST(SemanticStringFixture, greater_than_or_equal_operator_works) {
    ::testing::Test::RecordProperty("TEST_ID", "5d731b17-f787-46fc-b64d-3d86c9102008");

    EXPECT_TRUE(this->greater_value >= this->greater_value);
    EXPECT_FALSE(this->smaller_value >= this->greater_value);
    EXPECT_TRUE(this->greater_value >= this->smaller_value);
}

TYPED_TEST(SemanticStringFixture, greater_than_operator_works) {
    ::testing::Test::RecordProperty("TEST_ID", "8c046cff-fb69-43b4-9a45-e86f17c874db");

    EXPECT_FALSE(this->greater_value > this->greater_value);
    EXPECT_FALSE(this->smaller_value > this->greater_value);
    EXPECT_TRUE(this->greater_value > this->smaller_value);
}
} // namespace
