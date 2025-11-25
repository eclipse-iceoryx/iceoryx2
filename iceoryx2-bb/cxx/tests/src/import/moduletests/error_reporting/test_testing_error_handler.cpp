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

#include "iox/error_reporting/error_kind.hpp"
#include "iox/error_reporting/types.hpp"
#include "test.hpp"
#include <gtest/gtest.h>

#include "iceoryx_hoofs/testing/error_reporting/testing_error_handler.hpp"
#include "iox/error_reporting/source_location.hpp"

// NOLINTNEXTLINE(hicpp-deprecated-headers) required to work on some platforms
#include <setjmp.h>
#include <thread>

namespace
{
using namespace ::testing;
using namespace iox::er;
using namespace iox::testing;
using iox::er::ErrorDescriptor;

constexpr ErrorCode CODE1{73};
constexpr ErrorCode CODE2{37};
constexpr ErrorCode CODE3{21};
constexpr ErrorCode VIOLATION{12};

constexpr ModuleId MODULE{66};

class TestingErrorHandler_test : public Test
{
  public:
    void SetUp() override
    {
    }

    void TearDown() override
    {
    }

    TestingErrorHandler sut;

    bool hasPanicked() const
    {
        return sut.hasPanicked();
    }

    bool hasError() const
    {
        return sut.hasError();
    }

    bool hasError(ErrorCode code) const
    {
        return sut.hasError(code);
    }

    bool hasViolation() const
    {
        return sut.hasViolation(ErrorCode(VIOLATION));
    }

    bool hasAnyError() const
    {
        return hasPanicked() || hasError() || hasViolation();
    }
};

TEST_F(TestingErrorHandler_test, constructionAndDestructionWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "09f24453-aea1-4128-83f3-929337b9892a");
    EXPECT_FALSE(hasAnyError());
}

TEST_F(TestingErrorHandler_test, panicWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "e2c5e639-722f-4bab-85c7-98268345b033");
    sut.onPanic();
    EXPECT_TRUE(sut.hasPanicked());
    EXPECT_FALSE(sut.hasError());

    sut.reset();
    EXPECT_FALSE(hasAnyError());
}

TEST_F(TestingErrorHandler_test, reportErrorWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "90bd13cf-ece2-4221-8cce-7b2a99568a6a");
    sut.onReportError(ErrorDescriptor{IOX_CURRENT_SOURCE_LOCATION, CODE1, MODULE});
    EXPECT_FALSE(sut.hasPanicked());
    EXPECT_TRUE(sut.hasError());
    EXPECT_TRUE(sut.hasError(CODE1, MODULE));

    sut.reset();
    EXPECT_FALSE(hasAnyError());
    EXPECT_FALSE(hasError(CODE1)); // checked for consistency
}

TEST_F(TestingErrorHandler_test, reportViolationWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "5746886e-7309-4435-9e0a-2e6856a318f5");
    sut.onReportViolation(ErrorDescriptor{IOX_CURRENT_SOURCE_LOCATION, VIOLATION, MODULE});

    EXPECT_TRUE(hasViolation());

    sut.reset();
    EXPECT_FALSE(hasAnyError());
}

TEST_F(TestingErrorHandler_test, hasErrorDetectsOnlyreportErroredErrors)
{
    ::testing::Test::RecordProperty("TEST_ID", "0ee52915-88b7-4041-9f63-93ec5c882e95");
    sut.onReportError(ErrorDescriptor{IOX_CURRENT_SOURCE_LOCATION, CODE1, MODULE});
    sut.onReportError(ErrorDescriptor{IOX_CURRENT_SOURCE_LOCATION, CODE2, MODULE});

    EXPECT_FALSE(sut.hasPanicked());
    EXPECT_TRUE(sut.hasError(CODE1, MODULE));
    EXPECT_TRUE(sut.hasError(CODE2, MODULE));
    EXPECT_FALSE(sut.hasError(CODE3, MODULE));

    sut.reset();
    EXPECT_FALSE(sut.hasError(CODE1, MODULE));
    EXPECT_FALSE(sut.hasError(CODE2, MODULE));
    EXPECT_FALSE(sut.hasError(CODE3, MODULE));
}

TEST_F(TestingErrorHandler_test, resettingMultipleErrorsWorks)
{
    ::testing::Test::RecordProperty("TEST_ID", "9715c394-5576-4fd8-a0f6-24560f60c161");
    sut.onReportError(ErrorDescriptor{IOX_CURRENT_SOURCE_LOCATION, CODE1, MODULE});
    sut.onReportError(ErrorDescriptor{IOX_CURRENT_SOURCE_LOCATION, CODE2, MODULE});
    sut.onReportViolation(ErrorDescriptor{IOX_CURRENT_SOURCE_LOCATION, VIOLATION, MODULE});

    sut.onPanic();

    sut.reset();
    EXPECT_FALSE(hasAnyError());
}

TEST_F(TestingErrorHandler_test, fatalFailureTestContextWorksAndDoesNotPanic)
{
    ::testing::Test::RecordProperty("TEST_ID", "df6356a6-9e9e-4ee3-8a7c-7eb68cfe2516");
    EXPECT_TRUE(sut.fatalFailureTestContext([] {}));
    EXPECT_FALSE(sut.hasPanicked());
}

TEST_F(TestingErrorHandler_test, fatalFailureTestContextCanOnlyBeCalledOnce)
{
    ::testing::Test::RecordProperty("TEST_ID", "45ad9ab9-0f79-4b7c-8e36-76da3067c0fd");
    EXPECT_TRUE(sut.fatalFailureTestContext([] {}));
    EXPECT_FALSE(sut.fatalFailureTestContext([] {}));
}

TEST_F(TestingErrorHandler_test, fatalFailureTestContextWorksAfterReset)
{
    ::testing::Test::RecordProperty("TEST_ID", "1ff7942e-dd6a-4774-a162-0ec7050e4df1");
    EXPECT_TRUE(sut.fatalFailureTestContext([] {}));
    sut.reset();
    EXPECT_TRUE(sut.fatalFailureTestContext([] {}));
}

TEST_F(TestingErrorHandler_test, panicTriggersJump)
{
    ::testing::Test::RecordProperty("TEST_ID", "2d99e382-ed43-4357-86f2-ef8d70c6acd8");
    std::thread t([&] {
        // regular control flow panics
        sut.fatalFailureTestContext([&] {
            sut.onPanic();
            GTEST_FAIL() << "EXPECTED longjmp but control flow continued!";
        });
    });

    if (!t.joinable())
    {
        GTEST_FAIL() << "Thread should be joinable after longjmp but is not!";
    }

    t.join();

    EXPECT_TRUE(sut.hasPanicked());
}

} // namespace
