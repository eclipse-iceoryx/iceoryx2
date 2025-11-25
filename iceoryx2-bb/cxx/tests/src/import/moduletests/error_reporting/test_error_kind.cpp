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

#include "iox/error_reporting/custom/error_kind.hpp"
#include "iox/error_reporting/error_kind.hpp"

namespace
{

using namespace ::testing;
using namespace iox::er;

// check the type traits of the error kinds

// tautologies are always true ...
TEST(ErrorKind_test, fatalErrorsAreFatal)
{
    ::testing::Test::RecordProperty("TEST_ID", "2524192f-a29c-45bc-b950-ae29deb8b3ae");

    FatalKind sut;
    EXPECT_TRUE(IsFatal<FatalKind>::value);
    EXPECT_TRUE(isFatal(sut));
}

TEST(ErrorKind_test, enforceViolationsAreFatal)
{
    ::testing::Test::RecordProperty("TEST_ID", "21b79757-e46b-44fe-854a-7579b7f2243b");

    EnforceViolationKind sut;
    EXPECT_TRUE(IsFatal<EnforceViolationKind>::value);
    EXPECT_TRUE(isFatal(sut));
    EXPECT_TRUE(isFatal(ENFORCE_VIOLATION));
}

TEST(ErrorKind_test, assertViolationsAreFatal)
{
    ::testing::Test::RecordProperty("TEST_ID", "b502e70e-157d-45a0-9654-61ada213531d");

    AssertViolationKind sut;
    EXPECT_TRUE(IsFatal<AssertViolationKind>::value);
    EXPECT_TRUE(isFatal(sut));
    EXPECT_TRUE(isFatal(ASSERT_VIOLATION));
}

TEST(ErrorKind_test, runtimeErrorsAreNotFatal)
{
    ::testing::Test::RecordProperty("TEST_ID", "22c69c24-5082-4e81-8b3f-306e624731a5");

    RuntimeErrorKind sut;
    EXPECT_FALSE(IsFatal<RuntimeErrorKind>::value);
    EXPECT_FALSE(isFatal(sut));
}

} // namespace
