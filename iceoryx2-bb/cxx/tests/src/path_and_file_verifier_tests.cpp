// Copyright (c) 2019 by Robert Bosch GmbH. All rights reserved.
// Copyright (c) 2021 - 2022 by Apex AI Inc. All rights reserved.
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

#include "iox2/bb/detail/path_and_file_verifier.hpp"
#include "iox2/bb/file_name.hpp"
#include "iox2/bb/static_string.hpp"

#include <gmock/gmock.h>
#include <gtest/gtest.h>

#include <array>

namespace {
using namespace ::testing;
using namespace iox2::bb;
using namespace iox2::bb::detail;
using namespace iox2::legacy;

constexpr uint64_t FILE_PATH_LENGTH = 128U;

// NOLINTNEXTLINE(readability-identifier-length)
auto is_valid_file_character(const int32_t i) noexcept -> bool {
    return ((ASCII_A <= i && i <= ASCII_Z) || (ASCII_CAPITAL_A <= i && i <= ASCII_CAPITAL_Z)
            || (ASCII_0 <= i && i <= ASCII_9) || i == ASCII_DASH || i == ASCII_DOT || i == ASCII_COLON
            || i == ASCII_UNDERSCORE);
}

TEST(PathAndFileVerifier, is_valid_file_name__correct_internal_ascii_aliases) {
    ::testing::Test::RecordProperty("TEST_ID", "e729a0a1-e3c4-4d97-a948-d88017f6ac1e");
    EXPECT_EQ(ASCII_A, 'a');
    EXPECT_EQ(ASCII_Z, 'z');
    EXPECT_EQ(ASCII_CAPITAL_A, 'A');
    EXPECT_EQ(ASCII_CAPITAL_Z, 'Z');
    EXPECT_EQ(ASCII_0, '0');
    EXPECT_EQ(ASCII_9, '9');
    EXPECT_EQ(ASCII_DASH, '-');
    EXPECT_EQ(ASCII_DOT, '.');
    EXPECT_EQ(ASCII_COLON, ':');
    EXPECT_EQ(ASCII_UNDERSCORE, '_');
}

TEST(PathAndFileVerifier, is_valid_file_name__empty_name_is_invalid) {
    ::testing::Test::RecordProperty("TEST_ID", "b2b7aa63-c67e-4915-a906-e3b4779ab772");
    EXPECT_FALSE(is_valid_file_name(StaticString<FILE_PATH_LENGTH>()));
}

TEST(PathAndFileVerifier_is_valid_file_name, relative_path_components_are_invalid) {
    ::testing::Test::RecordProperty("TEST_ID", "b33b4534-f134-499f-ac72-65a3fecaef12");
    EXPECT_FALSE(is_valid_file_name(*StaticString<FILE_PATH_LENGTH>::from_utf8(".")));
    EXPECT_FALSE(is_valid_file_name(*StaticString<FILE_PATH_LENGTH>::from_utf8("..")));
}

// this restriction ensures that we are compatible with the windows
// api which does not support dots and spaces at the end
TEST(PathAndFileVerifier, is_valid_file_name__dots_and_spaces_are_not_valid_at_the_end) {
    ::testing::Test::RecordProperty("TEST_ID", "436b8146-6386-4b03-9fd0-939d2c91eed3");
    EXPECT_FALSE(is_valid_file_name(*StaticString<FILE_PATH_LENGTH>::from_utf8("dot.")));
    EXPECT_FALSE(is_valid_file_name(*StaticString<FILE_PATH_LENGTH>::from_utf8("dotdot..")));
    EXPECT_FALSE(is_valid_file_name(*StaticString<FILE_PATH_LENGTH>::from_utf8("dotdotdot...")));
    EXPECT_FALSE(is_valid_file_name(*StaticString<FILE_PATH_LENGTH>::from_utf8(" ")));
    EXPECT_FALSE(is_valid_file_name(*StaticString<FILE_PATH_LENGTH>::from_utf8(" .")));
    EXPECT_FALSE(is_valid_file_name(*StaticString<FILE_PATH_LENGTH>::from_utf8(" . ")));
    EXPECT_FALSE(is_valid_file_name(*StaticString<FILE_PATH_LENGTH>::from_utf8(". .")));
    EXPECT_FALSE(is_valid_file_name(*StaticString<FILE_PATH_LENGTH>::from_utf8("space ")));
    EXPECT_FALSE(is_valid_file_name(*StaticString<FILE_PATH_LENGTH>::from_utf8("more space  ")));
}

TEST(PathAndFileVerifier, is_valid_file_name__file_name_with_valid_symbols_and_dots_are_valid) {
    ::testing::Test::RecordProperty("TEST_ID", "1455491c-1fc3-4843-a72b-2f51f8f2fadc");
    EXPECT_TRUE(is_valid_file_name(*StaticString<FILE_PATH_LENGTH>::from_utf8("..bla")));
    EXPECT_TRUE(is_valid_file_name(*StaticString<FILE_PATH_LENGTH>::from_utf8(".blubb")));
    EXPECT_TRUE(is_valid_file_name(*StaticString<FILE_PATH_LENGTH>::from_utf8("scna..bla")));
    EXPECT_TRUE(is_valid_file_name(*StaticString<FILE_PATH_LENGTH>::from_utf8("scna.blubb")));
    EXPECT_TRUE(is_valid_file_name(*StaticString<FILE_PATH_LENGTH>::from_utf8(".bla.b.a.sla.a")));
    EXPECT_TRUE(is_valid_file_name(*StaticString<FILE_PATH_LENGTH>::from_utf8("...fuu...man...schmu")));
}

TEST(PathAndFileVerifier, is_valid_file_name__valid_letter_combinations_are_valid) {
    ::testing::Test::RecordProperty("TEST_ID", "1a8661ad-4511-4e54-8cd9-16f21074c332");
    constexpr uint32_t COMBINATION_CAPACITY = 3U;
    std::array<std::string, COMBINATION_CAPACITY> combinations;

    constexpr int32_t MAX_ASCII_CODE = 255;
    for (int32_t i = 0; i <= MAX_ASCII_CODE; ++i) {
        // for simplicity we exclude the valid dot here, since it is
        // invalid when it occurs alone.
        // it is tested separately
        if (i != ASCII_DOT && is_valid_file_character(i)) {
            const uint32_t index = static_cast<uint32_t>(i) % COMBINATION_CAPACITY;

            // index is always in the range of [0, COMBINATION_CAPACITY] since we calculate % COMBINATION_CAPACITY
            // NOLINTNEXTLINE(cppcoreguidelines-pro-bounds-constant-array-index,readability-identifier-length)
            auto& s = combinations[index];
            s.append(1, static_cast<char>(i));

            EXPECT_TRUE(
                is_valid_file_name(*StaticString<FILE_PATH_LENGTH>::from_utf8_null_terminated_unchecked(s.c_str())));
        }
    }
}

TEST(PathAndFileVerifier, is_valid_file_name__when_one_invalid_character_is_contained_file_name_is_invalid) {
    ::testing::Test::RecordProperty("TEST_ID", "067ddf95-8a5c-442b-8022-ecab580b5a7d");
    const std::string valid_name1 = "summon";
    const std::string valid_name2 = "TheHolyToad";

    constexpr int32_t MAX_ASCII_CODE = 127;
    for (int32_t i = 1; i <= MAX_ASCII_CODE; ++i) {
        if (is_valid_file_character(i)) {
            continue;
        }

        std::string invalid_character_front;
        invalid_character_front.append(1, static_cast<char>(i));
        invalid_character_front += valid_name1 + valid_name2;

        std::string invalid_character_middle = valid_name1;
        invalid_character_middle.append(1, static_cast<char>(i));
        invalid_character_middle += valid_name2;

        std::string invalid_character_end = valid_name1 + valid_name2;
        invalid_character_end.append(1, static_cast<char>(i));

        const auto invalid_character_front_test =
            *StaticString<FILE_PATH_LENGTH>::from_utf8_null_terminated_unchecked(invalid_character_front.c_str());
        const auto invalid_character_middle_test =
            *StaticString<FILE_PATH_LENGTH>::from_utf8_null_terminated_unchecked(invalid_character_middle.c_str());
        const auto invalid_character_end_test =
            *StaticString<FILE_PATH_LENGTH>::from_utf8_null_terminated_unchecked(invalid_character_end.c_str());

        EXPECT_FALSE(is_valid_file_name(invalid_character_front_test));
        EXPECT_FALSE(is_valid_file_name(invalid_character_middle_test));
        EXPECT_FALSE(is_valid_file_name(invalid_character_end_test));
    }
}

TEST(PathAndFileVerifier, is_valid_path_to_file__string_with_ending_slash_is_not_a_file_path) {
    ::testing::Test::RecordProperty("TEST_ID", "e0eecf9b-6f2f-4da2-8a18-466504348c50");
    EXPECT_FALSE(is_valid_path_to_file(*StaticString<FILE_PATH_LENGTH>::from_utf8("//")));
    EXPECT_FALSE(is_valid_path_to_file(*StaticString<FILE_PATH_LENGTH>::from_utf8("/")));
    EXPECT_FALSE(is_valid_path_to_file(*StaticString<FILE_PATH_LENGTH>::from_utf8("../")));
    EXPECT_FALSE(is_valid_path_to_file(*StaticString<FILE_PATH_LENGTH>::from_utf8("////")));
    EXPECT_FALSE(is_valid_path_to_file(*StaticString<FILE_PATH_LENGTH>::from_utf8("/fu/bla/far/")));
    EXPECT_FALSE(is_valid_path_to_file(*StaticString<FILE_PATH_LENGTH>::from_utf8("/schnappa/di/puppa//")));
}

TEST(PathAndFileVerifier, is_valid_path_to_file__multiple_slashs_are_valid_file_path) {
    ::testing::Test::RecordProperty("TEST_ID", "d7621d88-d128-4239-8acc-b18f47c92b62");
    EXPECT_TRUE(is_valid_path_to_file(*StaticString<FILE_PATH_LENGTH>::from_utf8("//beginning/double/slash")));
    EXPECT_TRUE(is_valid_path_to_file(*StaticString<FILE_PATH_LENGTH>::from_utf8("/middle//double/slash")));
    EXPECT_TRUE(is_valid_path_to_file(*StaticString<FILE_PATH_LENGTH>::from_utf8("middle//double/slash")));
    EXPECT_TRUE(is_valid_path_to_file(*StaticString<FILE_PATH_LENGTH>::from_utf8("/multi////slash")));
    EXPECT_TRUE(is_valid_path_to_file(*StaticString<FILE_PATH_LENGTH>::from_utf8("////multi/slash")));
    EXPECT_TRUE(is_valid_path_to_file(*StaticString<FILE_PATH_LENGTH>::from_utf8("//multi///slash////hypno")));
}

TEST(PathAndFileVerifier, is_valid_path_to_file__relative_path_components_are_valid) {
    ::testing::Test::RecordProperty("TEST_ID", "ec7d682f-ac7b-4173-a3f6-55969696ee92");
    EXPECT_TRUE(is_valid_path_to_file(*StaticString<FILE_PATH_LENGTH>::from_utf8("../some.file")));
    EXPECT_TRUE(is_valid_path_to_file(*StaticString<FILE_PATH_LENGTH>::from_utf8("./another_file")));
    EXPECT_TRUE(is_valid_path_to_file(*StaticString<FILE_PATH_LENGTH>::from_utf8("./dir/../../fuu-bar")));
    EXPECT_TRUE(is_valid_path_to_file(*StaticString<FILE_PATH_LENGTH>::from_utf8("./././gimme-blubb")));
    EXPECT_TRUE(is_valid_path_to_file(*StaticString<FILE_PATH_LENGTH>::from_utf8("./../.././gimme-blubb")));
}

TEST(PathAndFileVerifier, is_valid_path_to_file__relative_path_beginning_from_root_is_valid) {
    ::testing::Test::RecordProperty("TEST_ID", "30c24356-1777-42a0-906b-73890fd19830");
    EXPECT_TRUE(is_valid_path_to_file(*StaticString<FILE_PATH_LENGTH>::from_utf8("/./././gimme-blubb")));
    EXPECT_TRUE(is_valid_path_to_file(*StaticString<FILE_PATH_LENGTH>::from_utf8("/../../../gimme-blubb")));
    EXPECT_TRUE(is_valid_path_to_file(*StaticString<FILE_PATH_LENGTH>::from_utf8("/../some/dir/gimme-blubb")));
    EXPECT_TRUE(is_valid_path_to_file(*StaticString<FILE_PATH_LENGTH>::from_utf8("/./blubb/dir/gimme-blubb")));
}

TEST(PathAndFileVerifier, is_valid_path_to_file__single_file_is_valid_path) {
    ::testing::Test::RecordProperty("TEST_ID", "264d792f-34cb-4bc0-886c-ac9de05bb1f9");
    EXPECT_TRUE(is_valid_path_to_file(
        *StaticString<FILE_PATH_LENGTH>::from_utf8("gimme-blubb"))); // NOLINT: false positive out-of-bounds
    EXPECT_TRUE(is_valid_path_to_file(*StaticString<FILE_PATH_LENGTH>::from_utf8("a")));
    EXPECT_TRUE(is_valid_path_to_file(*StaticString<FILE_PATH_LENGTH>::from_utf8("fuu:blubb")));
    EXPECT_TRUE(is_valid_path_to_file(*StaticString<FILE_PATH_LENGTH>::from_utf8("/blarbi")));
    EXPECT_TRUE(is_valid_path_to_file(*StaticString<FILE_PATH_LENGTH>::from_utf8("/x")));
    EXPECT_TRUE(is_valid_path_to_file(*StaticString<FILE_PATH_LENGTH>::from_utf8("/fuu:-012")));
}

TEST(PathAndFileVerifier, is_valid_path_to_file__valid_paths_with_no_relative_component_are_valid) {
    ::testing::Test::RecordProperty("TEST_ID", "5556ef38-b028-4155-86c7-dda9530e8611");
    EXPECT_TRUE(is_valid_path_to_file(*StaticString<FILE_PATH_LENGTH>::from_utf8("/fuu/bla/blubb/balaa")));
    EXPECT_TRUE(is_valid_path_to_file(*StaticString<FILE_PATH_LENGTH>::from_utf8("/a/b/c/d/1/2/4")));
    EXPECT_TRUE(is_valid_path_to_file(*StaticString<FILE_PATH_LENGTH>::from_utf8("asd/fuu/asdaaas/1")));
    EXPECT_TRUE(is_valid_path_to_file(*StaticString<FILE_PATH_LENGTH>::from_utf8("123/456")));
}

TEST(PathAndFileVerifier, is_valid_path_to_file__ending_with_relative_path_component_is_invalid) {
    ::testing::Test::RecordProperty("TEST_ID", "c3a5c3e6-840d-4ed5-8064-fede7404391d");
    EXPECT_FALSE(is_valid_path_to_file(*StaticString<FILE_PATH_LENGTH>::from_utf8("/..")));
    EXPECT_FALSE(is_valid_path_to_file(*StaticString<FILE_PATH_LENGTH>::from_utf8("/.")));
    EXPECT_FALSE(is_valid_path_to_file(*StaticString<FILE_PATH_LENGTH>::from_utf8("./..")));
    EXPECT_FALSE(is_valid_path_to_file(*StaticString<FILE_PATH_LENGTH>::from_utf8("../.")));
    EXPECT_FALSE(is_valid_path_to_file(*StaticString<FILE_PATH_LENGTH>::from_utf8("some/path/to/..")));
    EXPECT_FALSE(is_valid_path_to_file(*StaticString<FILE_PATH_LENGTH>::from_utf8("/another/path/to/.")));
    EXPECT_FALSE(is_valid_path_to_file(*StaticString<FILE_PATH_LENGTH>::from_utf8("../bla/fuu/../blubb/.")));
    EXPECT_FALSE(is_valid_path_to_file(*StaticString<FILE_PATH_LENGTH>::from_utf8("./blubb/fuu/../bla/..")));
}

TEST(PathAndFileVerifier, is_valid_path_to_file__file_paths_with_ending_dots_are_invalid) {
    ::testing::Test::RecordProperty("TEST_ID", "2b0dd948-49a0-4eb6-9c78-bad6e6933833");
    EXPECT_FALSE(is_valid_path_to_file(*StaticString<FILE_PATH_LENGTH>::from_utf8("a.")));
    EXPECT_FALSE(is_valid_path_to_file(*StaticString<FILE_PATH_LENGTH>::from_utf8("/asda.")));
    EXPECT_FALSE(is_valid_path_to_file(*StaticString<FILE_PATH_LENGTH>::from_utf8("/bla/../fuu/asda..")));
    EXPECT_FALSE(is_valid_path_to_file(*StaticString<FILE_PATH_LENGTH>::from_utf8("/bla/./.././xa..")));
}

TEST(PathAndFileVerifier, is_valid_path_to_file__path_which_contains_all_valid_characters_is_valid) {
    ::testing::Test::RecordProperty("TEST_ID", "2667afd7-f60c-4d1a-8eff-bf272c68b47a");
    EXPECT_TRUE(is_valid_path_to_file(*StaticString<FILE_PATH_LENGTH>::from_utf8(
        "/abcdefghijklmnopqrstuvwxyz/ABCDEFGHIJKLMNOPQRSTUVWXYZ/0123456789/-.:_")));
    EXPECT_TRUE(is_valid_path_to_file(*StaticString<FILE_PATH_LENGTH>::from_utf8(
        "/abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-.:_")));
}

TEST(PathAndFileVerifier, is_valid_path_to_file__empty_file_path_is_invalid) {
    ::testing::Test::RecordProperty("TEST_ID", "a045581c-3a66-4d0e-b2e2-6ed5a97d4f89");
    EXPECT_FALSE(is_valid_path_to_file(*StaticString<FILE_PATH_LENGTH>::from_utf8("")));
}

// NOLINTNEXTLINE(readability-function-cognitive-complexity)
TEST(
    PathAndFileVerifier,
    is_valid_path_to_file__is_valid_path_to_directory__is_valid_path_entry__when_one_invalid_character_is_contained_path_is_invalid) {
    ::testing::Test::RecordProperty("TEST_ID", "a764cff3-2607-47bb-952b-4ca75f326721");
    const std::string valid_path1 = "/hello";
    const std::string valid_path2 = "fuu/world";

    // begin at 1 since 0 is string termination
    constexpr int32_t MAX_ASCII_CODE = 127;
    for (int32_t i = 1; i <= MAX_ASCII_CODE; ++i) {
        // ignore valid characters
        if (is_valid_file_character(i)) {
            continue;
        }

        // ignore path separators since they are valid path characters
        bool is_path_separator = false;
        for (const auto separator : platform::IOX2_PATH_SEPARATORS) {
            if (static_cast<char>(i) == separator) {
                is_path_separator = true;
                break;
            }
        }

        if (is_path_separator) {
            continue;
        }

        // test
        std::string invalid_character_front;
        invalid_character_front.resize(1);
        invalid_character_front[0] = static_cast<char>(i);
        invalid_character_front += valid_path1 + valid_path2;

        std::string invalid_character_middle = valid_path1;
        invalid_character_middle.resize(invalid_character_middle.size() + 1);
        invalid_character_middle[invalid_character_middle.size() - 1] = static_cast<char>(i);

        std::string invalid_character_end = valid_path1 + valid_path2;
        invalid_character_end.resize(invalid_character_end.size() + 1);
        invalid_character_end[invalid_character_end.size() - 1] = static_cast<char>(i);

        const auto invalid_character_front_test =
            *StaticString<FILE_PATH_LENGTH>::from_utf8_null_terminated_unchecked(invalid_character_front.c_str());
        const auto invalid_character_middle_test =
            *StaticString<FILE_PATH_LENGTH>::from_utf8_null_terminated_unchecked(invalid_character_middle.c_str());
        const auto invalid_character_end_test =
            *StaticString<FILE_PATH_LENGTH>::from_utf8_null_terminated_unchecked(invalid_character_end.c_str());

        EXPECT_FALSE(is_valid_path_to_file(invalid_character_front_test));
        EXPECT_FALSE(is_valid_path_to_file(invalid_character_middle_test));
        EXPECT_FALSE(is_valid_path_to_file(invalid_character_end_test));

        EXPECT_FALSE(is_valid_path_to_directory(invalid_character_front_test));
        EXPECT_FALSE(is_valid_path_to_directory(invalid_character_middle_test));
        EXPECT_FALSE(is_valid_path_to_directory(invalid_character_end_test));

        EXPECT_FALSE(is_valid_path_entry(invalid_character_front_test, RelativePathComponents::Accept));
        EXPECT_FALSE(is_valid_path_entry(invalid_character_middle_test, RelativePathComponents::Accept));
        EXPECT_FALSE(is_valid_path_entry(invalid_character_end_test, RelativePathComponents::Accept));

        EXPECT_FALSE(is_valid_path_entry(invalid_character_front_test, RelativePathComponents::Reject));
        EXPECT_FALSE(is_valid_path_entry(invalid_character_middle_test, RelativePathComponents::Reject));
        EXPECT_FALSE(is_valid_path_entry(invalid_character_end_test, RelativePathComponents::Reject));
    }
}

TEST(PathAndFileVerifier, is_valid_path_to_directory__multiple_slashs_are_valid_path) {
    ::testing::Test::RecordProperty("TEST_ID", "14c6f67f-486a-4b08-a91a-6ef30af84cce");
    EXPECT_TRUE(is_valid_path_to_directory(*StaticString<FILE_PATH_LENGTH>::from_utf8("//beginning/double/slash")));
    EXPECT_TRUE(is_valid_path_to_directory(*StaticString<FILE_PATH_LENGTH>::from_utf8("//beginning/double/slash//")));
    EXPECT_TRUE(is_valid_path_to_directory(*StaticString<FILE_PATH_LENGTH>::from_utf8("/middle//double/slash")));
    EXPECT_TRUE(is_valid_path_to_directory(*StaticString<FILE_PATH_LENGTH>::from_utf8("middle//double/slash")));
    EXPECT_TRUE(is_valid_path_to_directory(*StaticString<FILE_PATH_LENGTH>::from_utf8("middle//double/slash//")));
    EXPECT_TRUE(is_valid_path_to_directory(*StaticString<FILE_PATH_LENGTH>::from_utf8("/multi////slash")));
    EXPECT_TRUE(is_valid_path_to_directory(*StaticString<FILE_PATH_LENGTH>::from_utf8("/multi////slash////")));
    EXPECT_TRUE(is_valid_path_to_directory(*StaticString<FILE_PATH_LENGTH>::from_utf8("////multi/slash")));
    EXPECT_TRUE(is_valid_path_to_directory(*StaticString<FILE_PATH_LENGTH>::from_utf8("//multi///slash////hypno")));
    EXPECT_TRUE(is_valid_path_to_directory(*StaticString<FILE_PATH_LENGTH>::from_utf8("//multi///slash////hypno////")));
}

TEST(PathAndFileVerifier, is_valid_path_to_directory__relative_path_components_are_valid) {
    ::testing::Test::RecordProperty("TEST_ID", "97c215ca-7f67-4ec1-9b17-d98b219a804d");
    EXPECT_TRUE(is_valid_path_to_directory(*StaticString<FILE_PATH_LENGTH>::from_utf8("../some.file")));
    EXPECT_TRUE(is_valid_path_to_directory(*StaticString<FILE_PATH_LENGTH>::from_utf8("../some.dir/")));
    EXPECT_TRUE(is_valid_path_to_directory(*StaticString<FILE_PATH_LENGTH>::from_utf8("./another_file")));
    EXPECT_TRUE(is_valid_path_to_directory(*StaticString<FILE_PATH_LENGTH>::from_utf8("./another_dir/")));
    EXPECT_TRUE(is_valid_path_to_directory(*StaticString<FILE_PATH_LENGTH>::from_utf8("./dir/../../fuu-bar")));
    EXPECT_TRUE(is_valid_path_to_directory(*StaticString<FILE_PATH_LENGTH>::from_utf8("./dir/../../fuu-bar/dir/")));
    EXPECT_TRUE(is_valid_path_to_directory(*StaticString<FILE_PATH_LENGTH>::from_utf8("./././gimme-blubb")));
    EXPECT_TRUE(is_valid_path_to_directory(*StaticString<FILE_PATH_LENGTH>::from_utf8("./././gimme-blubb/dir/")));
    EXPECT_TRUE(is_valid_path_to_directory(*StaticString<FILE_PATH_LENGTH>::from_utf8("./../.././gimme-blubb")));
    EXPECT_TRUE(is_valid_path_to_directory(*StaticString<FILE_PATH_LENGTH>::from_utf8("./../.././gimme-blubb/dir/")));
    EXPECT_TRUE(is_valid_path_to_directory(*StaticString<FILE_PATH_LENGTH>::from_utf8("all/glory/to/the/hypnotoad")));
    EXPECT_TRUE(
        is_valid_path_to_directory(*StaticString<FILE_PATH_LENGTH>::from_utf8("./all/glory/to/the/hypnotoad/")));
    EXPECT_TRUE(
        is_valid_path_to_directory(*StaticString<FILE_PATH_LENGTH>::from_utf8("../all/glory/to/the/hypnotoad/")));
    EXPECT_TRUE(
        is_valid_path_to_directory(*StaticString<FILE_PATH_LENGTH>::from_utf8("../all/glory/to/the/hypnotoad/../")));
}

TEST(PathAndFileVerifier, is_valid_path_to_directory__relative_path_beginning_from_root_is_valid) {
    ::testing::Test::RecordProperty("TEST_ID", "6d2b2656-19ad-4ea0-9ade-77419af849ba");
    EXPECT_TRUE(is_valid_path_to_directory(*StaticString<FILE_PATH_LENGTH>::from_utf8("/./././gimme-blubb")));
    EXPECT_TRUE(is_valid_path_to_directory(*StaticString<FILE_PATH_LENGTH>::from_utf8("/./././gimme-blubb/dir/")));
    EXPECT_TRUE(is_valid_path_to_directory(*StaticString<FILE_PATH_LENGTH>::from_utf8("/../../../gimme-blubb")));
    EXPECT_TRUE(is_valid_path_to_directory(*StaticString<FILE_PATH_LENGTH>::from_utf8("/../../../gimme-blubb/dir/")));
    EXPECT_TRUE(is_valid_path_to_directory(*StaticString<FILE_PATH_LENGTH>::from_utf8("/../some/dir/gimme-blubb")));
    EXPECT_TRUE(
        is_valid_path_to_directory(*StaticString<FILE_PATH_LENGTH>::from_utf8("/../some/dir/gimme-blubb/./dir/")));
    EXPECT_TRUE(is_valid_path_to_directory(*StaticString<FILE_PATH_LENGTH>::from_utf8("/./blubb/dir/gimme-blubb")));
    EXPECT_TRUE(
        is_valid_path_to_directory(*StaticString<FILE_PATH_LENGTH>::from_utf8("/./blubb/dir/gimme-blubb/../dir/")));
}

TEST(PathAndFileVerifier, is_valid_path_to_directory__single_entry_is_valid_path) {
    ::testing::Test::RecordProperty("TEST_ID", "6983ab77-d658-408d-97aa-bd1d218560fb");
    EXPECT_TRUE(is_valid_path_to_directory(*StaticString<FILE_PATH_LENGTH>::from_utf8("gimme-blubb")));
    EXPECT_TRUE(is_valid_path_to_directory(*StaticString<FILE_PATH_LENGTH>::from_utf8("gimme-blubb/")));
    EXPECT_TRUE(is_valid_path_to_directory(*StaticString<FILE_PATH_LENGTH>::from_utf8("a")));
    EXPECT_TRUE(is_valid_path_to_directory(*StaticString<FILE_PATH_LENGTH>::from_utf8("a/")));
    EXPECT_TRUE(is_valid_path_to_directory(*StaticString<FILE_PATH_LENGTH>::from_utf8("fuu:blubb")));
    EXPECT_TRUE(is_valid_path_to_directory(*StaticString<FILE_PATH_LENGTH>::from_utf8("fuu:blubb/")));
    EXPECT_TRUE(is_valid_path_to_directory(*StaticString<FILE_PATH_LENGTH>::from_utf8("/blarbi")));
    EXPECT_TRUE(is_valid_path_to_directory(*StaticString<FILE_PATH_LENGTH>::from_utf8("/blarbi/")));
    EXPECT_TRUE(is_valid_path_to_directory(*StaticString<FILE_PATH_LENGTH>::from_utf8("/x")));
    EXPECT_TRUE(is_valid_path_to_directory(*StaticString<FILE_PATH_LENGTH>::from_utf8("/x/")));
    EXPECT_TRUE(is_valid_path_to_directory(*StaticString<FILE_PATH_LENGTH>::from_utf8("/fuu:-012")));
    EXPECT_TRUE(is_valid_path_to_directory(*StaticString<FILE_PATH_LENGTH>::from_utf8("/fuu:-012/")));
    EXPECT_TRUE(is_valid_path_to_directory(*StaticString<FILE_PATH_LENGTH>::from_utf8("./hypnotoad")));
    EXPECT_TRUE(is_valid_path_to_directory(*StaticString<FILE_PATH_LENGTH>::from_utf8("./hypnotoad/")));
}

TEST(PathAndFileVerifier, is_valid_path_to_directory__valid_paths_with_no_relative_component_are_valid) {
    ::testing::Test::RecordProperty("TEST_ID", "bf7a0a75-c59e-46a8-96f1-1f848e1c3e43");
    EXPECT_TRUE(is_valid_path_to_directory(*StaticString<FILE_PATH_LENGTH>::from_utf8("/fuu/bla/blubb/balaa")));
    EXPECT_TRUE(is_valid_path_to_directory(*StaticString<FILE_PATH_LENGTH>::from_utf8("/fuu/bla/blubb/")));
    EXPECT_TRUE(is_valid_path_to_directory(*StaticString<FILE_PATH_LENGTH>::from_utf8("/a/b/c/d/1/2/4")));
    EXPECT_TRUE(is_valid_path_to_directory(*StaticString<FILE_PATH_LENGTH>::from_utf8("/a/b/c/d/1/2/")));
    EXPECT_TRUE(is_valid_path_to_directory(*StaticString<FILE_PATH_LENGTH>::from_utf8("asd/fuu/asdaaas/1")));
    EXPECT_TRUE(is_valid_path_to_directory(*StaticString<FILE_PATH_LENGTH>::from_utf8("asd/fuu/asdaaas/")));
    EXPECT_TRUE(is_valid_path_to_directory(*StaticString<FILE_PATH_LENGTH>::from_utf8("123/456")));
    EXPECT_TRUE(is_valid_path_to_directory(*StaticString<FILE_PATH_LENGTH>::from_utf8("123/456/")));
}

TEST(PathAndFileVerifier, is_valid_path_to_directory__ending_with_relative_path_component_is_valid) {
    ::testing::Test::RecordProperty("TEST_ID", "506f9823-39cc-4cbc-b064-84d45b2311e8");
    EXPECT_TRUE(is_valid_path_to_directory(*StaticString<FILE_PATH_LENGTH>::from_utf8("/..")));
    EXPECT_TRUE(is_valid_path_to_directory(*StaticString<FILE_PATH_LENGTH>::from_utf8("/.")));
    EXPECT_TRUE(is_valid_path_to_directory(*StaticString<FILE_PATH_LENGTH>::from_utf8("./..")));
    EXPECT_TRUE(is_valid_path_to_directory(*StaticString<FILE_PATH_LENGTH>::from_utf8("../.")));
    EXPECT_TRUE(is_valid_path_to_directory(*StaticString<FILE_PATH_LENGTH>::from_utf8("some/path/to/..")));
    EXPECT_TRUE(is_valid_path_to_directory(*StaticString<FILE_PATH_LENGTH>::from_utf8("/another/path/to/.")));
    EXPECT_TRUE(is_valid_path_to_directory(*StaticString<FILE_PATH_LENGTH>::from_utf8("../bla/fuu/../blubb/.")));
    EXPECT_TRUE(is_valid_path_to_directory(*StaticString<FILE_PATH_LENGTH>::from_utf8("./blubb/fuu/../bla/..")));
}

TEST(PathAndFileVerifier, is_valid_path_to_directory__paths_with_ending_dots_are_invalid) {
    ::testing::Test::RecordProperty("TEST_ID", "f79660e6-12b5-4ad0-bc26-766da34898b8");
    EXPECT_FALSE(is_valid_path_to_directory(*StaticString<FILE_PATH_LENGTH>::from_utf8("a.")));
    EXPECT_FALSE(is_valid_path_to_directory(*StaticString<FILE_PATH_LENGTH>::from_utf8("/asda.")));
    EXPECT_FALSE(is_valid_path_to_directory(*StaticString<FILE_PATH_LENGTH>::from_utf8("/bla/../fuu/asda..")));
    EXPECT_FALSE(is_valid_path_to_directory(*StaticString<FILE_PATH_LENGTH>::from_utf8("/bla/./.././xa..")));
}

TEST(PathAndFileVerifier, is_valid_path_to_directory__path_which_contains_all_valid_characters_is_valid) {
    ::testing::Test::RecordProperty("TEST_ID", "8052b601-c9ad-4cb8-9a87-c301f213d8c4");
    EXPECT_TRUE(is_valid_path_to_directory(*StaticString<FILE_PATH_LENGTH>::from_utf8(
        "/abcdefghijklmnopqrstuvwxyz/ABCDEFGHIJKLMNOPQRSTUVWXYZ/0123456789/-.:_")));
    EXPECT_TRUE(is_valid_path_to_directory(*StaticString<FILE_PATH_LENGTH>::from_utf8(
        "/abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-.:_")));
}

TEST(PathAndFileVerifier, is_valid_path_to_directory__empty_path_is_invalid) {
    ::testing::Test::RecordProperty("TEST_ID", "9724b52e-2e5a-425f-853d-a0b43e553f8b");
    EXPECT_FALSE(is_valid_path_to_directory(StaticString<FILE_PATH_LENGTH>()));
}

TEST(PathAndFileVerifier, does_end_with_sath_separator__empty_path_does_not_end_with_path_separator) {
    ::testing::Test::RecordProperty("TEST_ID", "fe0be1e0-fdd5-4d56-841c-83826c40c3d2");
    EXPECT_FALSE(does_end_with_path_separator(StaticString<FILE_PATH_LENGTH>()));
}

TEST(PathAndFileVerifier,
     does_end_with_sath_separator__non_empty_path_with_no_path_separator_at_the_end_does_not_end_with_path_separator) {
    ::testing::Test::RecordProperty("TEST_ID", "a6d10202-aea0-4b1c-b9d9-704545102a2e");

    auto sut = *StaticString<FILE_PATH_LENGTH>::from_utf8("isThereOnlyOneHypnotoad");
    EXPECT_FALSE(does_end_with_path_separator(sut));

    ASSERT_TRUE(sut.try_append(1, platform::IOX2_PATH_SEPARATORS[0]));
    ASSERT_TRUE(sut.try_append_utf8_null_terminated_unchecked("thereIsOnlyOne"));
    EXPECT_FALSE(does_end_with_path_separator(sut));
}

TEST(PathAndFileVerifier,
     does_end_with_sath_separator__single_character_string_only_with_path_separator_as_one_at_the_end) {
    ::testing::Test::RecordProperty("TEST_ID", "18bf45aa-9b65-4351-956a-8ddc98fa0296");

    // NOLINTNEXTLINE(cppcoreguidelines-pro-bounds-array-to-pointer-decay,hicpp-no-array-decay)
    for (const auto separator : platform::IOX2_PATH_SEPARATORS) {
        auto sut = *StaticString<FILE_PATH_LENGTH>::from_utf8(" ");
        sut.unchecked_access()[0] = separator;
        EXPECT_TRUE(does_end_with_path_separator(sut));
    }
}

TEST(PathAndFileVerifier,
     does_end_with_sath_separator__multi_character_string_ending_with_path_separator_as_one_at_the_end) {
    ::testing::Test::RecordProperty("TEST_ID", "c702ec34-8f7f-4220-b50e-6b231ac4e736");

    // NOLINTNEXTLINE(cppcoreguidelines-pro-bounds-array-to-pointer-decay,hicpp-no-array-decay)
    for (const auto separator : platform::IOX2_PATH_SEPARATORS) {
        auto sut = *StaticString<FILE_PATH_LENGTH>::from_utf8("HypnotoadAteTheSpagettiMonster");
        ASSERT_TRUE(sut.try_append(1, separator));
        EXPECT_TRUE(does_end_with_path_separator(sut));
    }
}

TEST(PathAndFileVerifierFreeFunction, is_valid_path_entry__empty_path_entry_is_valid) {
    ::testing::Test::RecordProperty("TEST_ID", "1280b360-f26c-4ddf-8305-e01a99d58178");
    EXPECT_TRUE(
        is_valid_path_entry(StaticString<platform::IOX2_MAX_FILENAME_LENGTH>(), RelativePathComponents::Accept));
}

TEST(PathAndFileVerifierFreeFunction, is_valid_path_entry__path_entry_with_only_valid_characters_is_valid) {
    ::testing::Test::RecordProperty("TEST_ID", "166fb334-05c6-4b8c-a117-223d6cadb29b");
    EXPECT_TRUE(is_valid_path_entry(*StaticString<platform::IOX2_MAX_FILENAME_LENGTH>::from_utf8("a"),
                                    RelativePathComponents::Accept));
    EXPECT_TRUE(is_valid_path_entry(*StaticString<platform::IOX2_MAX_FILENAME_LENGTH>::from_utf8("agc"),
                                    RelativePathComponents::Accept));
    EXPECT_TRUE(is_valid_path_entry(*StaticString<platform::IOX2_MAX_FILENAME_LENGTH>::from_utf8("a.213jkgc"),
                                    RelativePathComponents::Accept));
}

TEST(PathAndFileVerifierFreeFunction, is_valid_path_entry__relative_path_entries_are_valid) {
    ::testing::Test::RecordProperty("TEST_ID", "d3432692-7cee-416a-a3f3-c246a02ad1a2");
    EXPECT_TRUE(is_valid_path_entry(*StaticString<platform::IOX2_MAX_FILENAME_LENGTH>::from_utf8("."),
                                    RelativePathComponents::Accept));
    EXPECT_TRUE(is_valid_path_entry(*StaticString<platform::IOX2_MAX_FILENAME_LENGTH>::from_utf8(".."),
                                    RelativePathComponents::Accept));
}

TEST(PathAndFileVerifierFreeFunction, is_valid_path_entry__entries_with_ending_dot_are_invalid) {
    ::testing::Test::RecordProperty("TEST_ID", "f937de46-19fc-48da-bce6-51292cd9d75e");
    EXPECT_FALSE(is_valid_path_entry(*StaticString<platform::IOX2_MAX_FILENAME_LENGTH>::from_utf8("abc."),
                                     RelativePathComponents::Accept));
    EXPECT_FALSE(is_valid_path_entry(*StaticString<platform::IOX2_MAX_FILENAME_LENGTH>::from_utf8("19283912asdb.."),
                                     RelativePathComponents::Accept));
    EXPECT_FALSE(is_valid_path_entry(*StaticString<platform::IOX2_MAX_FILENAME_LENGTH>::from_utf8("..19283912asdb.."),
                                     RelativePathComponents::Accept));
    EXPECT_FALSE(is_valid_path_entry(*StaticString<platform::IOX2_MAX_FILENAME_LENGTH>::from_utf8("..192839.12a.sdb.."),
                                     RelativePathComponents::Accept));
}

TEST(PathAndFileVerifierFreeFunction, is_valid_path_entry__entries_with_dots_not_at_the_end_are_valid) {
    ::testing::Test::RecordProperty("TEST_ID", "569aa328-2c47-418d-96e2-ddf73925e52f");
    EXPECT_TRUE(is_valid_path_entry(*StaticString<platform::IOX2_MAX_FILENAME_LENGTH>::from_utf8(".abc"),
                                    RelativePathComponents::Accept));
    EXPECT_TRUE(is_valid_path_entry(*StaticString<platform::IOX2_MAX_FILENAME_LENGTH>::from_utf8(".19283912asdb"),
                                    RelativePathComponents::Accept));
    EXPECT_TRUE(is_valid_path_entry(*StaticString<platform::IOX2_MAX_FILENAME_LENGTH>::from_utf8("..19283912asdb"),
                                    RelativePathComponents::Accept));
    EXPECT_TRUE(is_valid_path_entry(*StaticString<platform::IOX2_MAX_FILENAME_LENGTH>::from_utf8("..192839.12a.sdb"),
                                    RelativePathComponents::Accept));
}

TEST(PathAndFileVerifierFreeFunction, is_valid_path_entry__string_containing_all_valid_characters_is_valid) {
    ::testing::Test::RecordProperty("TEST_ID", "b2c19516-e8fb-4fb8-a366-2b7b5fd9a84b");
    EXPECT_TRUE(is_valid_path_entry(*StaticString<platform::IOX2_MAX_FILENAME_LENGTH>::from_utf8(
                                        "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-.:_"),
                                    RelativePathComponents::Accept));
}

TEST(PathAndFileVerifierFreeFunction, is_valid_path_entry__string_with_slash_is_invalid) {
    ::testing::Test::RecordProperty("TEST_ID", "b1119db1-f897-48a5-af92-9a92eb3f9832");
    EXPECT_FALSE(is_valid_path_entry(*StaticString<platform::IOX2_MAX_FILENAME_LENGTH>::from_utf8("/fuuuu/"),
                                     RelativePathComponents::Accept));
    EXPECT_FALSE(is_valid_path_entry(*StaticString<platform::IOX2_MAX_FILENAME_LENGTH>::from_utf8("fuu/uu"),
                                     RelativePathComponents::Accept));
    EXPECT_FALSE(is_valid_path_entry(*StaticString<platform::IOX2_MAX_FILENAME_LENGTH>::from_utf8("/fuuuu"),
                                     RelativePathComponents::Accept));
    EXPECT_FALSE(is_valid_path_entry(*StaticString<platform::IOX2_MAX_FILENAME_LENGTH>::from_utf8("uuuubbuu/"),
                                     RelativePathComponents::Accept));
}

TEST(PathAndFileVerifierFreeFunction,
     is_valid_path_entry__string_with_relative_components_is_invalid_when_it_contains_relative_components) {
    ::testing::Test::RecordProperty("TEST_ID", "6c73e08e-3b42-446e-b8d4-a4ed7685f28e");
    EXPECT_FALSE(is_valid_path_entry(*StaticString<platform::IOX2_MAX_FILENAME_LENGTH>::from_utf8("../to/be"),
                                     RelativePathComponents::Reject));
    EXPECT_FALSE(is_valid_path_entry(*StaticString<platform::IOX2_MAX_FILENAME_LENGTH>::from_utf8("../../or/not"),
                                     RelativePathComponents::Reject));
    EXPECT_FALSE(is_valid_path_entry(*StaticString<platform::IOX2_MAX_FILENAME_LENGTH>::from_utf8("to/../be"),
                                     RelativePathComponents::Reject));
    EXPECT_FALSE(
        is_valid_path_entry(*StaticString<platform::IOX2_MAX_FILENAME_LENGTH>::from_utf8("that/../../is/the/question"),
                            RelativePathComponents::Reject));
    EXPECT_FALSE(
        is_valid_path_entry(*StaticString<platform::IOX2_MAX_FILENAME_LENGTH>::from_utf8("whether/tis/nobler/.."),
                            RelativePathComponents::Reject));
    EXPECT_FALSE(is_valid_path_entry(
        *StaticString<platform::IOX2_MAX_FILENAME_LENGTH>::from_utf8("in/the/mind/to/suffer//../.."),
        RelativePathComponents::Reject));
    EXPECT_FALSE(is_valid_path_entry(
        *StaticString<platform::IOX2_MAX_FILENAME_LENGTH>::from_utf8("../the/slings/and/arrows/../.."),
        RelativePathComponents::Reject));
    EXPECT_FALSE(is_valid_path_entry(
        *StaticString<platform::IOX2_MAX_FILENAME_LENGTH>::from_utf8("../of/../outrageous/fortune/../.."),
        RelativePathComponents::Reject));
    EXPECT_FALSE(
        is_valid_path_entry(*StaticString<platform::IOX2_MAX_FILENAME_LENGTH>::from_utf8("./or/to/take/../arms/../.."),
                            RelativePathComponents::Reject));
    EXPECT_FALSE(is_valid_path_entry(
        *StaticString<platform::IOX2_MAX_FILENAME_LENGTH>::from_utf8("./agains/a/see/./of/troubles/../.."),
        RelativePathComponents::Reject));
    EXPECT_FALSE(
        is_valid_path_entry(*StaticString<platform::IOX2_MAX_FILENAME_LENGTH>::from_utf8("./and/by/../opposing/./."),
                            RelativePathComponents::Reject));
    EXPECT_FALSE(is_valid_path_entry(*StaticString<platform::IOX2_MAX_FILENAME_LENGTH>::from_utf8("./end/them"),
                                     RelativePathComponents::Reject));
    EXPECT_FALSE(is_valid_path_entry(*StaticString<platform::IOX2_MAX_FILENAME_LENGTH>::from_utf8("to/./die"),
                                     RelativePathComponents::Reject));
    EXPECT_FALSE(is_valid_path_entry(*StaticString<platform::IOX2_MAX_FILENAME_LENGTH>::from_utf8("to/./sleep/."),
                                     RelativePathComponents::Reject));
}

} // namespace
