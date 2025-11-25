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

#include "iox/cli_definition.hpp"
#include "iox/detail/hoofs_error_reporting.hpp"
#include "iox/optional.hpp"
#include "test.hpp"
#include "test_cli_command_line_common.hpp"

#include <cstdlib>
#include <iostream>
#include <memory>
#include <string>
#include <vector>

namespace
{
using namespace ::testing;
using namespace iox;

/// This test is only some kind of compilation test to verify that the
/// command line parser macros are working and connecting everything together correctly
/// The actual test of the command line parser can be found in
/// test_cli_command_line_argument_parser.cpp
class CliDefinition_test : public Test
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
};

struct CliDefinitionSut
{
    // NOLINTJUSTIFICATION hidden behind a macro
    // NOLINTNEXTLINE(readability-function-size)
    IOX_CLI_DEFINITION(CliDefinitionSut);

    IOX_CLI_OPTIONAL(string<100>, stringValue1, {"default value"}, 's', {"string-value-1"}, {"some description"});
    IOX_CLI_OPTIONAL(string<100>, stringValue2, {"some other value"}, 't', {"string-value-2"}, {"some description"});
    IOX_CLI_OPTIONAL(int64_t, optionalInt1, 123, 'i', {"int-value-1"}, {"some description"});
    IOX_CLI_OPTIONAL(int64_t, optionalInt2, 456, 'j', {"int-value-2"}, {"some description"});
    IOX_CLI_OPTIONAL(uint8_t, optionalUint1, 123, 'u', {"uint-value-1"}, {"some description"});
    IOX_CLI_OPTIONAL(uint8_t, optionalUint2, 212, 'v', {"uint-value-2"}, {"some description"});

    IOX_CLI_SWITCH(lightSwitch1, 'l', "light-switch-1", "do some stuff - some description");
    IOX_CLI_SWITCH(lightSwitch2, 'm', "light-switch-2", "do some stuff - some description");

    IOX_CLI_REQUIRED(string<100>, requiredString, 'r', "required-string", "some description");
    IOX_CLI_REQUIRED(float, requiredFloat, 'b', "required-float", "some description");
    IOX_CLI_REQUIRED(uint16_t, requiredUint, 'c', "required-uint", "some description");
};

TEST_F(CliDefinition_test, OnlyRequiredValuesSetsRemainingValuesToDefault)
{
    ::testing::Test::RecordProperty("TEST_ID", "451701b8-061f-4e30-9beb-1c09c7e6bc1b");
    CmdArgs args(
        {"myBinaryName", "--required-string", "bluubb", "--required-float", "123.456", "--required-uint", "12"});
    auto sut = CliDefinitionSut::parse(args.argc, args.argv, "My program description");

    EXPECT_THAT(sut.binaryName(), StrEq("myBinaryName"));

    // default values
    EXPECT_THAT(sut.stringValue1().c_str(), StrEq("default value"));
    EXPECT_THAT(sut.stringValue2().c_str(), StrEq("some other value"));
    EXPECT_THAT(sut.optionalInt1(), Eq(123));
    EXPECT_THAT(sut.optionalInt2(), Eq(456));
    EXPECT_THAT(sut.optionalUint1(), Eq(123));
    EXPECT_THAT(sut.optionalUint2(), Eq(212));
    EXPECT_THAT(sut.lightSwitch1(), Eq(false));
    EXPECT_THAT(sut.lightSwitch2(), Eq(false));

    EXPECT_THAT(sut.requiredString().c_str(), StrEq("bluubb"));
    EXPECT_THAT(sut.requiredFloat(), FloatEq(123.456F));
    EXPECT_THAT(sut.requiredUint(), Eq(12));
}

TEST_F(CliDefinition_test, AllValuesViaCommandLineArgumentDefinitionAreSetCorrectly)
{
    ::testing::Test::RecordProperty("TEST_ID", "0478575e-8eb4-4983-93bd-199d222e706e");
    CmdArgs args({"anotherOneBitesTheDust",
                  "--required-string",
                  "schnappidububa",
                  "--required-float",
                  "456.123",
                  "--required-uint",
                  "1212",
                  "--string-value-1",
                  "flatterdude",
                  "--string-value-2",
                  "evilhuhn",
                  "--int-value-1",
                  "4711123",
                  "--int-value-2",
                  "810456",
                  "--uint-value-1",
                  "39",
                  "--uint-value-2",
                  "31",
                  "--light-switch-1",
                  "--light-switch-2"});
    auto sut = CliDefinitionSut::parse(args.argc, args.argv, "My program description");

    EXPECT_THAT(sut.binaryName(), StrEq("anotherOneBitesTheDust"));

    EXPECT_THAT(sut.stringValue1().c_str(), StrEq("flatterdude"));
    EXPECT_THAT(sut.stringValue2().c_str(), StrEq("evilhuhn"));
    EXPECT_THAT(sut.optionalInt1(), Eq(4711123));
    EXPECT_THAT(sut.optionalInt2(), Eq(810456));
    EXPECT_THAT(sut.optionalUint1(), Eq(39));
    EXPECT_THAT(sut.optionalUint2(), Eq(31));
    EXPECT_THAT(sut.lightSwitch1(), Eq(true));
    EXPECT_THAT(sut.lightSwitch2(), Eq(true));

    EXPECT_THAT(sut.requiredString().c_str(), StrEq("schnappidububa"));
    EXPECT_THAT(sut.requiredFloat(), FloatEq(456.123F));
    EXPECT_THAT(sut.requiredUint(), Eq(1212));
}

TEST_F(CliDefinition_test, AllValuesViaCommandLineArgumentDefinitionAndShortcutAreSetCorrectly)
{
    ::testing::Test::RecordProperty("TEST_ID", "0c9abe4d-47ab-469a-a0fe-eff03a7aff37");
    CmdArgs args({"noOneBitesHypnotoad",
                  "-r",
                  "AllYouNeedIsHorst",
                  "-b",
                  "810.123",
                  "-c",
                  "31415",
                  "-s",
                  "DoNotTouchTheFishy",
                  "-t",
                  "NoLittleTouchyFishy",
                  "-i",
                  "3",
                  "-j",
                  "4",
                  "-u",
                  "5",
                  "-v",
                  "25",
                  "-l",
                  "-m"});
    auto sut = CliDefinitionSut::parse(args.argc, args.argv, "My program description");

    EXPECT_THAT(sut.binaryName(), StrEq("noOneBitesHypnotoad"));

    EXPECT_THAT(sut.stringValue1().c_str(), StrEq("DoNotTouchTheFishy"));
    EXPECT_THAT(sut.stringValue2().c_str(), StrEq("NoLittleTouchyFishy"));
    EXPECT_THAT(sut.optionalInt1(), Eq(3));
    EXPECT_THAT(sut.optionalInt2(), Eq(4));
    EXPECT_THAT(sut.optionalUint1(), Eq(5));
    EXPECT_THAT(sut.optionalUint2(), Eq(25));
    EXPECT_THAT(sut.lightSwitch1(), Eq(true));
    EXPECT_THAT(sut.lightSwitch2(), Eq(true));

    EXPECT_THAT(sut.requiredString().c_str(), StrEq("AllYouNeedIsHorst"));
    EXPECT_THAT(sut.requiredFloat(), FloatEq(810.123F));
    EXPECT_THAT(sut.requiredUint(), Eq(31415));
}
} // namespace
