// Copyright (c) 2022 by Apex.AI Inc. All rights reserved.
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

#include "iox/cli/command_line_parser.hpp"
#include "iox/detail/hoofs_error_reporting.hpp"
#include "iox/function.hpp"
#include "iox/optional.hpp"
#include "iox/std_string_support.hpp"
#include "test_cli_command_line_common.hpp"

#include "iceoryx_hoofs/testing/error_reporting/testing_support.hpp"
#include "test.hpp"

#include <cstdlib>
#include <iostream>
#include <memory>
#include <string>
#include <vector>

namespace
{
using namespace ::testing;
using namespace iox::cli;
using namespace iox;

class CommandLineParser_test : public Test
{
  public:
    void SetUp() override
    {
        // if we do not capture stdout then the console is filled with garbage
        // since the command line parser prints the help on failure
        outputBuffer.emplace();
    }
    void TearDown() override
    {
        if (Test::HasFailure())
        {
            auto output = outputBuffer->output();
            outputBuffer.reset();
            std::cout << "#### Captured output start ####" << std::endl;
            std::cout << output << std::endl;
            std::cout << "#### Captured output stop ####" << std::endl;
        }
    }

    optional<OutBuffer> outputBuffer;
    uint64_t numberOfErrorCallbackCalls = 0U;
    iox::function<void()> errorCallback = [this] { ++numberOfErrorCallbackCalls; };
    static Argument_t defaultValue;
};
Argument_t CommandLineParser_test::defaultValue = "DEFAULT VALUE";

TEST_F(CommandLineParser_test, SettingBinaryNameWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "bb5e0199-c061-4fa4-be14-c797f996fff6");
    const char* binaryName("AllHailHypnotoad");
    CmdArgs args({binaryName});
    auto options = parseCommandLineArguments(OptionDefinition(""), args.argc, args.argv);

    EXPECT_THAT(options.binaryName(), StrEq(binaryName));
}

TEST_F(CommandLineParser_test, EmptyArgcLeadsToExit)
{
    ::testing::Test::RecordProperty("TEST_ID", "627e7d26-7ba8-466f-8160-61dbff7f3a4d");
    IOX_DISCARD_RESULT(parseCommandLineArguments(OptionDefinition("", errorCallback), 0, nullptr));

    EXPECT_THAT(numberOfErrorCallbackCalls, Eq(1));
}

// NOLINTJUSTIFICATION okay for tests
// NOLINTNEXTLINE(bugprone-easily-swappable-parameters)
void FailureTest(const std::vector<std::string>& options,
                 const std::vector<std::string>& optionsToRegister = {},
                 const std::vector<std::string>& switchesToRegister = {},
                 const std::vector<std::string>& requiredValuesToRegister = {}) noexcept
{
    const char* binaryName("GloryToTheHasselToad");
    std::vector<std::string> optionVector{binaryName};
    optionVector.insert(optionVector.end(), options.begin(), options.end());
    CmdArgs args(optionVector);

    bool wasErrorHandlerCalled = false;
    {
        OptionDefinition optionSet("", [&] { wasErrorHandlerCalled = true; });
        for (const auto& o : optionsToRegister)
        {
            optionSet.addOptional(o[0], iox::into<iox::lossy<OptionName_t>>(o), "", "int", "0");
        }
        for (const auto& s : switchesToRegister)
        {
            optionSet.addSwitch(s[0], iox::into<iox::lossy<OptionName_t>>(s), "");
        }
        for (const auto& r : requiredValuesToRegister)
        {
            optionSet.addRequired(r[0], iox::into<iox::lossy<OptionName_t>>(r), "", "int");
        }

        IOX_DISCARD_RESULT(parseCommandLineArguments(optionSet, args.argc, args.argv, 1U));
    }

    EXPECT_TRUE(wasErrorHandlerCalled);
}

/// BEGIN syntax failure test

TEST_F(CommandLineParser_test, FailSyntaxWhenOptionDoesNotStartWithDash_SingleArgument)
{
    ::testing::Test::RecordProperty("TEST_ID", "e463c987-a908-4cd5-b268-05a2cbda5be2");
    std::vector<std::string> optionsToRegister{"i-have-no-dash"};
    FailureTest({"i-have-no-dash"}, optionsToRegister);
    FailureTest({"i-have-no-dash", "someValue"}, optionsToRegister);
}

TEST_F(CommandLineParser_test, FailSyntaxWhenOptionDoesNotStartWithDash_MultiArgument)
{
    ::testing::Test::RecordProperty("TEST_ID", "da57a066-83da-4bc0-994f-872d7713d8dd");
    std::vector<std::string> optionsToRegister{"i-have-no-dash", "set", "bla"};
    // begin
    FailureTest({"i-have-no-dash", "--set", "setValue", "--bla", "blaValue"}, optionsToRegister);
    FailureTest({"i-have-no-dash", "someValue", "--set", "setValue", "--bla", "blaValue"}, optionsToRegister);
    // middle
    FailureTest({"--set", "setValue", "i-have-no-dash", "--bla", "blaValue"}, optionsToRegister);
    FailureTest({"--set", "setValue", "i-have-no-dash", "someValue", "--bla", "blaValue"}, optionsToRegister);
    // end
    FailureTest({"--set", "setValue", "--bla", "blaValue", "i-have-no-dash"}, optionsToRegister);
    FailureTest({"--set", "setValue", "--bla", "blaValue", "i-have-no-dash", "someValue"}, optionsToRegister);
}

TEST_F(CommandLineParser_test, FailSyntaxWhenOptionDoesNotStartWithDash_MultiArgument_ShortOption)
{
    ::testing::Test::RecordProperty("TEST_ID", "b0f51f16-94ad-4a25-b6f7-11e7e2328472");
    std::vector<std::string> optionsToRegister{"i-have-no-dash", "set", "bla"};
    // begin
    FailureTest({"i", "-s", "setValue", "-b", "blaValue"}, optionsToRegister);
    FailureTest({"i", "someValue", "-s", "setValue", "-b", "blaValue"}, optionsToRegister);
    // middle
    FailureTest({"-s", "setValue", "i", "-b", "blaValue"}, optionsToRegister);
    FailureTest({"-s", "setValue", "i", "someValue", "-b", "blaValue"}, optionsToRegister);
    // end
    FailureTest({"-s", "setValue", "-b", "blaValue", "i"}, optionsToRegister);
    FailureTest({"-s", "setValue", "-b", "blaValue", "i", "someValue"}, optionsToRegister);
}

TEST_F(CommandLineParser_test, FailSyntaxWhenShortOptionNameIsEmpty_SingleArgument)
{
    ::testing::Test::RecordProperty("TEST_ID", "d85ef04b-d91e-438a-8804-bc21c1eebb84");
    FailureTest({"-"});
    FailureTest({"-", "someValue"});
}

TEST_F(CommandLineParser_test, FailSyntaxWhenShortOptionNameIsEmpty_MultiArgument)
{
    ::testing::Test::RecordProperty("TEST_ID", "39c9e200-dd22-4aef-a82e-af84f4336708");
    std::vector<std::string> optionsToRegister{"set", "bla"};
    // begin
    FailureTest({"-", "--set", "setValue123", "--bla", "blaValue455"}, optionsToRegister);
    FailureTest({"-", "someValue", "--set", "setValue123", "--bla", "blaValue455"}, optionsToRegister);
    // middle
    FailureTest({"--set", "setValue123", "-", "--bla", "blaValue455"}, optionsToRegister);
    FailureTest({"--set", "setValue123", "-", "someValue", "--bla", "blaValue455"}, optionsToRegister);
    // end
    FailureTest({"--set", "setValue123", "--bla", "blaValue455", "-"}, optionsToRegister);
    FailureTest({"--set", "setValue123", "--bla", "blaValue455", "-", "someValue"}, optionsToRegister);
}

TEST_F(CommandLineParser_test, FailSyntaxWhenOptionNameIsEmpty_SingleArgument)
{
    ::testing::Test::RecordProperty("TEST_ID", "5316ba5e-0490-4356-a81d-3afd89766b51");
    FailureTest({"--"});
    FailureTest({"--", "someValue"});
}

