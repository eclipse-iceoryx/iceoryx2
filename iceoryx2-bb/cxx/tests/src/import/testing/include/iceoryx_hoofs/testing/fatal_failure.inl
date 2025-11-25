// Copyright (c) 2023 by Apex.AI Inc. All rights reserved.
// Copyright (c) 2024 by ekxide IO GmbH. All rights reserved.
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

#ifndef IOX_HOOFS_TESTING_FATAL_FAILURE_INL
#define IOX_HOOFS_TESTING_FATAL_FAILURE_INL

#include "iceoryx_hoofs/testing/fatal_failure.hpp"

namespace iox
{
namespace testing
{
template <typename ErrorType, std::enable_if_t<std::is_same<ErrorType, iox::er::FatalKind>::value, bool>>
// NOLINTJUSTIFICATION The complexity comes from the expanded macros; without the expansions the function is quite readable
// NOLINTNEXTLINE(readability-function-size, readability-function-cognitive-complexity)
inline bool IOX_EXPECT_FATAL_FAILURE(const function_ref<void()> testFunction,
                                     const ErrorType expectedError IOX_MAYBE_UNUSED)
{
    iox::testing::ErrorHandler::instance().reset();
    runInTestThread([&] { testFunction(); });
    IOX_TESTING_EXPECT_PANIC();
    auto hasPanicked = iox::testing::hasPanicked();

    auto hasExpectedError{false};
    hasExpectedError = hasPanicked;
    if (!hasExpectedError)
    {
        IOX_LOG(Error, "Expected '" << iox::er::FatalKind::name << "' but it did not happen!");
    }

    EXPECT_TRUE(hasExpectedError);
    return hasExpectedError && hasPanicked;
}

template <typename ErrorType, std::enable_if_t<std::is_same<ErrorType, iox::er::EnforceViolationKind>::value, bool>>
// NOLINTJUSTIFICATION The complexity comes from the expanded macros; without the expansions the function is quite readable
// NOLINTNEXTLINE(readability-function-size, readability-function-cognitive-complexity)
inline bool IOX_EXPECT_FATAL_FAILURE(const function_ref<void()> testFunction,
                                     const ErrorType expectedError IOX_MAYBE_UNUSED)
{
    iox::testing::ErrorHandler::instance().reset();
    runInTestThread([&] { testFunction(); });
    IOX_TESTING_EXPECT_PANIC();
    auto hasPanicked = iox::testing::hasPanicked();

    auto hasExpectedError{false};
    hasExpectedError = iox::testing::hasEnforceViolation();
    if (!hasExpectedError)
    {
        IOX_LOG(Error, "Expected '" << iox::er::EnforceViolationKind::name << "' but it did not happen!");
    }

    EXPECT_TRUE(hasExpectedError);
    return hasExpectedError && hasPanicked;
}

template <typename ErrorType, std::enable_if_t<std::is_same<ErrorType, iox::er::AssertViolationKind>::value, bool>>
// NOLINTJUSTIFICATION The complexity comes from the expanded macros; without the expansions the function is quite readable
// NOLINTNEXTLINE(readability-function-size, readability-function-cognitive-complexity)
inline bool IOX_EXPECT_FATAL_FAILURE(const function_ref<void()> testFunction,
                                     const ErrorType expectedError IOX_MAYBE_UNUSED)
{
    iox::testing::ErrorHandler::instance().reset();
    runInTestThread([&] { testFunction(); });
    IOX_TESTING_EXPECT_PANIC();
    auto hasPanicked = iox::testing::hasPanicked();

    auto hasExpectedError{false};
    hasExpectedError = iox::testing::hasAssertViolation();
    if (!hasExpectedError)
    {
        IOX_LOG(Error, "Expected '" << iox::er::AssertViolationKind::name << "' but it did not happen!");
    }

    EXPECT_TRUE(hasExpectedError);
    return hasExpectedError && hasPanicked;
}

template <typename ErrorType,
          std::enable_if_t<!std::is_same<ErrorType, iox::er::FatalKind>::value
                               && !std::is_same<ErrorType, iox::er::EnforceViolationKind>::value
                               && !std::is_same<ErrorType, iox::er::AssertViolationKind>::value,
                           bool>>
// NOLINTJUSTIFICATION The complexity comes from the expanded macros; without the expansions the function is quite readable
// NOLINTNEXTLINE(readability-function-size, readability-function-cognitive-complexity)
inline bool IOX_EXPECT_FATAL_FAILURE(const function_ref<void()> testFunction, const ErrorType expectedError)
{
    iox::testing::ErrorHandler::instance().reset();
    runInTestThread([&] { testFunction(); });
    IOX_TESTING_EXPECT_PANIC();
    auto hasPanicked = iox::testing::hasPanicked();

    auto hasExpectedError{false};
    hasExpectedError = iox::testing::hasError(expectedError);
    if (!hasExpectedError)
    {
        IOX_LOG(Error, "Expected an '" << expectedError << "' error but it did not happen!");
    }

    EXPECT_TRUE(hasExpectedError);
    return hasExpectedError && hasPanicked;
}

inline bool IOX_EXPECT_NO_FATAL_FAILURE(const function_ref<void()> testFunction)
{
    runInTestThread([&] { testFunction(); });
    return !iox::testing::hasPanicked();
}

} // namespace testing
} // namespace iox

#endif // IOX_HOOFS_TESTING_FATAL_FAILURE_INL
