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
#include "test.hpp"
#include <gtest/gtest.h>

#include "iox/assertions.hpp"

// some dummy modules under test
#include "module_a/error_reporting.hpp"
#include "module_b/error_reporting.hpp"

// simplifies checking for errors during test execution
#include "iceoryx_hoofs/testing/error_reporting/testing_support.hpp"

#include <iostream>

namespace
{
using namespace ::testing;
using namespace iox::er;
using namespace iox::testing;

using MyErrorA = module_a::errors::Error;
using MyCodeA = module_a::errors::Code;

using MyErrorB = module_b::errors::Error;
using MyCodeB = module_b::errors::Code;

class ErrorReportingMacroApi_test : public Test
{
  public:
    void SetUp() override
    {
    }

    void TearDown() override
    {
    }
};

TEST_F(ErrorReportingMacroApi_test, panicWithoutMessage)
{
    ::testing::Test::RecordProperty("TEST_ID", "a55f00f1-c89d-4d4d-90ea-6ca510ad3942");
    auto f = []() { IOX_PANIC(""); };

    runInTestThread(f);

    IOX_TESTING_EXPECT_PANIC();
}

TEST_F(ErrorReportingMacroApi_test, panicWithMessage)
{
    ::testing::Test::RecordProperty("TEST_ID", "cfbaf43b-de11-4858-ab86-ae3ae3fac2fe");
    auto f = []() { IOX_PANIC("message"); };

    runInTestThread(f);

    IOX_TESTING_EXPECT_PANIC();
}

TEST_F(ErrorReportingMacroApi_test, reportNonFatal)
{
    ::testing::Test::RecordProperty("TEST_ID", "408a30b5-2764-4792-a5c6-97bff74f8902");
    auto f = []() { IOX_REPORT(MyCodeA::OutOfBounds, RUNTIME_ERROR); };

    runInTestThread(f);

    IOX_TESTING_EXPECT_NO_PANIC(); // but also not OK as there is an error!
    IOX_TESTING_EXPECT_ERROR(MyCodeA::OutOfBounds);
}

TEST_F(ErrorReportingMacroApi_test, reportFatal)
{
    ::testing::Test::RecordProperty("TEST_ID", "a65c28fb-8cf6-4b9b-96b9-079ee9cb6b88");
    auto f = []() { IOX_REPORT_FATAL(MyCodeA::OutOfBounds); };

    runInTestThread(f);

    IOX_TESTING_EXPECT_PANIC();
    IOX_TESTING_EXPECT_ERROR(MyCodeA::OutOfBounds);
}

TEST_F(ErrorReportingMacroApi_test, reportConditionalError)
{
    ::testing::Test::RecordProperty("TEST_ID", "d95fe843-5e1b-422f-bd15-a791b639b43e");
    auto f = []() { IOX_REPORT_IF(true, MyCodeA::OutOfBounds, RUNTIME_ERROR); };

    runInTestThread(f);

    IOX_TESTING_EXPECT_ERROR(MyCodeA::OutOfBounds);
}

TEST_F(ErrorReportingMacroApi_test, reportConditionalFatalError)
{
    ::testing::Test::RecordProperty("TEST_ID", "c69e3a0d-4c0b-4f4e-bb25-66485bc551b9");
    auto f = []() { IOX_REPORT_FATAL_IF(true, MyCodeA::OutOfMemory); };

    runInTestThread(f);

    IOX_TESTING_EXPECT_PANIC();
    IOX_TESTING_EXPECT_ERROR(MyCodeA::OutOfMemory);
}

TEST_F(ErrorReportingMacroApi_test, reportConditionalNoError)
{
    ::testing::Test::RecordProperty("TEST_ID", "9d9d6464-4586-4382-8d5f-38f3795af791");
    auto f = []() { IOX_REPORT_IF(false, MyCodeA::Unknown, RUNTIME_ERROR); };

    runInTestThread(f);

    IOX_TESTING_EXPECT_OK();
}

TEST_F(ErrorReportingMacroApi_test, checkEnforceConditionSatisfied)
{
    ::testing::Test::RecordProperty("TEST_ID", "3c684878-20f8-426f-bb8b-7576b567d04f");
    auto f = []() { IOX_ENFORCE(true, ""); };

    runInTestThread(f);

    IOX_TESTING_EXPECT_OK();
}

TEST_F(ErrorReportingMacroApi_test, checkEnforceConditionViolate)
{
    ::testing::Test::RecordProperty("TEST_ID", "fb62d315-8854-401b-82af-6161ae45a34e");
    auto f = []() { IOX_ENFORCE(false, ""); };

    runInTestThread(f);

    IOX_TESTING_EXPECT_PANIC();
    IOX_TESTING_EXPECT_ENFORCE_VIOLATION();
}

TEST_F(ErrorReportingMacroApi_test, checkAssertConditionSatisfied)
{
    ::testing::Test::RecordProperty("TEST_ID", "a76ce780-3387-4ae8-8e4c-c96bdb8aa753");
    auto f = [](int x) { IOX_ASSERT(x > 0, ""); };

    runInTestThread([&]() { f(1); });

    IOX_TESTING_EXPECT_OK();
}

TEST_F(ErrorReportingMacroApi_test, checkAssertConditionNotSatisfied)
{
    ::testing::Test::RecordProperty("TEST_ID", "9ee71bd3-9004-4950-8441-25e98cf8409c");
    auto f = [](int x) { IOX_ASSERT(x > 0, ""); };

    runInTestThread([&]() { f(0); });

    IOX_TESTING_EXPECT_PANIC();
    IOX_TESTING_EXPECT_ASSERT_VIOLATION();
}

TEST_F(ErrorReportingMacroApi_test, checkEnforceConditionNotSatisfiedWithMessage)
{
    ::testing::Test::RecordProperty("TEST_ID", "18d5b9a6-2d60-478e-8c50-d044a3672290");

    auto f = [](int x) { IOX_ENFORCE(x > 0, "some message"); };

    runInTestThread([&]() { f(0); });

    IOX_TESTING_EXPECT_PANIC();
    IOX_TESTING_EXPECT_ENFORCE_VIOLATION();
}

TEST_F(ErrorReportingMacroApi_test, checkAssertNotSatisfiedWithMessage)
{
    ::testing::Test::RecordProperty("TEST_ID", "b416674a-5861-4ab7-947b-0bd0af2f627b");
    auto f = [](int x) { IOX_ASSERT(x > 0, "some message"); };

    runInTestThread([&]() { f(0); });

    IOX_TESTING_EXPECT_PANIC();
    IOX_TESTING_EXPECT_ASSERT_VIOLATION();
}

TEST_F(ErrorReportingMacroApi_test, reportErrorsFromDifferentModules)
{
    ::testing::Test::RecordProperty("TEST_ID", "5bc53c41-4e4b-466e-b706-603ed5a3d0cf");
    auto f = []() {
        IOX_REPORT(MyCodeA::OutOfBounds, RUNTIME_ERROR);
        IOX_REPORT(MyCodeB::OutOfMemory, RUNTIME_ERROR);
    };

    runInTestThread(f);

    IOX_TESTING_EXPECT_NO_PANIC();
    IOX_TESTING_EXPECT_ERROR(MyCodeA::OutOfBounds);
    IOX_TESTING_EXPECT_ERROR(MyCodeB::OutOfMemory);
}

TEST_F(ErrorReportingMacroApi_test, distinguishErrorsFromDifferentModules)
{
    ::testing::Test::RecordProperty("TEST_ID", "f9547051-2ff7-477b-8144-e58995ff8366");
    auto f = []() { IOX_REPORT(MyCodeA::OutOfBounds, RUNTIME_ERROR); };

    runInTestThread(f);

    // these two are equivalent
    IOX_TESTING_EXPECT_ERROR(MyCodeA::OutOfBounds);
    EXPECT_TRUE(hasError(MyCodeA::OutOfBounds));

    // note that the below fails due to different enums (the errors are not the same)
    EXPECT_FALSE(hasError(MyCodeB::OutOfBounds));
}

TEST_F(ErrorReportingMacroApi_test, reportErrorsAndViolations)
{
    ::testing::Test::RecordProperty("TEST_ID", "b70331d9-f8ce-4be9-94f1-6d9505bad1d5");
    auto f = []() {
        IOX_REPORT(MyCodeA::OutOfBounds, RUNTIME_ERROR);
        IOX_REPORT(MyCodeB::OutOfMemory, RUNTIME_ERROR);
        IOX_ENFORCE(false, "");
    };

    runInTestThread(f);

    IOX_TESTING_EXPECT_PANIC();
    IOX_TESTING_EXPECT_VIOLATION();
    IOX_TESTING_EXPECT_ERROR(MyCodeA::OutOfBounds);
    IOX_TESTING_EXPECT_ERROR(MyCodeB::OutOfMemory);
}

TEST_F(ErrorReportingMacroApi_test, panicAtUnreachableCode)
{
    ::testing::Test::RecordProperty("TEST_ID", "54e84082-42eb-4fd3-af30-2647f9616719");
    auto f = []() { IOX_UNREACHABLE(); };

    runInTestThread(f);

    IOX_TESTING_EXPECT_PANIC();
}

} // namespace