TEST_F(CommandLineParser_test, FailSyntaxWhenOptionNameIsEmpty_MultiArgument)
{
    ::testing::Test::RecordProperty("TEST_ID", "ca211062-8f23-49a2-8de7-9cffddae6a39");
    std::vector<std::string> optionsToRegister{"set", "bla"};
    // begin
    FailureTest({"--", "--bla", "blaValue123123", "--set", "setValueXXX"}, optionsToRegister);
    FailureTest({"--", "someValue", "--bla", "blaValue123123", "--set", "setValueXXX"}, optionsToRegister);
    // middle
    FailureTest({"--bla", "blaValue123123", "--", "--set", "setValueXXX"}, optionsToRegister);
    FailureTest({"--bla", "blaValue123123", "--", "someValue", "--set", "setValueXXX"}, optionsToRegister);
    // end
    FailureTest({"--bla", "blaValue123123", "--set", "setValueXXX", "--"}, optionsToRegister);
    FailureTest({"--bla", "blaValue123123", "--set", "setValueXXX", "--", "someValue"}, optionsToRegister);
}

TEST_F(CommandLineParser_test, FailSyntaxWhenShortOptionNameHasMoreThenOneLetter_SingleArgument)
{
    ::testing::Test::RecordProperty("TEST_ID", "13776543-6126-403c-96ea-9137590e9e74");
    std::vector<std::string> optionsToRegister{"invalid-option"};
    FailureTest({"-invalid-option"}, optionsToRegister);
    FailureTest({"-invalid-option", "someValue"}, optionsToRegister);
}

TEST_F(CommandLineParser_test, FailSyntaxWhenShortOptionNameHasMoreThenOneLetter_MultiArgument)
{
    ::testing::Test::RecordProperty("TEST_ID", "3f4e337d-5e01-418a-8e30-1a947116ff53");
    std::vector<std::string> optionsToRegister{"set", "bla", "invalid-option"};
    // begin
    FailureTest({"-invalid-option", "--bla", "blaValue123123", "--set", "setValueXXX"}, optionsToRegister);
    FailureTest({"-invalid-option", "someValue", "--bla", "blaValue123123", "--set", "setValueXXX"}, optionsToRegister);
    // middle
    FailureTest({"--bla", "blaValue123123", "-invalid-option", "--set", "setValueXXX"}, optionsToRegister);
    FailureTest({"--bla", "blaValue123123", "-invalid-option", "someValue", "--set", "setValueXXX"}, optionsToRegister);
    // end
    FailureTest({"--bla", "blaValue123123", "--set", "setValueXXX", "-invalid-option"}, optionsToRegister);
    FailureTest({"--bla", "blaValue123123", "--set", "setValueXXX", "-invalid-option", "someValue"}, optionsToRegister);
}

TEST_F(CommandLineParser_test, FailSyntaxWhenLongOptionStartsWithTripleDash_SingleArgument)
{
    ::testing::Test::RecordProperty("TEST_ID", "39eff747-a03f-4c4c-bee3-bb970e32f5b5");
    std::vector<std::string> optionsToRegister{"invalid-long-option"};
    FailureTest({"---invalid-long-option"}, optionsToRegister);
    FailureTest({"---invalid-long-option", "someValue"}, optionsToRegister);
}

TEST_F(CommandLineParser_test, FailSyntaxWhenLongOptionStartsWithTripleDash_MultiArgument)
{
    ::testing::Test::RecordProperty("TEST_ID", "f8b9f82a-a0f4-48c3-b88c-d9b997359d45");
    std::vector<std::string> optionsToRegister{"set", "bla", "invalid-long-option"};
    // begin
    FailureTest({"---invalid-long-option", "--bla", "blaValue123123", "--set", "setValueXXX"}, optionsToRegister);
    FailureTest({"---invalid-long-option", "someValue", "--bla", "blaValue123123", "--set", "setValueXXX"},
                optionsToRegister);
    // middle
    FailureTest({"--bla", "blaValue123123", "---invalid-long-option", "--set", "setValueXXX"}, optionsToRegister);
    FailureTest({"--bla", "blaValue123123", "---invalid-long-option", "someValue", "--set", "setValueXXX"},
                optionsToRegister);
    // end
    FailureTest({"--bla", "blaValue123123", "--set", "setValueXXX", "---invalid-long-option"}, optionsToRegister);
    FailureTest({"--bla", "blaValue123123", "--set", "setValueXXX", "---invalid-long-option", "someValue"},
                optionsToRegister);
}

TEST_F(CommandLineParser_test, FailSyntaxWhenOptionNameExceedMaximumSize_SingleArgument)
{
    ::testing::Test::RecordProperty("TEST_ID", "8066d89f-0fc0-4db2-8bb5-11708f82794f");
    FailureTest({std::string("--") + std::string(MAX_OPTION_NAME_LENGTH + 1, 'a')});
    FailureTest({std::string("--") + std::string(MAX_OPTION_NAME_LENGTH + 1, 'a'), "someValue"});
}

TEST_F(CommandLineParser_test, FailSyntaxWhenOptionNameExceedMaximumSize_MultiArgument)
{
    ::testing::Test::RecordProperty("TEST_ID", "4c530a35-de80-4352-ae13-763a1ccfae5c");
    std::vector<std::string> optionsToRegister{"set", "bla"};
    // begin
    FailureTest(
        {std::string("--") + std::string(MAX_OPTION_NAME_LENGTH + 1, 'a'), "--set", "setValue", "--bla", "blaValue"},
        optionsToRegister);
    FailureTest({std::string("--") + std::string(MAX_OPTION_NAME_LENGTH + 1, 'a'),
                 "someValue",
                 "--set",
                 "setValue",
                 "--bla",
                 "blaValue"},
                optionsToRegister);
    // middle
    FailureTest(
        {"--set", "setValue", std::string("--") + std::string(MAX_OPTION_NAME_LENGTH + 1, 'a'), "--bla", "blaValue"},
        optionsToRegister);
    FailureTest({"someValue",
                 "--set",
                 std::string("--") + std::string(MAX_OPTION_NAME_LENGTH + 1, 'a'),
                 "setValue",
                 "--bla",
                 "blaValue"},
                optionsToRegister);
    // end
    FailureTest(
        {"--set", "setValue", "--bla", "blaValue", std::string("--") + std::string(MAX_OPTION_NAME_LENGTH + 1, 'a')},
        optionsToRegister);
    FailureTest({"--set",
                 "setValue",
                 "--bla",
                 "blaValue",
                 std::string("--") + std::string(MAX_OPTION_NAME_LENGTH + 1, 'a'),
                 "someValue"},
                optionsToRegister);
}

/// END syntax failure test

/// BEGIN option failure test
TEST_F(CommandLineParser_test, FailWhenOptionWasNotRegistered_SingleArgument)
{
    ::testing::Test::RecordProperty("TEST_ID", "ce0c8994-7999-41cf-8356-dafc6cfd5107");
    std::vector<std::string> optionsToRegister{"sputnik", "rosetta"};
    FailureTest({"--conway", "gameOfLife"}, optionsToRegister);
}

TEST_F(CommandLineParser_test, FailWhenOptionWasNotRegistered_MultiArgument)
{
    ::testing::Test::RecordProperty("TEST_ID", "68e4cdb8-b50d-42da-a51a-2c98b882613b");
    std::vector<std::string> optionsToRegister{"sputnik", "rosetta"};
    // begin
    FailureTest({"--conway", "gameOfLife", "--sputnik", "iWasFirst", "--rosetta", "uhWhatsThere"}, optionsToRegister);
    // middle
    FailureTest({"--sputnik", "iWasFirst", "--conway", "gameOfLife", "--rosetta", "uhWhatsThere"}, optionsToRegister);
    // end
    FailureTest({"--sputnik", "iWasFirst", "--rosetta", "uhWhatsThere", "--conway", "gameOfLife"}, optionsToRegister);
}

