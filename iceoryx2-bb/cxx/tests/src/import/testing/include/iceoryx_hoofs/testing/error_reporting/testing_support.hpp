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

#ifndef IOX_HOOFS_TESTING_ERROR_REPORTING_TESTING_SUPPORT_HPP
#define IOX_HOOFS_TESTING_ERROR_REPORTING_TESTING_SUPPORT_HPP

#include <gtest/gtest.h>

#include "iceoryx_hoofs/testing/error_reporting/testing_error_handler.hpp"
#include "iox/function_ref.hpp"

#include <thread>
#include <utility>

// NOLINTNEXTLINE(hicpp-deprecated-headers) required to work on some platforms
#include <setjmp.h>

namespace iox
{
namespace testing
{

/// @brief indicates whether the test error handler registered a specific error
template <typename Code>
inline bool hasError(Code&& code)
{
    auto e = iox::er::toError(std::forward<Code>(code));
    return ErrorHandler::instance().hasError(e.code(), e.module());
}

/// @brief indicates whether the test error handler invoked panic
bool hasPanicked();

/// @brief indicates whether the test error handler registered any error
bool hasError();

/// @brief indicates whether the test error handler registered an enforce violation
bool hasEnforceViolation();

/// @brief indicates whether the test error handler registered an assert violation
bool hasAssertViolation();

/// @brief indicates whether the test error handler registered  violation (there are only two kinds).
bool hasViolation();

/// @brief indicates there is no error, violation or panic.
bool isInNormalState();

/// @brief runs testFunction in a testContext that can detect fatal failures;
/// runs in a separate thread
/// @note uses a longjump inside the thread it runs the function in
void runInTestThread(const function_ref<void()> testFunction);

} // namespace testing
} // namespace iox

// Use macros to preserve line numbers in tests (failure case).

// ASSERT_* aborts test if the check fails.

// NOLINTBEGIN(cppcoreguidelines-macro-usage) macro required for source location in tests

#define IOX_TESTING_ASSERT_OK() ASSERT_TRUE(iox::testing::isInNormalState())

#define IOX_TESTING_ASSERT_NO_PANIC() ASSERT_FALSE(iox::testing::hasPanicked())

#define IOX_TESTING_ASSERT_PANIC() ASSERT_TRUE(iox::testing::hasPanicked())

#define IOX_TESTING_ASSERT_ERROR(code) ASSERT_TRUE(iox::testing::hasError(code))

#define IOX_TESTING_ASSERT_NO_ERROR() ASSERT_FALSE(iox::testing::hasError())

#define IOX_TESTING_ASSERT_VIOLATION() ASSERT_TRUE(iox::testing::hasViolation())

#define IOX_TESTING_ASSERT_NO_VIOLATION() ASSERT_FALSE(iox::testing::hasViolation())

#define IOX_TESTING_ASSERT_ASSERT_VIOLATION() ASSERT_TRUE(iox::testing::hasAssertViolation())

#define IOX_TESTING_ASSERT_ENFORCE_VIOLATION() ASSERT_TRUE(iox::testing::hasEnforceViolation())

// EXPECT_* continues with test if the check fails.

#define IOX_TESTING_EXPECT_OK() EXPECT_TRUE(iox::testing::isInNormalState())

#define IOX_TESTING_EXPECT_NO_PANIC() EXPECT_FALSE(iox::testing::hasPanicked())

#define IOX_TESTING_EXPECT_PANIC() EXPECT_TRUE(iox::testing::hasPanicked())

#define IOX_TESTING_EXPECT_ERROR(code) EXPECT_TRUE(iox::testing::hasError(code))

#define IOX_TESTING_EXPECT_NO_ERROR() EXPECT_FALSE(iox::testing::hasError())

#define IOX_TESTING_EXPECT_VIOLATION() EXPECT_TRUE(iox::testing::hasViolation())

#define IOX_TESTING_EXPECT_NO_VIOLATION() EXPECT_FALSE(iox::testing::hasViolation())

#define IOX_TESTING_EXPECT_ASSERT_VIOLATION() EXPECT_TRUE(iox::testing::hasAssertViolation())

#define IOX_TESTING_EXPECT_ENFORCE_VIOLATION() EXPECT_TRUE(iox::testing::hasEnforceViolation())

// NOLINTEND(cppcoreguidelines-macro-usage)

#endif
