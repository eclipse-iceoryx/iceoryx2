// Copyright (c) 2019 by Robert Bosch GmbH. All rights reserved.
// Copyright (c) 2021 - 2022 by Apex AI Inc. All rights reserved.
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

#include "iox/detail/path_and_file_verifier.hpp"

#include "iceoryx_hoofs/testing/mocks/logger_mock.hpp"
#include "test.hpp"

#include <array>

namespace
{
using namespace ::testing;
using namespace iox;
using namespace iox::detail;

using iox::testing::Logger_Mock;

constexpr uint64_t FILE_PATH_LENGTH = 128U;

bool isValidFileCharacter(const int32_t i) noexcept
{
    return ((ASCII_A <= i && i <= ASCII_Z) || (ASCII_CAPITAL_A <= i && i <= ASCII_CAPITAL_Z)
            || (ASCII_0 <= i && i <= ASCII_9) || i == ASCII_DASH || i == ASCII_DOT || i == ASCII_COLON
            || i == ASCII_UNDERSCORE);
}

TEST(path_and_flie_verifier_test_isValidFileName, CorrectInternalAsciiAliases)
{
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

TEST(path_and_flie_verifier_test_isValidFileName, EmptyNameIsInvalid)
{
    ::testing::Test::RecordProperty("TEST_ID", "b2b7aa63-c67e-4915-a906-e3b4779ab772");
    EXPECT_FALSE(isValidFileName(string<FILE_PATH_LENGTH>("")));
}

TEST(path_and_flie_verifier_test_isValidFileName, RelativePathComponentsAreInvalid)
{
    ::testing::Test::RecordProperty("TEST_ID", "b33b4534-f134-499f-ac72-65a3fecaef12");
    EXPECT_FALSE(isValidFileName(string<FILE_PATH_LENGTH>(".")));
    EXPECT_FALSE(isValidFileName(string<FILE_PATH_LENGTH>("..")));
}

// this restriction ensures that we are compatible with the windows
// api which does not support dots and spaces at the end
TEST(path_and_flie_verifier_test_isValidFileName, DotsAndSpacesAreNotValidAtTheEnd)
{
    ::testing::Test::RecordProperty("TEST_ID", "436b8146-6386-4b03-9fd0-939d2c91eed3");
    EXPECT_FALSE(isValidFileName(string<FILE_PATH_LENGTH>("dot.")));
    EXPECT_FALSE(isValidFileName(string<FILE_PATH_LENGTH>("dotdot..")));
    EXPECT_FALSE(isValidFileName(string<FILE_PATH_LENGTH>("dotdotdot...")));
    EXPECT_FALSE(isValidFileName(string<FILE_PATH_LENGTH>(" ")));
    EXPECT_FALSE(isValidFileName(string<FILE_PATH_LENGTH>(" .")));
    EXPECT_FALSE(isValidFileName(string<FILE_PATH_LENGTH>(" . ")));
    EXPECT_FALSE(isValidFileName(string<FILE_PATH_LENGTH>(". .")));
    EXPECT_FALSE(isValidFileName(string<FILE_PATH_LENGTH>("space ")));
    EXPECT_FALSE(isValidFileName(string<FILE_PATH_LENGTH>("more space  ")));
}

TEST(path_and_flie_verifier_test_isValidFileName, FileNameWithValidSymbolsAndDotsAreValid)
{
    ::testing::Test::RecordProperty("TEST_ID", "1455491c-1fc3-4843-a72b-2f51f8f2fadc");
    EXPECT_TRUE(isValidFileName(string<FILE_PATH_LENGTH>("..bla")));
    EXPECT_TRUE(isValidFileName(string<FILE_PATH_LENGTH>(".blubb")));
    EXPECT_TRUE(isValidFileName(string<FILE_PATH_LENGTH>("scna..bla")));
    EXPECT_TRUE(isValidFileName(string<FILE_PATH_LENGTH>("scna.blubb")));
    EXPECT_TRUE(isValidFileName(string<FILE_PATH_LENGTH>(".bla.b.a.sla.a")));
    EXPECT_TRUE(isValidFileName(string<FILE_PATH_LENGTH>("...fuu...man...schmu")));
}

TEST(path_and_flie_verifier_test_isValidFileName, ValidLetterCombinationsAreValid)
{
    ::testing::Test::RecordProperty("TEST_ID", "1a8661ad-4511-4e54-8cd9-16f21074c332");
    constexpr uint32_t COMBINATION_CAPACITY = 3U;
    std::array<std::string, COMBINATION_CAPACITY> combinations;

    constexpr int32_t MAX_ASCII_CODE = 255;
    for (int32_t i = 0; i <= MAX_ASCII_CODE; ++i)
    {
        // for simplicity we exclude the valid dot here, since it is
        // invalid when it occurs alone.
        // it is tested separately
        if (i != ASCII_DOT && isValidFileCharacter(i))
        {
            uint32_t index = static_cast<uint32_t>(i) % COMBINATION_CAPACITY;

            // index is always in the range of [0, COMBINATION_CAPACITY] since we calculate % COMBINATION_CAPACITY
            // NOLINTNEXTLINE(cppcoreguidelines-pro-bounds-constant-array-index)
            auto& s = combinations[index];
            s.append(1, static_cast<char>(i));

            EXPECT_TRUE(isValidFileName(string<FILE_PATH_LENGTH>(iox::TruncateToCapacity, s.c_str(), s.size())));
        }
    }
}

TEST(path_and_flie_verifier_test_isValidFileName, WhenOneInvalidCharacterIsContainedFileNameIsInvalid)
{
    ::testing::Test::RecordProperty("TEST_ID", "067ddf95-8a5c-442b-8022-ecab580b5a7d");
    std::string validName1 = "summon";
    std::string validName2 = "TheHolyToad";

    constexpr int32_t MAX_ASCII_CODE = 255;
    for (int32_t i = 0; i <= MAX_ASCII_CODE; ++i)
    {
        if (isValidFileCharacter(i))
        {
            continue;
        }

        std::string invalidCharacterFront;
        invalidCharacterFront.append(1, static_cast<char>(i));
        invalidCharacterFront += validName1 + validName2;

        std::string invalidCharacterMiddle = validName1;
        invalidCharacterMiddle.append(1, static_cast<char>(i));

        std::string invalidCharacterEnd = validName1 + validName2;
        invalidCharacterEnd.append(1, static_cast<char>(i));

        string<FILE_PATH_LENGTH> invalidCharacterFrontTest(
            iox::TruncateToCapacity, invalidCharacterFront.c_str(), invalidCharacterFront.size());
        string<FILE_PATH_LENGTH> invalidCharacterMiddleTest(
            iox::TruncateToCapacity, invalidCharacterMiddle.c_str(), invalidCharacterMiddle.size());
        string<FILE_PATH_LENGTH> invalidCharacterEndTest(
            iox::TruncateToCapacity, invalidCharacterEnd.c_str(), invalidCharacterEnd.size());

        EXPECT_FALSE(isValidFileName(invalidCharacterFrontTest));
        EXPECT_FALSE(isValidFileName(invalidCharacterMiddleTest));
        EXPECT_FALSE(isValidFileName(invalidCharacterEndTest));
    }
}

TEST(path_and_flie_verifier_test_isValidPathToFile, StringWithEndingSlashIsNotAFilePath)
{
    ::testing::Test::RecordProperty("TEST_ID", "e0eecf9b-6f2f-4da2-8a18-466504348c50");
    EXPECT_FALSE(isValidPathToFile(string<FILE_PATH_LENGTH>("//")));
    EXPECT_FALSE(isValidPathToFile(string<FILE_PATH_LENGTH>("/")));
    EXPECT_FALSE(isValidPathToFile(string<FILE_PATH_LENGTH>("../")));
    EXPECT_FALSE(isValidPathToFile(string<FILE_PATH_LENGTH>("////")));
    EXPECT_FALSE(isValidPathToFile(string<FILE_PATH_LENGTH>("/fu/bla/far/")));
    EXPECT_FALSE(isValidPathToFile(string<FILE_PATH_LENGTH>("/schnappa/di/puppa//")));
}

TEST(path_and_flie_verifier_test_isValidPathToFile, MultipleSlashsAreValidFilePath)
{
    ::testing::Test::RecordProperty("TEST_ID", "d7621d88-d128-4239-8acc-b18f47c92b62");
    EXPECT_TRUE(isValidPathToFile(string<FILE_PATH_LENGTH>("//beginning/double/slash")));
    EXPECT_TRUE(isValidPathToFile(string<FILE_PATH_LENGTH>("/middle//double/slash")));
    EXPECT_TRUE(isValidPathToFile(string<FILE_PATH_LENGTH>("middle//double/slash")));
    EXPECT_TRUE(isValidPathToFile(string<FILE_PATH_LENGTH>("/multi////slash")));
    EXPECT_TRUE(isValidPathToFile(string<FILE_PATH_LENGTH>("////multi/slash")));
    EXPECT_TRUE(isValidPathToFile(string<FILE_PATH_LENGTH>("//multi///slash////hypno")));
}

TEST(path_and_flie_verifier_test_isValidPathToFile, RelativePathComponentsAreValid)
{
    ::testing::Test::RecordProperty("TEST_ID", "ec7d682f-ac7b-4173-a3f6-55969696ee92");
    EXPECT_TRUE(isValidPathToFile(string<FILE_PATH_LENGTH>("../some.file")));
    EXPECT_TRUE(isValidPathToFile(string<FILE_PATH_LENGTH>("./another_file")));
    EXPECT_TRUE(isValidPathToFile(string<FILE_PATH_LENGTH>("./dir/../../fuu-bar")));
    EXPECT_TRUE(isValidPathToFile(string<FILE_PATH_LENGTH>("./././gimme-blubb")));
    EXPECT_TRUE(isValidPathToFile(string<FILE_PATH_LENGTH>("./../.././gimme-blubb")));
}

TEST(path_and_flie_verifier_test_isValidPathToFile, RelativePathBeginningFromRootIsValid)
{
    ::testing::Test::RecordProperty("TEST_ID", "30c24356-1777-42a0-906b-73890fd19830");
    EXPECT_TRUE(isValidPathToFile(string<FILE_PATH_LENGTH>("/./././gimme-blubb")));
    EXPECT_TRUE(isValidPathToFile(string<FILE_PATH_LENGTH>("/../../../gimme-blubb")));
    EXPECT_TRUE(isValidPathToFile(string<FILE_PATH_LENGTH>("/../some/dir/gimme-blubb")));
    EXPECT_TRUE(isValidPathToFile(string<FILE_PATH_LENGTH>("/./blubb/dir/gimme-blubb")));
}

TEST(path_and_flie_verifier_test_isValidPathToFile, SingleFileIsValidPath)
{
    ::testing::Test::RecordProperty("TEST_ID", "264d792f-34cb-4bc0-886c-ac9de05bb1f9");
    EXPECT_TRUE(isValidPathToFile(string<FILE_PATH_LENGTH>("gimme-blubb")));
    EXPECT_TRUE(isValidPathToFile(string<FILE_PATH_LENGTH>("a")));
    EXPECT_TRUE(isValidPathToFile(string<FILE_PATH_LENGTH>("fuu:blubb")));
    EXPECT_TRUE(isValidPathToFile(string<FILE_PATH_LENGTH>("/blarbi")));
    EXPECT_TRUE(isValidPathToFile(string<FILE_PATH_LENGTH>("/x")));
    EXPECT_TRUE(isValidPathToFile(string<FILE_PATH_LENGTH>("/fuu:-012")));
}

TEST(path_and_flie_verifier_test_isValidPathToFile, ValidPathsWithNoRelativeComponentAreValid)
{
    ::testing::Test::RecordProperty("TEST_ID", "5556ef38-b028-4155-86c7-dda9530e8611");
    EXPECT_TRUE(isValidPathToFile(string<FILE_PATH_LENGTH>("/fuu/bla/blubb/balaa")));
    EXPECT_TRUE(isValidPathToFile(string<FILE_PATH_LENGTH>("/a/b/c/d/1/2/4")));
    EXPECT_TRUE(isValidPathToFile(string<FILE_PATH_LENGTH>("asd/fuu/asdaaas/1")));
    EXPECT_TRUE(isValidPathToFile(string<FILE_PATH_LENGTH>("123/456")));
}

TEST(path_and_flie_verifier_test_isValidPathToFile, EndingWithRelativePathComponentIsInvalid)
{
    ::testing::Test::RecordProperty("TEST_ID", "c3a5c3e6-840d-4ed5-8064-fede7404391d");
    EXPECT_FALSE(isValidPathToFile(string<FILE_PATH_LENGTH>("/..")));
    EXPECT_FALSE(isValidPathToFile(string<FILE_PATH_LENGTH>("/.")));
    EXPECT_FALSE(isValidPathToFile(string<FILE_PATH_LENGTH>("./..")));
    EXPECT_FALSE(isValidPathToFile(string<FILE_PATH_LENGTH>("../.")));
    EXPECT_FALSE(isValidPathToFile(string<FILE_PATH_LENGTH>("some/path/to/..")));
    EXPECT_FALSE(isValidPathToFile(string<FILE_PATH_LENGTH>("/another/path/to/.")));
    EXPECT_FALSE(isValidPathToFile(string<FILE_PATH_LENGTH>("../bla/fuu/../blubb/.")));
    EXPECT_FALSE(isValidPathToFile(string<FILE_PATH_LENGTH>("./blubb/fuu/../bla/..")));
}

TEST(path_and_flie_verifier_test_isValidPathToFile, FilePathsWithEndingDotsAreInvalid)
{
    ::testing::Test::RecordProperty("TEST_ID", "2b0dd948-49a0-4eb6-9c78-bad6e6933833");
    EXPECT_FALSE(isValidPathToFile(string<FILE_PATH_LENGTH>("a.")));
    EXPECT_FALSE(isValidPathToFile(string<FILE_PATH_LENGTH>("/asda.")));
    EXPECT_FALSE(isValidPathToFile(string<FILE_PATH_LENGTH>("/bla/../fuu/asda..")));
    EXPECT_FALSE(isValidPathToFile(string<FILE_PATH_LENGTH>("/bla/./.././xa..")));
}

TEST(path_and_flie_verifier_test_isValidPathToFile, PathWhichContainsAllValidCharactersIsValid)
{
    ::testing::Test::RecordProperty("TEST_ID", "2667afd7-f60c-4d1a-8eff-bf272c68b47a");
    EXPECT_TRUE(isValidPathToFile(
        string<FILE_PATH_LENGTH>("/abcdefghijklmnopqrstuvwxyz/ABCDEFGHIJKLMNOPQRSTUVWXYZ/0123456789/-.:_")));
    EXPECT_TRUE(isValidPathToFile(
        string<FILE_PATH_LENGTH>("/abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-.:_")));
}

TEST(path_and_flie_verifier_test_isValidPathToFile, EmptyFilePathIsInvalid)
{
    ::testing::Test::RecordProperty("TEST_ID", "a045581c-3a66-4d0e-b2e2-6ed5a97d4f89");
    EXPECT_FALSE(isValidPathToFile(string<FILE_PATH_LENGTH>("")));
}

TEST(path_and_flie_verifier_test_isValidPathToFile_isValidPathToDirectory_isValidPathEntry,
     WhenOneInvalidCharacterIsContainedPathIsInvalid)
{
    ::testing::Test::RecordProperty("TEST_ID", "a764cff3-2607-47bb-952b-4ca75f326721");
    std::string validPath1 = "/hello";
    std::string validPath2 = "fuu/world";

    // begin at 1 since 0 is string termination
    constexpr int32_t MAX_ASCII_CODE = 255;
    for (int32_t i = 1; i <= MAX_ASCII_CODE; ++i)
    {
        // ignore valid characters
        if (isValidFileCharacter(i))
        {
            continue;
        }

        // ignore path separators since they are valid path characters
        bool isPathSeparator = false;
        for (const auto separator : iox::platform::IOX_PATH_SEPARATORS)
        {
            if (static_cast<char>(i) == separator)
            {
                isPathSeparator = true;
                break;
            }
        }

        if (isPathSeparator)
        {
            continue;
        }

        // test
        std::string invalidCharacterFront;
        invalidCharacterFront.resize(1);
        invalidCharacterFront[0] = static_cast<char>(i);
        invalidCharacterFront += validPath1 + validPath2;

        std::string invalidCharacterMiddle = validPath1;
        invalidCharacterMiddle.resize(invalidCharacterMiddle.size() + 1);
        invalidCharacterMiddle[invalidCharacterMiddle.size() - 1] = static_cast<char>(i);

        std::string invalidCharacterEnd = validPath1 + validPath2;
        invalidCharacterEnd.resize(invalidCharacterEnd.size() + 1);
        invalidCharacterEnd[invalidCharacterEnd.size() - 1] = static_cast<char>(i);

        string<FILE_PATH_LENGTH> invalidCharacterFrontTest(
            iox::TruncateToCapacity, invalidCharacterFront.c_str(), invalidCharacterFront.size());
        string<FILE_PATH_LENGTH> invalidCharacterMiddleTest(
            iox::TruncateToCapacity, invalidCharacterMiddle.c_str(), invalidCharacterMiddle.size());
        string<FILE_PATH_LENGTH> invalidCharacterEndTest(
            iox::TruncateToCapacity, invalidCharacterEnd.c_str(), invalidCharacterEnd.size());

        EXPECT_FALSE(isValidPathToFile(invalidCharacterFrontTest));
        EXPECT_FALSE(isValidPathToFile(invalidCharacterMiddleTest));
        EXPECT_FALSE(isValidPathToFile(invalidCharacterEndTest));

        EXPECT_FALSE(isValidPathToDirectory(invalidCharacterFrontTest));
        EXPECT_FALSE(isValidPathToDirectory(invalidCharacterMiddleTest));
        EXPECT_FALSE(isValidPathToDirectory(invalidCharacterEndTest));

        EXPECT_FALSE(isValidPathEntry(invalidCharacterFrontTest, RelativePathComponents::Accept));
        EXPECT_FALSE(isValidPathEntry(invalidCharacterMiddleTest, RelativePathComponents::Accept));
        EXPECT_FALSE(isValidPathEntry(invalidCharacterEndTest, RelativePathComponents::Accept));

        EXPECT_FALSE(isValidPathEntry(invalidCharacterFrontTest, RelativePathComponents::Reject));
        EXPECT_FALSE(isValidPathEntry(invalidCharacterMiddleTest, RelativePathComponents::Reject));
        EXPECT_FALSE(isValidPathEntry(invalidCharacterEndTest, RelativePathComponents::Reject));
    }
}

TEST(path_and_flie_verifier_test_isValidPathToDirectory, MultipleSlashsAreValidPath)
{
    ::testing::Test::RecordProperty("TEST_ID", "14c6f67f-486a-4b08-a91a-6ef30af84cce");
    EXPECT_TRUE(isValidPathToDirectory(string<FILE_PATH_LENGTH>("//beginning/double/slash")));
    EXPECT_TRUE(isValidPathToDirectory(string<FILE_PATH_LENGTH>("//beginning/double/slash//")));
    EXPECT_TRUE(isValidPathToDirectory(string<FILE_PATH_LENGTH>("/middle//double/slash")));
    EXPECT_TRUE(isValidPathToDirectory(string<FILE_PATH_LENGTH>("middle//double/slash")));
    EXPECT_TRUE(isValidPathToDirectory(string<FILE_PATH_LENGTH>("middle//double/slash//")));
    EXPECT_TRUE(isValidPathToDirectory(string<FILE_PATH_LENGTH>("/multi////slash")));
    EXPECT_TRUE(isValidPathToDirectory(string<FILE_PATH_LENGTH>("/multi////slash////")));
    EXPECT_TRUE(isValidPathToDirectory(string<FILE_PATH_LENGTH>("////multi/slash")));
    EXPECT_TRUE(isValidPathToDirectory(string<FILE_PATH_LENGTH>("//multi///slash////hypno")));
    EXPECT_TRUE(isValidPathToDirectory(string<FILE_PATH_LENGTH>("//multi///slash////hypno////")));
}

TEST(path_and_flie_verifier_test_isValidPathToDirectory, RelativePathComponentsAreValid)
{
    ::testing::Test::RecordProperty("TEST_ID", "97c215ca-7f67-4ec1-9b17-d98b219a804d");
    EXPECT_TRUE(isValidPathToDirectory(string<FILE_PATH_LENGTH>("../some.file")));
    EXPECT_TRUE(isValidPathToDirectory(string<FILE_PATH_LENGTH>("../some.dir/")));
    EXPECT_TRUE(isValidPathToDirectory(string<FILE_PATH_LENGTH>("./another_file")));
    EXPECT_TRUE(isValidPathToDirectory(string<FILE_PATH_LENGTH>("./another_dir/")));
    EXPECT_TRUE(isValidPathToDirectory(string<FILE_PATH_LENGTH>("./dir/../../fuu-bar")));
    EXPECT_TRUE(isValidPathToDirectory(string<FILE_PATH_LENGTH>("./dir/../../fuu-bar/dir/")));
    EXPECT_TRUE(isValidPathToDirectory(string<FILE_PATH_LENGTH>("./././gimme-blubb")));
    EXPECT_TRUE(isValidPathToDirectory(string<FILE_PATH_LENGTH>("./././gimme-blubb/dir/")));
    EXPECT_TRUE(isValidPathToDirectory(string<FILE_PATH_LENGTH>("./../.././gimme-blubb")));
    EXPECT_TRUE(isValidPathToDirectory(string<FILE_PATH_LENGTH>("./../.././gimme-blubb/dir/")));
    EXPECT_TRUE(isValidPathToDirectory(string<FILE_PATH_LENGTH>("all/glory/to/the/hypnotoad")));
    EXPECT_TRUE(isValidPathToDirectory(string<FILE_PATH_LENGTH>("./all/glory/to/the/hypnotoad/")));
    EXPECT_TRUE(isValidPathToDirectory(string<FILE_PATH_LENGTH>("../all/glory/to/the/hypnotoad/")));
    EXPECT_TRUE(isValidPathToDirectory(string<FILE_PATH_LENGTH>("../all/glory/to/the/hypnotoad/../")));
}

TEST(path_and_flie_verifier_test_isValidPathToDirectory, RelativePathBeginningFromRootIsValid)
{
    ::testing::Test::RecordProperty("TEST_ID", "6d2b2656-19ad-4ea0-9ade-77419af849ba");
    EXPECT_TRUE(isValidPathToDirectory(string<FILE_PATH_LENGTH>("/./././gimme-blubb")));
    EXPECT_TRUE(isValidPathToDirectory(string<FILE_PATH_LENGTH>("/./././gimme-blubb/dir/")));
    EXPECT_TRUE(isValidPathToDirectory(string<FILE_PATH_LENGTH>("/../../../gimme-blubb")));
    EXPECT_TRUE(isValidPathToDirectory(string<FILE_PATH_LENGTH>("/../../../gimme-blubb/dir/")));
    EXPECT_TRUE(isValidPathToDirectory(string<FILE_PATH_LENGTH>("/../some/dir/gimme-blubb")));
    EXPECT_TRUE(isValidPathToDirectory(string<FILE_PATH_LENGTH>("/../some/dir/gimme-blubb/./dir/")));
    EXPECT_TRUE(isValidPathToDirectory(string<FILE_PATH_LENGTH>("/./blubb/dir/gimme-blubb")));
    EXPECT_TRUE(isValidPathToDirectory(string<FILE_PATH_LENGTH>("/./blubb/dir/gimme-blubb/../dir/")));
}

TEST(path_and_flie_verifier_test_isValidPathToDirectory, SingleEntryIsValidPath)
{
    ::testing::Test::RecordProperty("TEST_ID", "6983ab77-d658-408d-97aa-bd1d218560fb");
    EXPECT_TRUE(isValidPathToDirectory(string<FILE_PATH_LENGTH>("gimme-blubb")));
    EXPECT_TRUE(isValidPathToDirectory(string<FILE_PATH_LENGTH>("gimme-blubb/")));
    EXPECT_TRUE(isValidPathToDirectory(string<FILE_PATH_LENGTH>("a")));
    EXPECT_TRUE(isValidPathToDirectory(string<FILE_PATH_LENGTH>("a/")));
    EXPECT_TRUE(isValidPathToDirectory(string<FILE_PATH_LENGTH>("fuu:blubb")));
    EXPECT_TRUE(isValidPathToDirectory(string<FILE_PATH_LENGTH>("fuu:blubb/")));
    EXPECT_TRUE(isValidPathToDirectory(string<FILE_PATH_LENGTH>("/blarbi")));
    EXPECT_TRUE(isValidPathToDirectory(string<FILE_PATH_LENGTH>("/blarbi/")));
    EXPECT_TRUE(isValidPathToDirectory(string<FILE_PATH_LENGTH>("/x")));
    EXPECT_TRUE(isValidPathToDirectory(string<FILE_PATH_LENGTH>("/x/")));
    EXPECT_TRUE(isValidPathToDirectory(string<FILE_PATH_LENGTH>("/fuu:-012")));
    EXPECT_TRUE(isValidPathToDirectory(string<FILE_PATH_LENGTH>("/fuu:-012/")));
    EXPECT_TRUE(isValidPathToDirectory(string<FILE_PATH_LENGTH>("./hypnotoad")));
    EXPECT_TRUE(isValidPathToDirectory(string<FILE_PATH_LENGTH>("./hypnotoad/")));
}

TEST(path_and_flie_verifier_test_isValidPathToDirectory, ValidPathsWithNoRelativeComponentAreValid)
{
    ::testing::Test::RecordProperty("TEST_ID", "bf7a0a75-c59e-46a8-96f1-1f848e1c3e43");
    EXPECT_TRUE(isValidPathToDirectory(string<FILE_PATH_LENGTH>("/fuu/bla/blubb/balaa")));
    EXPECT_TRUE(isValidPathToDirectory(string<FILE_PATH_LENGTH>("/fuu/bla/blubb/")));
    EXPECT_TRUE(isValidPathToDirectory(string<FILE_PATH_LENGTH>("/a/b/c/d/1/2/4")));
    EXPECT_TRUE(isValidPathToDirectory(string<FILE_PATH_LENGTH>("/a/b/c/d/1/2/")));
    EXPECT_TRUE(isValidPathToDirectory(string<FILE_PATH_LENGTH>("asd/fuu/asdaaas/1")));
    EXPECT_TRUE(isValidPathToDirectory(string<FILE_PATH_LENGTH>("asd/fuu/asdaaas/")));
    EXPECT_TRUE(isValidPathToDirectory(string<FILE_PATH_LENGTH>("123/456")));
    EXPECT_TRUE(isValidPathToDirectory(string<FILE_PATH_LENGTH>("123/456/")));
}

TEST(path_and_flie_verifier_test_isValidPathToDirectory, EndingWithRelativePathComponentIsValid)
{
    ::testing::Test::RecordProperty("TEST_ID", "506f9823-39cc-4cbc-b064-84d45b2311e8");
    EXPECT_TRUE(isValidPathToDirectory(string<FILE_PATH_LENGTH>("/..")));
    EXPECT_TRUE(isValidPathToDirectory(string<FILE_PATH_LENGTH>("/.")));
    EXPECT_TRUE(isValidPathToDirectory(string<FILE_PATH_LENGTH>("./..")));
    EXPECT_TRUE(isValidPathToDirectory(string<FILE_PATH_LENGTH>("../.")));
    EXPECT_TRUE(isValidPathToDirectory(string<FILE_PATH_LENGTH>("some/path/to/..")));
    EXPECT_TRUE(isValidPathToDirectory(string<FILE_PATH_LENGTH>("/another/path/to/.")));
    EXPECT_TRUE(isValidPathToDirectory(string<FILE_PATH_LENGTH>("../bla/fuu/../blubb/.")));
    EXPECT_TRUE(isValidPathToDirectory(string<FILE_PATH_LENGTH>("./blubb/fuu/../bla/..")));
}

TEST(path_and_flie_verifier_test_isValidPathToDirectory, PathsWithEndingDotsAreInvalid)
{
    ::testing::Test::RecordProperty("TEST_ID", "f79660e6-12b5-4ad0-bc26-766da34898b8");
    EXPECT_FALSE(isValidPathToDirectory(string<FILE_PATH_LENGTH>("a.")));
    EXPECT_FALSE(isValidPathToDirectory(string<FILE_PATH_LENGTH>("/asda.")));
    EXPECT_FALSE(isValidPathToDirectory(string<FILE_PATH_LENGTH>("/bla/../fuu/asda..")));
    EXPECT_FALSE(isValidPathToDirectory(string<FILE_PATH_LENGTH>("/bla/./.././xa..")));
}

TEST(path_and_flie_verifier_test_isValidPathToDirectory, PathWhichContainsAllValidCharactersIsValid)
{
    ::testing::Test::RecordProperty("TEST_ID", "8052b601-c9ad-4cb8-9a87-c301f213d8c4");
    EXPECT_TRUE(isValidPathToDirectory(
        string<FILE_PATH_LENGTH>("/abcdefghijklmnopqrstuvwxyz/ABCDEFGHIJKLMNOPQRSTUVWXYZ/0123456789/-.:_")));
    EXPECT_TRUE(isValidPathToDirectory(
        string<FILE_PATH_LENGTH>("/abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-.:_")));
}

TEST(path_and_flie_verifier_test_isValidPathToDirectory, EmptyPathIsInvalid)
{
    ::testing::Test::RecordProperty("TEST_ID", "9724b52e-2e5a-425f-853d-a0b43e553f8b");
    EXPECT_FALSE(isValidPathToDirectory(string<FILE_PATH_LENGTH>("")));
}

TEST(path_and_flie_verifier_test_doesEndWithPathSeparator, EmptyPathDoesNotEndWithPathSeparator)
{
    ::testing::Test::RecordProperty("TEST_ID", "fe0be1e0-fdd5-4d56-841c-83826c40c3d2");
    EXPECT_FALSE(doesEndWithPathSeparator(string<FILE_PATH_LENGTH>("")));
}

TEST(path_and_flie_verifier_test_doesEndWithPathSeparator,
     NonEmptyPathWithNoPathSeparatorAtTheEndDoesNotEndWithPathSeparator)
{
    ::testing::Test::RecordProperty("TEST_ID", "a6d10202-aea0-4b1c-b9d9-704545102a2e");

    string<FILE_PATH_LENGTH> sut = "isThereOnlyOneHypnotoad";
    EXPECT_FALSE(doesEndWithPathSeparator(sut));

    sut.unsafe_append(iox::platform::IOX_PATH_SEPARATORS);
    sut.unsafe_append("thereIsOnlyOne");
    EXPECT_FALSE(doesEndWithPathSeparator(sut));
}

TEST(path_and_flie_verifier_test_doesEndWithPathSeparator, SingleCharacterStringOnlyWithPathSeparatorAsOneAtTheEnd)
{
    ::testing::Test::RecordProperty("TEST_ID", "18bf45aa-9b65-4351-956a-8ddc98fa0296");

    for (const auto separator : iox::platform::IOX_PATH_SEPARATORS)
    {
        string<FILE_PATH_LENGTH> sut = " ";
        sut[0] = separator;
        EXPECT_TRUE(doesEndWithPathSeparator(sut));
    }
}

TEST(path_and_flie_verifier_test_doesEndWithPathSeparator, MultiCharacterStringEndingWithPathSeparatorAsOneAtTheEnd)
{
    ::testing::Test::RecordProperty("TEST_ID", "c702ec34-8f7f-4220-b50e-6b231ac4e736");

    for (const auto separator : iox::platform::IOX_PATH_SEPARATORS)
    {
        string<FILE_PATH_LENGTH> sut = "HypnotoadAteTheSpagettiMonster";
        ASSERT_TRUE(sut.unsafe_append(separator));
        EXPECT_TRUE(doesEndWithPathSeparator(sut));
    }
}

TEST(path_and_flie_verifier_test_isValidPathEntry, EmptyPathEntryIsValid)
{
    ::testing::Test::RecordProperty("TEST_ID", "1280b360-f26c-4ddf-8305-e01a99d58178");
    EXPECT_TRUE(isValidPathEntry(string<iox::platform::IOX_MAX_FILENAME_LENGTH>(""), RelativePathComponents::Accept));
}

TEST(path_and_flie_verifier_test_isValidPathEntry, PathEntryWithOnlyValidCharactersIsValid)
{
    ::testing::Test::RecordProperty("TEST_ID", "166fb334-05c6-4b8c-a117-223d6cadb29b");
    EXPECT_TRUE(isValidPathEntry(string<iox::platform::IOX_MAX_FILENAME_LENGTH>("a"), RelativePathComponents::Accept));
    EXPECT_TRUE(
        isValidPathEntry(string<iox::platform::IOX_MAX_FILENAME_LENGTH>("agc"), RelativePathComponents::Accept));
    EXPECT_TRUE(
        isValidPathEntry(string<iox::platform::IOX_MAX_FILENAME_LENGTH>("a.213jkgc"), RelativePathComponents::Accept));
}

TEST(path_and_flie_verifier_test_isValidPathEntry, RelativePathEntriesAreValid)
{
    ::testing::Test::RecordProperty("TEST_ID", "d3432692-7cee-416a-a3f3-c246a02ad1a2");
    EXPECT_TRUE(isValidPathEntry(string<iox::platform::IOX_MAX_FILENAME_LENGTH>("."), RelativePathComponents::Accept));
    EXPECT_TRUE(isValidPathEntry(string<iox::platform::IOX_MAX_FILENAME_LENGTH>(".."), RelativePathComponents::Accept));
}

TEST(path_and_flie_verifier_test_isValidPathEntry, EntriesWithEndingDotAreInvalid)
{
    ::testing::Test::RecordProperty("TEST_ID", "f937de46-19fc-48da-bce6-51292cd9d75e");
    EXPECT_FALSE(
        isValidPathEntry(string<iox::platform::IOX_MAX_FILENAME_LENGTH>("abc."), RelativePathComponents::Accept));
    EXPECT_FALSE(isValidPathEntry(string<iox::platform::IOX_MAX_FILENAME_LENGTH>("19283912asdb.."),
                                  RelativePathComponents::Accept));
    EXPECT_FALSE(isValidPathEntry(string<iox::platform::IOX_MAX_FILENAME_LENGTH>("..19283912asdb.."),
                                  RelativePathComponents::Accept));
    EXPECT_FALSE(isValidPathEntry(string<iox::platform::IOX_MAX_FILENAME_LENGTH>("..192839.12a.sdb.."),
                                  RelativePathComponents::Accept));
}

TEST(path_and_flie_verifier_test_isValidPathEntry, EntriesWithDotsNotAtTheEndAreValid)
{
    ::testing::Test::RecordProperty("TEST_ID", "569aa328-2c47-418d-96e2-ddf73925e52f");
    EXPECT_TRUE(
        isValidPathEntry(string<iox::platform::IOX_MAX_FILENAME_LENGTH>(".abc"), RelativePathComponents::Accept));
    EXPECT_TRUE(isValidPathEntry(string<iox::platform::IOX_MAX_FILENAME_LENGTH>(".19283912asdb"),
                                 RelativePathComponents::Accept));
    EXPECT_TRUE(isValidPathEntry(string<iox::platform::IOX_MAX_FILENAME_LENGTH>("..19283912asdb"),
                                 RelativePathComponents::Accept));
    EXPECT_TRUE(isValidPathEntry(string<iox::platform::IOX_MAX_FILENAME_LENGTH>("..192839.12a.sdb"),
                                 RelativePathComponents::Accept));
}

TEST(path_and_flie_verifier_test_isValidPathEntry, StringContainingAllValidCharactersIsValid)
{
    ::testing::Test::RecordProperty("TEST_ID", "b2c19516-e8fb-4fb8-a366-2b7b5fd9a84b");
    EXPECT_TRUE(isValidPathEntry(string<iox::platform::IOX_MAX_FILENAME_LENGTH>(
                                     "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-.:_"),
                                 RelativePathComponents::Accept));
}

TEST(path_and_flie_verifier_test_isValidPathEntry, StringWithSlashIsInvalid)
{
    ::testing::Test::RecordProperty("TEST_ID", "b1119db1-f897-48a5-af92-9a92eb3f9832");
    EXPECT_FALSE(
        isValidPathEntry(string<iox::platform::IOX_MAX_FILENAME_LENGTH>("/fuuuu/"), RelativePathComponents::Accept));
    EXPECT_FALSE(
        isValidPathEntry(string<iox::platform::IOX_MAX_FILENAME_LENGTH>("fuu/uu"), RelativePathComponents::Accept));
    EXPECT_FALSE(
        isValidPathEntry(string<iox::platform::IOX_MAX_FILENAME_LENGTH>("/fuuuu"), RelativePathComponents::Accept));
    EXPECT_FALSE(
        isValidPathEntry(string<iox::platform::IOX_MAX_FILENAME_LENGTH>("uuuubbuu/"), RelativePathComponents::Accept));
}

TEST(path_and_flie_verifier_test_isValidPathEntry,
     StringWithRelativeComponentsIsInvalidWhenItContainsRelativeComponents)
{
    ::testing::Test::RecordProperty("TEST_ID", "6c73e08e-3b42-446e-b8d4-a4ed7685f28e");
    EXPECT_FALSE(
        isValidPathEntry(string<iox::platform::IOX_MAX_FILENAME_LENGTH>("../to/be"), RelativePathComponents::Reject));
    EXPECT_FALSE(isValidPathEntry(string<iox::platform::IOX_MAX_FILENAME_LENGTH>("../../or/not"),
                                  RelativePathComponents::Reject));
    EXPECT_FALSE(
        isValidPathEntry(string<iox::platform::IOX_MAX_FILENAME_LENGTH>("to/../be"), RelativePathComponents::Reject));
    EXPECT_FALSE(isValidPathEntry(string<iox::platform::IOX_MAX_FILENAME_LENGTH>("that/../../is/the/question"),
                                  RelativePathComponents::Reject));
    EXPECT_FALSE(isValidPathEntry(string<iox::platform::IOX_MAX_FILENAME_LENGTH>("whether/tis/nobler/.."),
                                  RelativePathComponents::Reject));
    EXPECT_FALSE(isValidPathEntry(string<iox::platform::IOX_MAX_FILENAME_LENGTH>("in/the/mind/to/suffer//../.."),
                                  RelativePathComponents::Reject));
    EXPECT_FALSE(isValidPathEntry(string<iox::platform::IOX_MAX_FILENAME_LENGTH>("../the/slings/and/arrows/../.."),
                                  RelativePathComponents::Reject));
    EXPECT_FALSE(isValidPathEntry(string<iox::platform::IOX_MAX_FILENAME_LENGTH>("../of/../outrageous/fortune/../.."),
                                  RelativePathComponents::Reject));
    EXPECT_FALSE(isValidPathEntry(string<iox::platform::IOX_MAX_FILENAME_LENGTH>("./or/to/take/../arms/../.."),
                                  RelativePathComponents::Reject));
    EXPECT_FALSE(isValidPathEntry(string<iox::platform::IOX_MAX_FILENAME_LENGTH>("./agains/a/see/./of/troubles/../.."),
                                  RelativePathComponents::Reject));
    EXPECT_FALSE(isValidPathEntry(string<iox::platform::IOX_MAX_FILENAME_LENGTH>("./and/by/../opposing/./."),
                                  RelativePathComponents::Reject));
    EXPECT_FALSE(
        isValidPathEntry(string<iox::platform::IOX_MAX_FILENAME_LENGTH>("./end/them"), RelativePathComponents::Reject));
    EXPECT_FALSE(
        isValidPathEntry(string<iox::platform::IOX_MAX_FILENAME_LENGTH>("to/./die"), RelativePathComponents::Reject));
    EXPECT_FALSE(isValidPathEntry(string<iox::platform::IOX_MAX_FILENAME_LENGTH>("to/./sleep/."),
                                  RelativePathComponents::Reject));
}

} // namespace