TEST_F(CommandLineParser_test, FailWhenOptionWasNotRegistered_MultiArgument_ShortOption)
{
    ::testing::Test::RecordProperty("TEST_ID", "a4de02ed-3057-4d54-bcad-5a55ed8ea9ed");
    std::vector<std::string> optionsToRegister{"sputnik", "rosetta"};
    // begin
    FailureTest({"-c", "gameOfLife", "-s", "iWasFirst", "-r", "uhWhatsThere"}, optionsToRegister);
    // middle
    FailureTest({"-s", "gameOfLife", "-c", "gameOfLife", "-r", "uhWhatsThere"}, optionsToRegister);
    // end
    FailureTest({"-s", "gameOfLife", "-r", "uhWhatsThere", "-c", "gameOfLife"}, optionsToRegister);
}

TEST_F(CommandLineParser_test, FailWhenValueOptionIsFollowedByAnotherOption_SingleArgument)
{
    ::testing::Test::RecordProperty("TEST_ID", "72eb64a6-a323-4755-a7d4-303a04b31383");
    std::vector<std::string> optionsToRegister{"set"};
    FailureTest({"--set"}, optionsToRegister);
}

TEST_F(CommandLineParser_test, FailWhenValueOptionIsFollowedByAnotherOption_MultiArgument)
{
    ::testing::Test::RecordProperty("TEST_ID", "0743ad80-d6dc-4095-b1c3-d81562eb4c85");
    std::vector<std::string> optionsToRegister{"set", "bla", "fuu", "oh-no-i-am-an-option"};
    FailureTest({"--fuu", "fuuValue", "--bla", "blaValue", "--set", "someValue", "--oh-no-i-am-an-option"},
                optionsToRegister);
}

TEST_F(CommandLineParser_test, FailWhenValueOptionIsFollowedByAnotherOption_MultiArgument_ShortOption)
{
    ::testing::Test::RecordProperty("TEST_ID", "0bc9bbf0-b1fe-4d59-89de-d3462d90f7ff");
    std::vector<std::string> optionsToRegister{"set", "bla", "fuu", "oh-no-i-am-an-option"};
    FailureTest({"-f", "fuuValue", "-b", "blaValue", "-s", "blubb", "-o"}, optionsToRegister);
}

TEST_F(CommandLineParser_test, FailWhenValueOptionIsSetMultipleTimes_SingleArgument)
{
    ::testing::Test::RecordProperty("TEST_ID", "1c17a7f9-6a34-4bb4-a4f4-0415c92325d4");
    std::vector<std::string> optionsToRegister{"set"};
    FailureTest({"--set", "bla", "--set", "fuu"}, optionsToRegister);
}

TEST_F(CommandLineParser_test, FailWhenValueOptionIsSetMultipleTimes_MultiArgument)
{
    ::testing::Test::RecordProperty("TEST_ID", "2c1f9325-4839-41eb-a0ea-8a8ca06e5357");
    std::vector<std::string> optionsToRegister{"set", "bla", "fuu"};
    FailureTest({"--set", "fuuu", "--bla", "blaValue", "--fuu", "fuuValue", "--set", "bla"}, optionsToRegister);
    FailureTest({"--bla", "blaValue", "--set", "fuuu", "--fuu", "fuuValue", "--set", "bla"}, optionsToRegister);
    FailureTest({"--set", "fuuu", "--bla", "blaValue", "--set", "bla", "--fuu", "fuuValue"}, optionsToRegister);
}

TEST_F(CommandLineParser_test, FailWhenValueOptionIsSetMultipleTimes_MultiArgument_ShortOption)
{
    ::testing::Test::RecordProperty("TEST_ID", "17fd7b1c-6026-4b15-b0c2-862c4ce0b00b");
    std::vector<std::string> optionsToRegister{"set", "bla", "fuu"};
    FailureTest({"-s", "fuuu", "-b", "blaValue", "-f", "fuuValue", "-s", "bla"}, optionsToRegister);
    FailureTest({"-b", "blaValue", "-s", "fuuu", "-f", "fuuValue", "-s", "bla"}, optionsToRegister);
    FailureTest({"-s", "fuuu", "-b", "blaValue", "-s", "bla", "-f", "fuuValue"}, optionsToRegister);
}

TEST_F(CommandLineParser_test, FailWhenOptionValueExceedMaximumSize_SingleArgument)
{
    ::testing::Test::RecordProperty("TEST_ID", "500e56b2-b7a6-4ed9-8583-d109f530d09f");
    std::vector<std::string> optionsToRegister{"set"};
    FailureTest({"--set", std::string(MAX_OPTION_ARGUMENT_LENGTH + 1, 'a')}, optionsToRegister);
}

TEST_F(CommandLineParser_test, FailWhenOptionValueExceedMaximumSize_MultiArgument)
{
    ::testing::Test::RecordProperty("TEST_ID", "04b6ef11-8882-4e6e-8759-44eac330c822");
    std::vector<std::string> optionsToRegister{"set", "bla", "fuu"};

    // begin
    FailureTest({"--set", std::string(MAX_OPTION_ARGUMENT_LENGTH + 1, 'a'), "--bla", "blaValue", "--fuu", "fuuValue"},
                optionsToRegister);
    // middle
    FailureTest({"--set", "blaValue", "--bla", std::string(MAX_OPTION_ARGUMENT_LENGTH + 1, 'a'), "--fuu", "fuuValue"},
                optionsToRegister);
    // end
    FailureTest({"--set", "blaValue", "--bla", "fuuValue", "--fuu", std::string(MAX_OPTION_ARGUMENT_LENGTH + 1, 'a')},
                optionsToRegister);
}

TEST_F(CommandLineParser_test, FailWhenOptionValueExceedMaximumSize_MultiArgument_ShortOption)
{
    ::testing::Test::RecordProperty("TEST_ID", "4a74242a-d7c5-4275-8a7b-2249107036ff");
    std::vector<std::string> optionsToRegister{"set", "bla", "fuu"};

    // begin
    FailureTest({"-s", std::string(MAX_OPTION_ARGUMENT_LENGTH + 1, 'a'), "-b", "blaValue", "-f", "fuuValue"},
                optionsToRegister);
    // middle
    FailureTest({"-s", "blaValue", "-b", std::string(MAX_OPTION_ARGUMENT_LENGTH + 1, 'a'), "-f", "fuuValue"},
                optionsToRegister);
    // end
    FailureTest({"-s", "blaValue", "-b", "fuuValue", "-f", std::string(MAX_OPTION_ARGUMENT_LENGTH + 1, 'a')},
                optionsToRegister);
}
/// END option failure test

/// BEGIN switch failure test
TEST_F(CommandLineParser_test, FailWhenSwitchWasNotRegistered_SingleArgument)
{
    ::testing::Test::RecordProperty("TEST_ID", "92236356-4729-414e-8edc-65eb23cd20d0");
    std::vector<std::string> optionsToRegister;
    std::vector<std::string> switchesToRegister{"supergandalf", "grand-alf"};

    FailureTest({"--mario"}, optionsToRegister, switchesToRegister);
}

TEST_F(CommandLineParser_test, FailWhenSwitchWasNotRegistered_MultiArgument)
{
    ::testing::Test::RecordProperty("TEST_ID", "f5a9fa53-4170-4f21-a028-f4cfd7beeac3");
    std::vector<std::string> optionsToRegister;
    std::vector<std::string> switchesToRegister{"supergandalf", "grand-alf"};

    // begin
    FailureTest({"--mario", "--supergandalf", "--grand-alf"}, optionsToRegister, switchesToRegister);
    // middle
    FailureTest({"--supergandalf", "--mario", "--grand-alf"}, optionsToRegister, switchesToRegister);
    // end
    FailureTest({"--supergandalf", "--grand-alf", "--mario"}, optionsToRegister, switchesToRegister);
}

