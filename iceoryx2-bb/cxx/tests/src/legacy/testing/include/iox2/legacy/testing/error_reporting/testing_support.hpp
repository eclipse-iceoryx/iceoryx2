// Copyright (c) 2023 by Apex.AI Inc. All rights reserved.
// Copyright (c) 2025 Contributors to the Eclipse Foundation
//
// See the NOTICE file(s) distributed with this work for additional
// information regarding copyright ownership.
//
// This program and the accompanying materials are made available under the
// terms of the Apache Software License 2.0 which is available at
// https://www.apache.org/licenses/LICENSE-2.0, or the MIT license
// which is available at https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

#ifndef IOX2_BB_TESTING_ERROR_REPORTING_TESTING_SUPPORT_HPP
#define IOX2_BB_TESTING_ERROR_REPORTING_TESTING_SUPPORT_HPP

#include <gtest/gtest.h>

#include "iox2/bb/static_function.hpp"
#include "iox2/legacy/testing/error_reporting/testing_error_handler.hpp"

#include <thread>
#include <utility>

// NOLINTNEXTLINE(hicpp-deprecated-headers) required to work on some platforms
#include <setjmp.h>

namespace iox2 {
namespace legacy {
namespace testing {

/// @brief indicates whether the test error handler registered a specific error
template <typename Code>
inline bool hasError(Code&& code) {
    auto e = iox2::legacy::er::toError(std::forward<Code>(code));
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
void runInTestThread(const bb::StaticFunction<void()> testFunction);

} // namespace testing
} // namespace legacy
} // namespace iox2

// Use macros to preserve line numbers in tests (failure case).

// ASSERT_* aborts test if the check fails.

// NOLINTBEGIN(cppcoreguidelines-macro-usage) macro required for source location in tests

#define IOX2_TESTING_ASSERT_OK() ASSERT_TRUE(iox2::legacy::testing::isInNormalState())

#define IOX2_TESTING_ASSERT_NO_PANIC() ASSERT_FALSE(iox2::legacy::testing::hasPanicked())

#define IOX2_TESTING_ASSERT_PANIC() ASSERT_TRUE(iox2::legacy::testing::hasPanicked())

#define IOX2_TESTING_ASSERT_ERROR(code) ASSERT_TRUE(iox2::legacy::testing::hasError(code))

#define IOX2_TESTING_ASSERT_NO_ERROR() ASSERT_FALSE(iox2::legacy::testing::hasError())

#define IOX2_TESTING_ASSERT_VIOLATION() ASSERT_TRUE(iox2::legacy::testing::hasViolation())

#define IOX2_TESTING_ASSERT_NO_VIOLATION() ASSERT_FALSE(iox2::legacy::testing::hasViolation())

#define IOX2_TESTING_ASSERT_ASSERT_VIOLATION() ASSERT_TRUE(iox2::legacy::testing::hasAssertViolation())

#define IOX2_TESTING_ASSERT_ENFORCE_VIOLATION() ASSERT_TRUE(iox2::legacy::testing::hasEnforceViolation())

// EXPECT_* continues with test if the check fails.

#define IOX2_TESTING_EXPECT_OK() EXPECT_TRUE(iox2::legacy::testing::isInNormalState())

#define IOX2_TESTING_EXPECT_NO_PANIC() EXPECT_FALSE(iox2::legacy::testing::hasPanicked())

#define IOX2_TESTING_EXPECT_PANIC() EXPECT_TRUE(iox2::legacy::testing::hasPanicked())

#define IOX2_TESTING_EXPECT_ERROR(code) EXPECT_TRUE(iox2::legacy::testing::hasError(code))

#define IOX2_TESTING_EXPECT_NO_ERROR() EXPECT_FALSE(iox2::legacy::testing::hasError())

#define IOX2_TESTING_EXPECT_VIOLATION() EXPECT_TRUE(iox2::legacy::testing::hasViolation())

#define IOX2_TESTING_EXPECT_NO_VIOLATION() EXPECT_FALSE(iox2::legacy::testing::hasViolation())

#define IOX2_TESTING_EXPECT_ASSERT_VIOLATION() EXPECT_TRUE(iox2::legacy::testing::hasAssertViolation())

#define IOX2_TESTING_EXPECT_ENFORCE_VIOLATION() EXPECT_TRUE(iox2::legacy::testing::hasEnforceViolation())

// NOLINTEND(cppcoreguidelines-macro-usage)

#endif
