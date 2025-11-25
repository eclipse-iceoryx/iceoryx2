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

#include "iox/cli/option_definition.hpp"
#include "iox/detail/hoofs_error_reporting.hpp"
#include "iox/function.hpp"
#include "iox/optional.hpp"
#include "test_cli_command_line_common.hpp"

#include "test.hpp"

#include <cstdlib>
#include <iostream>
#include <memory>
#include <string>
#include <vector>

namespace
{
using namespace ::testing;
using namespace iox;
using namespace iox::cli;

/// All the success tests are handled indirectly in the CommandLineArgumentParser_test
/// where every combination of short and long option is parsed and verified
class OptionDefinition_test : public Test
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

    iox::optional<OutBuffer> outputBuffer;
    uint64_t numberOfErrorCallbackCalls = 0U;
    iox::function<void()> errorCallback = [this] { ++numberOfErrorCallbackCalls; };
    static Argument_t defaultValue;
};
Argument_t OptionDefinition_test::defaultValue = "DEFAULT VALUE";

TEST_F(OptionDefinition_test, AddingTheSameShortOptionLeadsToExit)
{
    ::testing::Test::RecordProperty("TEST_ID", "f1340876-e3f6-4f62-b0f3-4e9551a5f67a");
    OptionDefinition optionSet("", errorCallback);
    optionSet.addOptional('c', "firstEntry", "", "", "");

    optionSet.addSwitch('c', "duplicateShortOption", "");
    EXPECT_THAT(numberOfErrorCallbackCalls, Eq(1));

    optionSet.addOptional('c', "duplicateShortOption", "", "", "");
    EXPECT_THAT(numberOfErrorCallbackCalls, Eq(2));

    optionSet.addRequired('c', "duplicateShortOption", "", "");
    EXPECT_THAT(numberOfErrorCallbackCalls, Eq(3));
}

TEST_F(OptionDefinition_test, AddingTheSameLongOptionLeadsToExit)
{
    ::testing::Test::RecordProperty("TEST_ID", "076b8877-e3fc-46f7-851b-d3e7953f67d6");
    OptionDefinition optionSet("", errorCallback);
    optionSet.addSwitch('c', "duplicate", "");

    optionSet.addSwitch('x', "duplicate", "");
    EXPECT_THAT(numberOfErrorCallbackCalls, Eq(1));

    optionSet.addOptional('x', "duplicate", "", "", "");
    EXPECT_THAT(numberOfErrorCallbackCalls, Eq(2));

    optionSet.addRequired('x', "duplicate", "", "");
    EXPECT_THAT(numberOfErrorCallbackCalls, Eq(3));
}

TEST_F(OptionDefinition_test, AddingOptionWithSameShortAndLongNameLeadsToExit)
{
    ::testing::Test::RecordProperty("TEST_ID", "4e01ed47-473d-4915-aed2-60aacce37de8");
    OptionDefinition optionSet("", errorCallback);
    optionSet.addRequired('d', "duplicate", "", "");

    optionSet.addSwitch('d', "duplicate", "");
    EXPECT_THAT(numberOfErrorCallbackCalls, Eq(1));

    optionSet.addOptional('d', "duplicate", "", "", "");
    EXPECT_THAT(numberOfErrorCallbackCalls, Eq(2));

    optionSet.addRequired('d', "duplicate", "", "");
    EXPECT_THAT(numberOfErrorCallbackCalls, Eq(3));
}

TEST_F(OptionDefinition_test, AddingSwitchWithDashAsShortOptionLeadsToFailure)
{
    ::testing::Test::RecordProperty("TEST_ID", "5c6558ec-ecd9-47e9-b396-593445cef68f");
    OptionDefinition optionSet("", errorCallback);
    optionSet.addSwitch('-', "", "");

    EXPECT_THAT(numberOfErrorCallbackCalls, Eq(1));
}

TEST_F(OptionDefinition_test, AddingOptionalValueWithDashAsShortOptionLeadsToFailure)
{
    ::testing::Test::RecordProperty("TEST_ID", "8afd403b-9a77-4bde-92df-0200d4fb661b");
    OptionDefinition optionSet("", errorCallback);
    optionSet.addOptional('-', "", "", "", "");

    EXPECT_THAT(numberOfErrorCallbackCalls, Eq(1));
}

TEST_F(OptionDefinition_test, AddingRequiredValueWithDashAsShortOptionLeadsToFailure)
{
    ::testing::Test::RecordProperty("TEST_ID", "04e358dd-6ef4-48e4-988e-ee1d0514632b");
    OptionDefinition optionSet("", errorCallback);
    optionSet.addRequired('-', "", "", "");

    EXPECT_THAT(numberOfErrorCallbackCalls, Eq(1));
}

TEST_F(OptionDefinition_test, AddingSwitchWithDashStartingLongOptionLeadsToFailure)
{
    ::testing::Test::RecordProperty("TEST_ID", "62c0882a-7055-4a74-9dd9-8505d72da1e0");
    OptionDefinition optionSet("", errorCallback);
    optionSet.addSwitch('a', "-oh-no-i-start-with-dash", "");

    EXPECT_THAT(numberOfErrorCallbackCalls, Eq(1));
}

TEST_F(OptionDefinition_test, AddingOptionalValueWithDashStartingLongOptionLeadsToFailure)
{
    ::testing::Test::RecordProperty("TEST_ID", "69c975d1-57d3-429a-b894-7ff1efa9f473");
    OptionDefinition optionSet("", errorCallback);
    optionSet.addOptional('c', "-whoopsie-there-is-a-dash", "", "", "");

    EXPECT_THAT(numberOfErrorCallbackCalls, Eq(1));
}

TEST_F(OptionDefinition_test, AddingRequiredValueWithDashStartingLongOptionLeadsToFailure)
{
    ::testing::Test::RecordProperty("TEST_ID", "43929047-1051-45cd-8a13-ebf8ea8c4e26");
    OptionDefinition optionSet("", errorCallback);
    optionSet.addRequired('b', "-dash-is-all-i-need", "", "");

    EXPECT_THAT(numberOfErrorCallbackCalls, Eq(1));
}

TEST_F(OptionDefinition_test, AddingSwitchWithEmptyShortAndLongOptionLeadsToFailure)
{
    ::testing::Test::RecordProperty("TEST_ID", "f1aa3314-0355-43d8-85b3-b2e7d604440e");
    OptionDefinition optionSet("", errorCallback);
    optionSet.addSwitch(NO_SHORT_OPTION, "", "");

    EXPECT_THAT(numberOfErrorCallbackCalls, Eq(1));
}

TEST_F(OptionDefinition_test, AddingOptionalWithEmptyShortAndLongOptionLeadsToFailure)
{
    ::testing::Test::RecordProperty("TEST_ID", "ee6866b7-a4f8-4406-ab50-ba0d1b798696");
    OptionDefinition optionSet("", errorCallback);
    optionSet.addOptional(NO_SHORT_OPTION, "", "", "", "");

    EXPECT_THAT(numberOfErrorCallbackCalls, Eq(1));
}

TEST_F(OptionDefinition_test, AddingRequiredValueWithEmptyShortAndLongOptionLeadsToFailure)
{
    ::testing::Test::RecordProperty("TEST_ID", "bf33281b-de11-4482-8bc4-1e443c5b3bc1");
    OptionDefinition optionSet("", errorCallback);
    optionSet.addRequired(NO_SHORT_OPTION, "", "", "");

    EXPECT_THAT(numberOfErrorCallbackCalls, Eq(1));
}
} // namespace