TEST_F(CommandLineParser_test, FailWhenSwitchWasNotRegistered_MultiArgument_ShortOption)
{
    ::testing::Test::RecordProperty("TEST_ID", "311873e0-159f-4f0a-8228-32e659bd52ea");
    std::vector<std::string> optionsToRegister;
    std::vector<std::string> switchesToRegister{"supergandalf", "grand-alf"};

    // begin
    FailureTest({"-m", "-s", "-g"}, optionsToRegister, switchesToRegister);
    // middle
    FailureTest({"-s", "-m", "-g"}, optionsToRegister, switchesToRegister);
    // end
    FailureTest({"-s", "-g", "-m"}, optionsToRegister, switchesToRegister);
}

TEST_F(CommandLineParser_test, FailWhenSwitchHasValueSet_SingleArgument)
{
    ::testing::Test::RecordProperty("TEST_ID", "24d76c82-dc7b-48b3-a88b-dada402802cc");
    std::vector<std::string> optionsToRegister;
    std::vector<std::string> switchesToRegister{"set"};

    FailureTest({"--set", "noValueAfterSwitch"}, optionsToRegister, switchesToRegister);
}

TEST_F(CommandLineParser_test, FailWhenSwitchHasValueSet_MultiArgument)
{
    ::testing::Test::RecordProperty("TEST_ID", "b8b562ad-d502-4ada-8eac-8d8ffce22689");
    std::vector<std::string> optionsToRegister;
    std::vector<std::string> switchesToRegister{"set", "bla", "fuu"};

    // begin
    FailureTest({"--set", "noValueAfterSwitch", "--bla", "--fuu"}, optionsToRegister, switchesToRegister);
    // middle
    FailureTest({"--set", "--bla", "noValueAfterSwitch", "--fuu"}, optionsToRegister, switchesToRegister);
    // end
    FailureTest({"--set", "--bla", "--fuu", "noValueAfterSwitch"}, optionsToRegister, switchesToRegister);
}

TEST_F(CommandLineParser_test, FailWhenSwitchHasValueSet_MultiArgument_ShortOption)
{
    ::testing::Test::RecordProperty("TEST_ID", "e03c4823-00e2-4d4d-acf0-933086e77bef");
    std::vector<std::string> optionsToRegister;
    std::vector<std::string> switchesToRegister{"set", "bla", "fuu"};

    // begin
    FailureTest({"-s", "noValueAfterSwitch", "-b", "-f"}, optionsToRegister, switchesToRegister);
    // middle
    FailureTest({"-s", "-b", "noValueAfterSwitch", "-f"}, optionsToRegister, switchesToRegister);
    // end
    FailureTest({"-s", "-b", "-f", "noValueAfterSwitch"}, optionsToRegister, switchesToRegister);
}

TEST_F(CommandLineParser_test, FailWhenSwitchIsSetMultipleTimes_SingleArgument)
{
    ::testing::Test::RecordProperty("TEST_ID", "a900e3c2-c3ef-415e-b8ce-734ce44c479e");
    std::vector<std::string> optionsToRegister;
    std::vector<std::string> switchesToRegister{"set"};
    FailureTest({"--set", "--set"}, optionsToRegister, switchesToRegister);
}

TEST_F(CommandLineParser_test, FailWhenSwitchIsSetMultipleTimes_MultiArgument)
{
    ::testing::Test::RecordProperty("TEST_ID", "5010cb12-cd30-470d-8065-2618d3b257c3");
    std::vector<std::string> optionsToRegister;
    std::vector<std::string> switchesToRegister{"set", "bla", "fuu"};

    // begin
    FailureTest({"--set", "--set", "--bla", "--fuu"}, optionsToRegister, switchesToRegister);
    // middle
    FailureTest({"--set", "--bla", "--set", "--fuu"}, optionsToRegister, switchesToRegister);
    // end
    FailureTest({"--set", "--bla", "--fuu", "--set"}, optionsToRegister, switchesToRegister);
    // center
    FailureTest({"--set", "--fuu", "--fuu", "--bla"}, optionsToRegister, switchesToRegister);
}

/// END switch failure test

/// BEGIN required option failure test
TEST_F(CommandLineParser_test, FailWhenRequiredOptionIsNotPresent_SingleArgument)
{
    ::testing::Test::RecordProperty("TEST_ID", "1007661d-4d84-49a1-8554-9f21f2ccc3f3");
    std::vector<std::string> optionsToRegister{};
    std::vector<std::string> switchesToRegister{};
    std::vector<std::string> requiredValuesToRegister{"set", "fuu"};

    FailureTest({"--set", "ohIForgotFuu"}, optionsToRegister, switchesToRegister, requiredValuesToRegister);
}

TEST_F(CommandLineParser_test, FailWhenRequiredOptionIsNotPresent_MultiArgument)
{
    ::testing::Test::RecordProperty("TEST_ID", "4786109a-f7b2-42c9-a40b-d1fb94a45432");
    std::vector<std::string> optionsToRegister{};
    std::vector<std::string> switchesToRegister{};
    std::vector<std::string> requiredValuesToRegister{"set", "fuu", "bla", "muu"};

    // begin
    FailureTest({"--bla", "ohIForgotSet", "--fuu", "someFuu", "--muu", "blaarb"},
                optionsToRegister,
                switchesToRegister,
                requiredValuesToRegister);
    // middle
    FailureTest({"--set", "ohIForgetBla", "--fuu", "someFuu", "--muu", "blaarb"},
                optionsToRegister,
                switchesToRegister,
                requiredValuesToRegister);
    // end
    FailureTest({"--set", "ohIForgotMuu", "--fuu", "someFuu", "--bla", "someBlaa"},
                optionsToRegister,
                switchesToRegister,
                requiredValuesToRegister);
}

TEST_F(CommandLineParser_test, FailWhenRequiredOptionIsNotPresent_MultiArgument_ShortOption)
{
    ::testing::Test::RecordProperty("TEST_ID", "cd7f22f2-cde4-48e3-8c0a-5610fee3c9fc");
    std::vector<std::string> optionsToRegister{};
    std::vector<std::string> switchesToRegister{};
    std::vector<std::string> requiredValuesToRegister{"set", "fuu", "bla", "muu"};

    // begin
    FailureTest({"-b", "ohIForgotSet", "-f", "someFuu", "-m", "blaarb"},
                optionsToRegister,
                switchesToRegister,
                requiredValuesToRegister);
    // middle
    FailureTest({"-s", "ohIForgetBla", "-f", "someFuu", "-m", "blaarb"},
                optionsToRegister,
                switchesToRegister,
                requiredValuesToRegister);
    // end
    FailureTest({"-s", "ohIForgotMuu", "-f", "someFuu", "-b", "someBlaa"},
                optionsToRegister,
                switchesToRegister,
                requiredValuesToRegister);
}

TEST_F(CommandLineParser_test, FailWhenRequiredOptionIsNotFollowedByValue_SingleArgument)
{
    ::testing::Test::RecordProperty("TEST_ID", "154306d7-816c-4200-a1ad-f27a3cdb62e1");
    std::vector<std::string> optionsToRegister{};
    std::vector<std::string> switchesToRegister{};
    std::vector<std::string> requiredValuesToRegister{"set"};

    FailureTest({"--set"}, optionsToRegister, switchesToRegister, requiredValuesToRegister);
}

TEST_F(CommandLineParser_test, FailWhenRequiredOptionIsNotFollowedByValue_MultiArgument)
{
    ::testing::Test::RecordProperty("TEST_ID", "e97c1d07-39e8-48e8-b941-d0b6f3f7e73f");
    std::vector<std::string> optionsToRegister{};
    std::vector<std::string> switchesToRegister{};
    std::vector<std::string> requiredValuesToRegister{"set", "fuu", "bla", "toad"};

    // begin
    FailureTest({"--set", "--fuu", "someValue", "--bla", "blaValue", "--toad", "hypno"},
                optionsToRegister,
                switchesToRegister,
                requiredValuesToRegister);
    // middle
    FailureTest({"--set", "someSet", "--fuu", "someValue", "--bla", "--toad", "hypno"},
                optionsToRegister,
                switchesToRegister,
                requiredValuesToRegister);
    // end
    FailureTest({"--set", "someSet", "--fuu", "someValue", "--bla", "--toad"},
                optionsToRegister,
                switchesToRegister,
                requiredValuesToRegister);
}

