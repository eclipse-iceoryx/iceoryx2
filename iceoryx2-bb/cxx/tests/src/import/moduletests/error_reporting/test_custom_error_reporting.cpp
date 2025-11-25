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

#include "iox/error_reporting/custom/error_kind.hpp"
#include "iox/error_reporting/error_kind.hpp"
#include "iox/error_reporting/source_location.hpp"
#include "iox/error_reporting/violation.hpp"
#include "moduletests/error_reporting/module_a/errors.hpp"
#include "test.hpp"
#include <gtest/gtest.h>

#include "iceoryx_hoofs/testing/error_reporting/testing_support.hpp"
#include "iox/error_reporting/custom/error_reporting.hpp"

#include "module_a/errors.hpp"

namespace
{

using namespace ::testing;
using namespace iox::er;

constexpr auto ERROR_CODE{module_a::errors::Code::OutOfBounds};
constexpr module_a::errors::Error ERROR_MODULE{ERROR_CODE};

// Here we test the custom API that the public API forwards to.
// To observe the side effects, this requires using the TestingErrorHandler (similar to the public API).

class ErrorReporting_test : public Test
{
  public:
    void SetUp() override
    {
    }

    void TearDown() override
    {
    }
};

TEST_F(ErrorReporting_test, panicWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "27f25cec-c815-4541-9f7d-fd2aa02474c1");

    auto f = []() { panic(); };

    iox::testing::runInTestThread(f);

    IOX_TESTING_EXPECT_PANIC();
}

TEST_F(ErrorReporting_test, panicWithLocationWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "5aca0c31-1c1b-4004-bd41-b4b400258c12");

    auto f = []() { panic(IOX_CURRENT_SOURCE_LOCATION); };

    iox::testing::runInTestThread(f);

    IOX_TESTING_EXPECT_PANIC();
}

TEST_F(ErrorReporting_test, panicWithMessageWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "f0e44332-ea9b-4041-88f4-8155ccf7538d");

    auto f = []() { panic(IOX_CURRENT_SOURCE_LOCATION, "message"); };

    iox::testing::runInTestThread(f);

    IOX_TESTING_EXPECT_PANIC();
}

TEST_F(ErrorReporting_test, reportNonFatalErrorWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "1a1cec1b-5297-487a-bb95-e80af99886b6");

    auto f = []() {
        constexpr const char* STRINGIFIED_CONDITION{""};
        report(IOX_CURRENT_SOURCE_LOCATION, RUNTIME_ERROR, ERROR_MODULE, STRINGIFIED_CONDITION);
    };

    iox::testing::runInTestThread(f);

    IOX_TESTING_EXPECT_NO_PANIC();
    IOX_TESTING_EXPECT_ERROR(ERROR_CODE);
}

TEST_F(ErrorReporting_test, reportFatalErrorWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "");

    auto f = []() {
        constexpr const char* STRINGIFIED_CONDITION{""};
        report(IOX_CURRENT_SOURCE_LOCATION, FATAL, ERROR_MODULE, STRINGIFIED_CONDITION);
    };

    iox::testing::runInTestThread(f);

    // panic is not required at this level as we cannot trust the custom API to enforce it
    // While we could also call panic in the custom API, there should only be one decison point
    // for it at a higher level
    IOX_TESTING_EXPECT_ERROR(ERROR_CODE);
}

TEST_F(ErrorReporting_test, reportAssertViolatonWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "feb63aa0-1921-408a-a887-abbb99522b31");

    auto f = []() {
        auto v = Violation::createAssertViolation();
        constexpr const char* STRINGIFIED_CONDITION{""};
        report(IOX_CURRENT_SOURCE_LOCATION, ASSERT_VIOLATION, v, STRINGIFIED_CONDITION);
    };

    iox::testing::runInTestThread(f);

    IOX_TESTING_EXPECT_ASSERT_VIOLATION();
}

// the message is printed but otherwise lost, so we cannot check for it
TEST_F(ErrorReporting_test, reportAssertViolatonWithMessageWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "9228c696-d555-49c5-ade1-b65d16159e8c");

    auto f = []() {
        auto v = Violation::createAssertViolation();
        constexpr const char* STRINGIFIED_CONDITION{""};
        report(IOX_CURRENT_SOURCE_LOCATION, ASSERT_VIOLATION, v, STRINGIFIED_CONDITION, "message");
    };

    iox::testing::runInTestThread(f);

    IOX_TESTING_EXPECT_ASSERT_VIOLATION();
}

TEST_F(ErrorReporting_test, reportEnforceViolatonWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "f866b43a-3a88-4097-adde-4704fc1a5e8f");

    auto f = []() {
        auto v = Violation::createEnforceViolation();
        constexpr const char* STRINGIFIED_CONDITION{""};
        report(IOX_CURRENT_SOURCE_LOCATION, ENFORCE_VIOLATION, v, STRINGIFIED_CONDITION);
    };

    iox::testing::runInTestThread(f);

    IOX_TESTING_EXPECT_ENFORCE_VIOLATION();
}

// the message is printed but otherwise lost, so we cannot check for it
TEST_F(ErrorReporting_test, reportEnforceViolatonWithMessageWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "1cccd0f7-c944-4904-bf64-6f575ea13b85");

    auto f = []() {
        auto v = Violation::createEnforceViolation();
        constexpr const char* STRINGIFIED_CONDITION{""};
        report(IOX_CURRENT_SOURCE_LOCATION, ENFORCE_VIOLATION, v, STRINGIFIED_CONDITION, "message");
    };

    iox::testing::runInTestThread(f);

    IOX_TESTING_EXPECT_ENFORCE_VIOLATION();
}

} // namespace