TEST_F(CommandLineParser_test, FailWhenRequiredOptionIsNotFollowedByValue_MultiArgument_ShortOption)
{
    ::testing::Test::RecordProperty("TEST_ID", "8afa4f3a-5d94-4cf4-aa10-b3612ecfc817");
    std::vector<std::string> optionsToRegister{};
    std::vector<std::string> switchesToRegister{};
    std::vector<std::string> requiredValuesToRegister{"set", "fuu", "bla", "toad"};

    // begin
    FailureTest({"-s", "-f", "someValue", "-b", "blaValue", "-t", "hypno"},
                optionsToRegister,
                switchesToRegister,
                requiredValuesToRegister);
    // middle
    FailureTest({"-s", "someSet", "-f", "someValue", "-b", "-t", "hypno"},
                optionsToRegister,
                switchesToRegister,
                requiredValuesToRegister);
    // end
    FailureTest({"-s", "someSet", "-f", "someValue", "-b", "-t"},
                optionsToRegister,
                switchesToRegister,
                requiredValuesToRegister);
}
/// END required option failure test

/// BEGIN required, optional option and switch failure mix
TEST_F(CommandLineParser_test, FailWhenOneRequiredOptionIsNotSet_MixedArguments)
{
    ::testing::Test::RecordProperty("TEST_ID", "c437b65b-585b-4ec9-8a3a-abb7add92f0c");
    std::vector<std::string> optionsToRegister{"a-opt", "b-opt", "c-opt"};
    std::vector<std::string> switchesToRegister{"d-switch", "e-switch", "f-switch"};
    std::vector<std::string> requiredValuesToRegister{"i-req", "j-req", "k-req"};

    FailureTest({"--d-switch", "--f-switch", "--a-opt", "someA", "--k-req", "fSet", "--i-req", "asd"},
                optionsToRegister,
                switchesToRegister,
                requiredValuesToRegister);
}

TEST_F(CommandLineParser_test, FailWhenMultipleRequiredOptionsAreNotSet_MixedArguments)
{
    ::testing::Test::RecordProperty("TEST_ID", "e36058bb-2bde-4781-9aff-8e9fa524e925");
    std::vector<std::string> optionsToRegister{"a-opt", "b-opt", "c-opt"};
    std::vector<std::string> switchesToRegister{"d-switch", "e-switch", "f-switch"};
    std::vector<std::string> requiredValuesToRegister{"i-req", "j-req", "k-req"};

    FailureTest({"--d-switch", "--f-switch", "--a-opt", "someA", "--i-req", "asd", "--b-opt", "asd"},
                optionsToRegister,
                switchesToRegister,
                requiredValuesToRegister);
}

TEST_F(CommandLineParser_test, FailWhenNoRequiredOptionIsSet_MixedArguments)
{
    ::testing::Test::RecordProperty("TEST_ID", "a8e079aa-6d35-4ea6-9df0-f9d14ecab0ec");
    std::vector<std::string> optionsToRegister{"a-opt", "b-opt", "c-opt"};
    std::vector<std::string> switchesToRegister{"d-switch", "e-switch", "f-switch"};
    std::vector<std::string> requiredValuesToRegister{"i-req", "j-req", "k-req"};

    FailureTest({"--d-switch", "--f-switch", "--a-opt", "someA", "--e-switch", "--b-opt", "asd"},
                optionsToRegister,
                switchesToRegister,
                requiredValuesToRegister);
}

TEST_F(CommandLineParser_test, FailWhenSwitchHasValueSet_MixedArguments)
{
    ::testing::Test::RecordProperty("TEST_ID", "9b1d7663-2c26-417c-a5d3-d9d4b67e104d");
    std::vector<std::string> optionsToRegister{"a-opt", "b-opt", "c-opt"};
    std::vector<std::string> switchesToRegister{"d-switch", "e-switch", "f-switch"};
    std::vector<std::string> requiredValuesToRegister{"i-req", "j-req", "k-req"};

    FailureTest({"--d-switch",
                 "ohNoASwitchValue",
                 "--f-switch",
                 "--a-opt",
                 "someA",
                 "--k-req",
                 "fSet",
                 "--i-req",
                 "asd",
                 "--j-req",
                 "fuu"},
                optionsToRegister,
                switchesToRegister,
                requiredValuesToRegister);
}

TEST_F(CommandLineParser_test, FailWhenOptionHasNoValueSet_MixedArguments)
{
    ::testing::Test::RecordProperty("TEST_ID", "e90be9ac-8839-4252-84b3-e487ceb095d0");
    std::vector<std::string> optionsToRegister{"a-opt", "b-opt", "c-opt"};
    std::vector<std::string> switchesToRegister{"d-switch", "e-switch", "f-switch"};
    std::vector<std::string> requiredValuesToRegister{"i-req", "j-req", "k-req"};

    FailureTest({"--d-switch",
                 "--f-switch",
                 "--a-opt",
                 "ohBHasNoValue",
                 "--b-opt",
                 "--k-req",
                 "fSet",
                 "--i-req",
                 "asd",
                 "--j-req",
                 "fuu"},
                optionsToRegister,
                switchesToRegister,
                requiredValuesToRegister);
}

TEST_F(CommandLineParser_test, FailWhenRequiredOptionHasNoValueSet_MixedArguments)
{
    ::testing::Test::RecordProperty("TEST_ID", "7d69a1a9-3235-42b4-a88f-dbfd2886d5ef");
    std::vector<std::string> optionsToRegister{"a-opt", "b-opt", "c-opt"};
    std::vector<std::string> switchesToRegister{"d-switch", "e-switch", "f-switch"};
    std::vector<std::string> requiredValuesToRegister{"i-req", "j-req", "k-req"};

    FailureTest({"--d-switch",
                 "--f-switch",
                 "--a-opt",
                 "aVal",
                 "--b-opt",
                 "bVal",
                 "--k-req",
                 "ohNoIHasNoValue",
                 "--i-req",
                 "--j-req",
                 "fuu"},
                optionsToRegister,
                switchesToRegister,
                requiredValuesToRegister);
}

TEST_F(CommandLineParser_test, FailWhenOptionIsNotRegistered_MixedArguments)
{
    ::testing::Test::RecordProperty("TEST_ID", "f7c314e7-f103-45be-b5f8-f96e01a2e3cc");
    std::vector<std::string> optionsToRegister{"a-opt", "b-opt", "c-opt"};
    std::vector<std::string> switchesToRegister{"d-switch", "e-switch", "f-switch"};
    std::vector<std::string> requiredValuesToRegister{"i-req", "j-req", "k-req"};

    FailureTest({"--d-switch",
                 "--f-switch",
                 "--a-opt",
                 "aVal",
                 "--nobody-knows-me",
                 "mrUnknown",
                 "--b-opt",
                 "bVal",
                 "--k-req",
                 "ohNoIHasNoValue",
                 "--i-req",
                 "someI",
                 "--j-req",
                 "fuu"},
                optionsToRegister,
                switchesToRegister,
                requiredValuesToRegister);
}

TEST_F(CommandLineParser_test, FailWhenSwitchIsNotRegistered_MixedArguments)
{
    ::testing::Test::RecordProperty("TEST_ID", "893857b5-e3f5-4a4d-8da1-ed52ff15ef33");
    std::vector<std::string> optionsToRegister{"a-opt", "b-opt", "c-opt"};
    std::vector<std::string> switchesToRegister{"d-switch", "e-switch", "f-switch"};
    std::vector<std::string> requiredValuesToRegister{"i-req", "j-req", "k-req"};

    FailureTest({"--unknown-switch",
                 "--d-switch",
                 "--f-switch",
                 "--a-opt",
                 "aVal",
                 "--b-opt",
                 "bVal",
                 "--k-req",
                 "ohNoIHasNoValue",
                 "--i-req",
                 "someI",
                 "--j-req",
                 "fuu"},
                optionsToRegister,
                switchesToRegister,
                requiredValuesToRegister);
}
/// END required, optional option and switch failure mix

TEST_F(CommandLineParser_test, DefaultValuesAreLoadedForShortOptionsOnly)
{
    ::testing::Test::RecordProperty("TEST_ID", "b3b71f74-29d6-44b2-b1b7-c92a99e3ba5c");

    constexpr int32_t DEFAULT_VALUE_1 = 4712;
    constexpr int32_t DEFAULT_VALUE_2 = 19230;

    OptionDefinition optionSet("");
    optionSet.addOptional('a', "", "", "int", Argument_t(TruncateToCapacity, std::to_string(DEFAULT_VALUE_1).c_str()));
    optionSet.addOptional('b', "", "", "int", Argument_t(TruncateToCapacity, std::to_string(DEFAULT_VALUE_2).c_str()));

    CmdArgs args({"binaryName"});
    auto retVal = parseCommandLineArguments(optionSet, args.argc, args.argv, 0);

    auto result1 = retVal.get<int32_t>("a");
    ASSERT_FALSE(result1.has_error());
    EXPECT_THAT(*result1, Eq(DEFAULT_VALUE_1));

    auto result2 = retVal.get<int32_t>("b");
    ASSERT_FALSE(result2.has_error());
    EXPECT_THAT(*result2, Eq(DEFAULT_VALUE_2));
}

TEST_F(CommandLineParser_test, DefaultValuesAreLoadedForLongOptionsOnly)
{
    ::testing::Test::RecordProperty("TEST_ID", "d3efb8c4-b546-418e-85b9-a6c2b447c0cb");

    constexpr int32_t DEFAULT_VALUE_1 = 187293;
    constexpr int32_t DEFAULT_VALUE_2 = 5512341;

    OptionDefinition optionSet("");
    optionSet.addOptional(iox::cli::NO_SHORT_OPTION,
                          "bla",
                          "",
                          "int",
                          Argument_t(TruncateToCapacity, std::to_string(DEFAULT_VALUE_1).c_str()));
    optionSet.addOptional(iox::cli::NO_SHORT_OPTION,
                          "fuu",
                          "",
                          "int",
                          Argument_t(TruncateToCapacity, std::to_string(DEFAULT_VALUE_2).c_str()));

    CmdArgs args({"binaryName"});
    auto retVal = parseCommandLineArguments(optionSet, args.argc, args.argv, 0);

    auto result1 = retVal.get<int32_t>("bla");
    ASSERT_FALSE(result1.has_error());
    EXPECT_THAT(*result1, Eq(DEFAULT_VALUE_1));

    auto result2 = retVal.get<int32_t>("fuu");
    ASSERT_FALSE(result2.has_error());
    EXPECT_THAT(*result2, Eq(DEFAULT_VALUE_2));
}

TEST_F(CommandLineParser_test, DetectMissingRequiredOptionsWithShortOptionsOnly)
{
    ::testing::Test::RecordProperty("TEST_ID", "a414b0f8-88cc-4a0a-bc73-c9be1f5dcc79");
    bool wasErrorHandlerCalled{false};
    OptionDefinition optionSet("", [&] { wasErrorHandlerCalled = true; });
    optionSet.addRequired('a', "", "", "int");
    optionSet.addRequired('b', "", "", "int");

    CmdArgs args({"binaryName"});
    auto retVal = parseCommandLineArguments(optionSet, args.argc, args.argv, 0);

    EXPECT_THAT(wasErrorHandlerCalled, Eq(true));
}

TEST_F(CommandLineParser_test, DetectMissingRequiredOptionsWithLongOptionsOnly)
{
    ::testing::Test::RecordProperty("TEST_ID", "8115efb2-9ddf-4457-9d69-526f215d974a");
    bool wasErrorHandlerCalled{false};
    OptionDefinition optionSet("", [&] { wasErrorHandlerCalled = true; });
    optionSet.addRequired(iox::cli::NO_SHORT_OPTION, "alpha", "", "int");
    optionSet.addRequired(iox::cli::NO_SHORT_OPTION, "beta", "", "int");

    CmdArgs args({"binaryName"});
    auto retVal = parseCommandLineArguments(optionSet, args.argc, args.argv, 0);

    EXPECT_THAT(wasErrorHandlerCalled, Eq(true));
}

// NOLINTJUSTIFICATION okay for tests
// NOLINTNEXTLINE(bugprone-easily-swappable-parameters, readability-function-size)
Arguments SuccessTest(const std::vector<std::string>& options,
                      const std::vector<std::string>& optionsToRegister = {},
                      const std::vector<std::string>& switchesToRegister = {},
                      const std::vector<std::string>& requiredValuesToRegister = {},
                      const uint64_t argcOffset = 1U) noexcept
{
    const char* binaryName("GloryToTheHasselToad");
    std::vector<std::string> optionVector{binaryName};
    optionVector.insert(optionVector.end(), options.begin(), options.end());
    CmdArgs args(optionVector);
    Arguments retVal;

    {
        OptionDefinition optionSet("");
        for (const auto& o : optionsToRegister)
        {
            optionSet.addOptional(
                o[0], iox::into<iox::lossy<OptionName_t>>(o), "", "int", CommandLineParser_test::defaultValue);
        }
        for (const auto& s : switchesToRegister)
        {
            optionSet.addSwitch(s[0], iox::into<iox::lossy<OptionName_t>>(s), "");
        }
        for (const auto& r : requiredValuesToRegister)
        {
            optionSet.addRequired(r[0], iox::into<iox::lossy<OptionName_t>>(r), "", "int");
        }

        {
            retVal = parseCommandLineArguments(optionSet, args.argc, args.argv, argcOffset);
        }
    }

    IOX_TESTING_EXPECT_OK();

    return retVal;
}

template <typename T>
void verifyEntry(const Arguments& options, const OptionName_t& entry, const optional<T>& value)
{
    auto result = options.get<T>(entry);

    value
        .and_then([&](auto& v) {
            // ASSERT_ does not work in function calls
            if (result.has_error())
            {
                EXPECT_TRUE(false);
                return;
            }

            EXPECT_THAT(*result, Eq(v));
        })
        .or_else([&] {
            if (!result.has_error())
            {
                EXPECT_TRUE(false);
                return;
            }

            EXPECT_THAT(result.error(), Eq(Arguments::Error::UNABLE_TO_CONVERT_VALUE));
        });
}

template <>
void verifyEntry<float>(const Arguments& options, const OptionName_t& entry, const optional<float>& value)
{
    auto result = options.get<float>(entry);

    value
        .and_then([&](auto& v) {
            // ASSERT_ does not work in function calls
            if (result.has_error())
            {
                EXPECT_TRUE(false);
                return;
            }

            EXPECT_THAT(*result, FloatEq(v));
        })
        .or_else([&] {
            if (!result.has_error())
            {
                EXPECT_TRUE(false);
                return;
            }

            EXPECT_THAT(result.error(), Eq(Arguments::Error::UNABLE_TO_CONVERT_VALUE));
        });
}

template <>
void verifyEntry<double>(const Arguments& options, const OptionName_t& entry, const optional<double>& value)
{
    auto result = options.get<double>(entry);

    value
        .and_then([&](auto& v) {
            // ASSERT_ does not work in function calls
            if (result.has_error())
            {
                EXPECT_TRUE(false);
                return;
            }

            EXPECT_THAT(*result, DoubleEq(v));
        })
        .or_else([&] {
            if (!result.has_error())
            {
                EXPECT_TRUE(false);
                return;
            }

            EXPECT_THAT(result.error(), Eq(Arguments::Error::UNABLE_TO_CONVERT_VALUE));
        });
}

/// BEGIN acquire values correctly

TEST_F(CommandLineParser_test, ReadOptionSuccessfully_SingleArgument)
{
    ::testing::Test::RecordProperty("TEST_ID", "b5e7b7b0-9423-4ea7-a1c5-83c891fc39fd");
    std::vector<std::string> optionsToRegister{"conway"};
    auto option = SuccessTest({"--conway", "gameOfLife"}, optionsToRegister);

    verifyEntry<std::string>(option, "conway", {"gameOfLife"});
}

TEST_F(CommandLineParser_test, ReadOptionSuccessfully_MultiArgument)
{
    ::testing::Test::RecordProperty("TEST_ID", "68c91bc7-b56d-4fdd-a835-32173fe7e05c");
    std::vector<std::string> optionsToRegister{"conway", "tungsten", "moon"};
    auto option = SuccessTest({"--moon", "bright", "--conway", "gameOfLife", "--tungsten", "heavy"}, optionsToRegister);

    verifyEntry<std::string>(option, "conway", {"gameOfLife"});
    verifyEntry<std::string>(option, "moon", {"bright"});
    verifyEntry<std::string>(option, "tungsten", {"heavy"});
}

TEST_F(CommandLineParser_test, ReadOptionSuccessfully_MultiArgument_ShortOption)
{
    ::testing::Test::RecordProperty("TEST_ID", "6113b30a-1274-4b2b-b6e2-45cfad493b45");
    std::vector<std::string> optionsToRegister{"conway", "tungsten", "moon"};
    auto option = SuccessTest({"-m", "bright", "-c", "gameOfLife", "-t", "heavy"}, optionsToRegister);

    verifyEntry<std::string>(option, "c", {"gameOfLife"});
    verifyEntry<std::string>(option, "m", {"bright"});
    verifyEntry<std::string>(option, "t", {"heavy"});
}

TEST_F(CommandLineParser_test, ReadOptionSuccessfully_PartialSet)
{
    ::testing::Test::RecordProperty("TEST_ID", "7432b080-6d18-424b-bbe1-5c9293e8584a");
    std::vector<std::string> optionsToRegister{"conway", "tungsten", "moon"};
    auto option = SuccessTest({"-m", "bright"}, optionsToRegister);

    verifyEntry<std::string>(option, "moon", {"bright"});
    verifyEntry<std::string>(option, "conway", {defaultValue.c_str()});
    verifyEntry<std::string>(option, "tungsten", {defaultValue.c_str()});
}

TEST_F(CommandLineParser_test, ReadOptionSuccessfully_Offset)
{
    ::testing::Test::RecordProperty("TEST_ID", "58a5f953-f33f-48a6-ac48-d60006726cb6");
    std::vector<std::string> optionsToRegister{"conway", "tungsten", "moon"};
    constexpr uint64_t ARGC_OFFSET = 5U;
    auto option =
        SuccessTest({"whatever", "bright", "-t", "heavy", "-c", "gameOfLife"}, optionsToRegister, {}, {}, ARGC_OFFSET);

    verifyEntry<std::string>(option, "moon", {defaultValue.c_str()});
    verifyEntry<std::string>(option, "conway", {"gameOfLife"});
    verifyEntry<std::string>(option, "tungsten", {defaultValue.c_str()});
}

TEST_F(CommandLineParser_test, ReadRequiredValueSuccessfully_SingleArgument)
{
    ::testing::Test::RecordProperty("TEST_ID", "8397de7f-5a1b-49d7-80bd-30a28f143efa");
    std::vector<std::string> optionsToRegister{};
    std::vector<std::string> switchesToRegister{};
    std::vector<std::string> requiredValuesToRegister{"fuubar"};
    auto option = SuccessTest({"--fuubar", "ohFuBa"}, optionsToRegister, switchesToRegister, requiredValuesToRegister);

    verifyEntry<std::string>(option, "fuubar", {"ohFuBa"});
}

TEST_F(CommandLineParser_test, ReadRequiredValueSuccessfully_MultiArgument)
{
    ::testing::Test::RecordProperty("TEST_ID", "fa54f70f-be0d-426a-a00b-4d4d3a57a90d");
    std::vector<std::string> optionsToRegister{};
    std::vector<std::string> switchesToRegister{};
    std::vector<std::string> requiredValuesToRegister{"fuubar", "c64", "amiga"};
    auto option = SuccessTest({"--fuubar", "ohFuBa", "--amiga", "Os2 Warp", "--c64", "cobra"},
                              optionsToRegister,
                              switchesToRegister,
                              requiredValuesToRegister);

    verifyEntry<std::string>(option, "fuubar", {"ohFuBa"});
    verifyEntry<std::string>(option, "amiga", {"Os2 Warp"});
    verifyEntry<std::string>(option, "c64", {"cobra"});
}

TEST_F(CommandLineParser_test, ReadRequiredValueSuccessfully_MultiArgument_ShortOption)
{
    ::testing::Test::RecordProperty("TEST_ID", "aeefeddf-7578-4451-a266-116219fdb150");
    std::vector<std::string> optionsToRegister{};
    std::vector<std::string> switchesToRegister{};
    std::vector<std::string> requiredValuesToRegister{"fuubar", "c64", "amiga"};
    auto option = SuccessTest({"-f", "ohFuBa", "-a", "Os2 Warp", "-c", "cobra"},
                              optionsToRegister,
                              switchesToRegister,
                              requiredValuesToRegister);

    verifyEntry<std::string>(option, "f", {"ohFuBa"});
    verifyEntry<std::string>(option, "a", {"Os2 Warp"});
    verifyEntry<std::string>(option, "c", {"cobra"});
}

TEST_F(CommandLineParser_test, ReadRequiredValueSuccessfully_Offset)
{
    ::testing::Test::RecordProperty("TEST_ID", "f84a9ad7-c6d9-4b56-bd18-a0d1bdbcde2a");
    std::vector<std::string> optionsToRegister{};
    std::vector<std::string> switchesToRegister{};
    std::vector<std::string> requiredValuesToRegister{"fuubar", "c64", "amiga"};
    constexpr uint64_t ARGC_OFFSET = 3U;
    auto option = SuccessTest({"-f", "iWillNotBeParsed", "-f", "ohFuBa", "-a", "Os2 Warp", "-c", "cobra"},
                              optionsToRegister,
                              switchesToRegister,
                              requiredValuesToRegister,
                              ARGC_OFFSET);

    verifyEntry<std::string>(option, "f", {"ohFuBa"});
    verifyEntry<std::string>(option, "a", {"Os2 Warp"});
    verifyEntry<std::string>(option, "c", {"cobra"});
}

TEST_F(CommandLineParser_test, ReadSwitchValueSuccessfullyWhenSet_SingleArgument)
{
    ::testing::Test::RecordProperty("TEST_ID", "08cfe8c0-6b37-418b-9bb0-265ba4419513");
    std::vector<std::string> optionsToRegister{};
    std::vector<std::string> switchesToRegister{"light"};
    auto option = SuccessTest({"--light"}, optionsToRegister, switchesToRegister);

    EXPECT_TRUE(option.isSwitchSet("light"));
}

TEST_F(CommandLineParser_test, ReadSwitchValueSuccessfullyWhenSet_MultiArgument)
{
    ::testing::Test::RecordProperty("TEST_ID", "1bdba94b-1f42-4627-b847-16a2b49c08b0");
    std::vector<std::string> optionsToRegister{};
    std::vector<std::string> switchesToRegister{"light", "fridge", "muu"};
    auto option = SuccessTest({"--light", "--fridge", "--muu"}, optionsToRegister, switchesToRegister);

    EXPECT_TRUE(option.isSwitchSet("light"));
    EXPECT_TRUE(option.isSwitchSet("fridge"));
    EXPECT_TRUE(option.isSwitchSet("muu"));
}

TEST_F(CommandLineParser_test, ReadSwitchValueSuccessfullyWhenSet_MultiArgument_ShortOption)
{
    ::testing::Test::RecordProperty("TEST_ID", "85a0e72c-3eb0-416d-8e4f-7b49982ecaf8");
    std::vector<std::string> optionsToRegister{};
    std::vector<std::string> switchesToRegister{"light", "fridge", "muu"};
    auto option = SuccessTest({"-l", "-f", "-m"}, optionsToRegister, switchesToRegister);

    EXPECT_TRUE(option.isSwitchSet("l"));
    EXPECT_TRUE(option.isSwitchSet("f"));
    EXPECT_TRUE(option.isSwitchSet("m"));
}

TEST_F(CommandLineParser_test, ReadSwitchValueSuccessfullyWhenSet_PartialSet)
{
    ::testing::Test::RecordProperty("TEST_ID", "9f203ddd-2505-40b4-84b1-2246c4e7cf3a");
    std::vector<std::string> optionsToRegister{};
    std::vector<std::string> switchesToRegister{"light", "fridge", "muu"};
    auto option = SuccessTest({"-l"}, optionsToRegister, switchesToRegister);

    EXPECT_TRUE(option.isSwitchSet("light"));
    EXPECT_FALSE(option.isSwitchSet("fridge"));
    EXPECT_FALSE(option.isSwitchSet("muu"));
}

TEST_F(CommandLineParser_test, ReadSwitchValueSuccessfullyWhenSet_Offset)
{
    ::testing::Test::RecordProperty("TEST_ID", "e90f18dc-32c7-4b39-adb7-17e039a165d8");
    std::vector<std::string> optionsToRegister{};
    std::vector<std::string> switchesToRegister{"light", "fridge", "muu"};
    constexpr uint64_t ARGC_OFFSET = 2U;
    auto option =
        SuccessTest({"----unknown-dont-care", "-f", "-m"}, optionsToRegister, switchesToRegister, {}, ARGC_OFFSET);

    EXPECT_FALSE(option.isSwitchSet("light"));
    EXPECT_TRUE(option.isSwitchSet("fridge"));
    EXPECT_TRUE(option.isSwitchSet("muu"));
}
/// END acquire values correctly

/// BEGIN acquire mixed values correctly

TEST_F(CommandLineParser_test, ReadMixedValueSuccessfully)
{
    ::testing::Test::RecordProperty("TEST_ID", "eb1c565d-a10b-4a80-b6e4-1aac54b96324");
    std::vector<std::string> optionsToRegister{"a-opt", "b-opt", "c-opt"};
    std::vector<std::string> switchesToRegister{"d-switch", "e-switch", "f-switch"};
    std::vector<std::string> requiredValuesToRegister{"g-req", "i-req", "j-req"};
    auto option = SuccessTest({"--a-opt",
                               "oh-my-blah",
                               "--d-switch",
                               "--i-req",
                               "someI",
                               "--j-req",
                               "someJ",
                               "--f-switch",
                               "--g-req",
                               "someG"},
                              optionsToRegister,
                              switchesToRegister,
                              requiredValuesToRegister);

    verifyEntry<std::string>(option, "a-opt", {"oh-my-blah"});
    verifyEntry<std::string>(option, "b-opt", {defaultValue.c_str()});
    verifyEntry<std::string>(option, "c-opt", {defaultValue.c_str()});
    verifyEntry<std::string>(option, "i-req", {"someI"});
    verifyEntry<std::string>(option, "j-req", {"someJ"});
    verifyEntry<std::string>(option, "g-req", {"someG"});

    EXPECT_TRUE(option.isSwitchSet("d-switch"));
    EXPECT_FALSE(option.isSwitchSet("e-switch"));
    EXPECT_TRUE(option.isSwitchSet("f-switch"));
}

TEST_F(CommandLineParser_test, ReadMixedValueSuccessfully_ShortOption)
{
    ::testing::Test::RecordProperty("TEST_ID", "a250997f-7bc8-4ba6-a860-b8d650f59f39");
    std::vector<std::string> optionsToRegister{"a-opt", "b-opt", "c-opt"};
    std::vector<std::string> switchesToRegister{"d-switch", "e-switch", "f-switch"};
    std::vector<std::string> requiredValuesToRegister{"g-req", "i-req", "j-req"};
    auto option = SuccessTest({"-a", "anotherA", "-b", "someB", "-e", "-i", "blaI", "-j", "blaJ", "-g", "blaG"},
                              optionsToRegister,
                              switchesToRegister,
                              requiredValuesToRegister);

    verifyEntry<std::string>(option, "a-opt", {"anotherA"});
    verifyEntry<std::string>(option, "b-opt", {"someB"});
    verifyEntry<std::string>(option, "c-opt", {defaultValue.c_str()});
    verifyEntry<std::string>(option, "i-req", {"blaI"});
    verifyEntry<std::string>(option, "j-req", {"blaJ"});
    verifyEntry<std::string>(option, "g-req", {"blaG"});

    EXPECT_FALSE(option.isSwitchSet("d-switch"));
    EXPECT_TRUE(option.isSwitchSet("e-switch"));
    EXPECT_FALSE(option.isSwitchSet("f-switch"));
}

TEST_F(CommandLineParser_test, ReadMixedValueSuccessfully_Offset)
{
    ::testing::Test::RecordProperty("TEST_ID", "3bb7943e-d446-4f83-8a6a-5cc9de997359");
    std::vector<std::string> optionsToRegister{"a-opt", "b-opt", "c-opt"};
    std::vector<std::string> switchesToRegister{"d-switch", "e-switch", "f-switch"};
    std::vector<std::string> requiredValuesToRegister{"g-req", "i-req", "j-req"};
    constexpr uint64_t ARGC_OFFSET = 3U;
    auto option = SuccessTest({"-a", "anotherA", "-b", "someB", "-e", "-i", "blaI", "-j", "blaJ", "-g", "blaG"},
                              optionsToRegister,
                              switchesToRegister,
                              requiredValuesToRegister,
                              ARGC_OFFSET);

    verifyEntry<std::string>(option, "a-opt", {defaultValue.c_str()});
    verifyEntry<std::string>(option, "b-opt", {"someB"});
    verifyEntry<std::string>(option, "c-opt", {defaultValue.c_str()});
    verifyEntry<std::string>(option, "i-req", {"blaI"});
    verifyEntry<std::string>(option, "j-req", {"blaJ"});
    verifyEntry<std::string>(option, "g-req", {"blaG"});

    EXPECT_FALSE(option.isSwitchSet("d-switch"));
    EXPECT_TRUE(option.isSwitchSet("e-switch"));
    EXPECT_FALSE(option.isSwitchSet("f-switch"));
}
/// END acquire mixed values correctly

/// BEGIN conversions
TEST_F(CommandLineParser_test, SuccessfulConversionToNumbers)
{
    ::testing::Test::RecordProperty("TEST_ID", "f3ffc60c-bcf8-4c4c-99c8-16f81625893f");
    std::vector<std::string> optionsToRegister{"a-opt", "b-opt", "c-opt"};
    std::vector<std::string> switchesToRegister{};
    std::vector<std::string> requiredValuesToRegister{"g-req", "i-req", "j-req"};

    auto option = SuccessTest({"--a-opt", "123", "--i-req", "-456", "--j-req", "123.123", "--g-req", "-891.19012"},
                              optionsToRegister,
                              switchesToRegister,
                              requiredValuesToRegister);

    verifyEntry<uint8_t>(option, "a-opt", {123});
    verifyEntry<int16_t>(option, "i-req", {-456});
    verifyEntry<float>(option, "j-req", {123.123F});
    verifyEntry<double>(option, "g-req", {-891.19012});
}

TEST_F(CommandLineParser_test, MultipleConversionFailures)
{
    ::testing::Test::RecordProperty("TEST_ID", "0a4bc316-7a0a-4524-b3b4-1bcf30f87380");
    std::vector<std::string> optionsToRegister{"a-opt", "b-opt", "c-opt"};
    std::vector<std::string> switchesToRegister{};
    std::vector<std::string> requiredValuesToRegister{"g-req", "i-req", "j-req"};

    auto option =
        SuccessTest({"--a-opt", "-123", "--i-req", "123123123", "--j-req", "iAmNotAFloat", "--g-req", "-891.19012"},
                    optionsToRegister,
                    switchesToRegister,
                    requiredValuesToRegister);

    verifyEntry<uint8_t>(option, "a-opt", nullopt);
    verifyEntry<int16_t>(option, "i-req", nullopt);
    verifyEntry<float>(option, "j-req", nullopt);
    verifyEntry<int64_t>(option, "g-req", nullopt);
}
/// END conversions

} // namespace
