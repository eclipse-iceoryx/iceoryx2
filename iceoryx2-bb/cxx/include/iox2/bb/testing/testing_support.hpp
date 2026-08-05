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
#include "iox2/bb/testing/testing_error_handler.hpp"

#include <thread>
#include <utility>

namespace iox2 {
namespace bb {
namespace testing {

/// @brief indicates whether the test error handler registered a specific error
template <typename Code>
inline auto has_error(Code&& code) -> bool {
    auto err = legacy::er::toError(std::forward<Code>(code));
    return ErrorHandler::instance().has_error(err.code(), err.module());
}

/// @brief indicates whether the test error handler invoked panic
inline auto has_panicked() -> bool {
    return ErrorHandler::instance().has_panicked();
}

/// @brief indicates whether the test error handler registered any error
inline auto has_error() -> bool {
    return ErrorHandler::instance().has_error();
}

/// @brief indicates whether the test error handler registered an enforce violation
inline auto has_enforce_violation() -> bool {
    auto code = legacy::er::Violation(legacy::er::ViolationErrorCode::ENFORCE_VIOLATION).code();
    return ErrorHandler::instance().has_violation(code);
}

/// @brief indicates whether the test error handler registered an assert violation
inline auto has_assert_violation() -> bool {
    auto code = legacy::er::Violation(legacy::er::ViolationErrorCode::ASSERT_VIOLATION).code();
    return ErrorHandler::instance().has_violation(code);
}

/// @brief indicates whether the test error handler registered  violation (there are only two kinds).
inline auto has_violation() -> bool {
    return has_enforce_violation() || has_assert_violation();
}

/// @brief indicates there is no error, violation or panic.
inline auto is_in_normal_state() -> bool {
    return !(has_panicked() || has_error() || has_violation());
}

/// @brief runs test_function in a test context that can detect fatal failures;
/// runs in a separate thread
/// @note uses a longjump inside the thread it runs the function in
inline auto run_in_test_thread(const bb::StaticFunction<void()> test_function) -> void {
    auto thread = std::thread([&]() -> auto {
        auto successfull_run = ErrorHandler::instance().fatal_failure_test_context(test_function);
        if (!successfull_run) {
            GTEST_FAIL() << "This should not fail! Incorrect usage!";
        }
    });

    if (thread.joinable()) {
        thread.join();
    }
}

} // namespace testing
} // namespace bb
} // namespace iox2

// Use macros to preserve line numbers in tests (failure case).

// ASSERT_* aborts test if the check fails.

// NOLINTBEGIN(cppcoreguidelines-macro-usage) macro required for source location in tests

#define IOX2_TESTING_ASSERT_OK() ASSERT_TRUE(iox2::bb::testing::is_in_normal_state())

#define IOX2_TESTING_ASSERT_NO_PANIC() ASSERT_FALSE(iox2::bb::testing::has_panicked())

#define IOX2_TESTING_ASSERT_PANIC() ASSERT_TRUE(iox2::bb::testing::has_panicked())

#define IOX2_TESTING_ASSERT_ERROR(code) ASSERT_TRUE(iox2::bb::testing::has_error(code))

#define IOX2_TESTING_ASSERT_NO_ERROR() ASSERT_FALSE(iox2::bb::testing::has_error())

#define IOX2_TESTING_ASSERT_VIOLATION() ASSERT_TRUE(iox2::bb::testing::has_violation())

#define IOX2_TESTING_ASSERT_NO_VIOLATION() ASSERT_FALSE(iox2::bb::testing::has_violation())

#define IOX2_TESTING_ASSERT_ASSERT_VIOLATION() ASSERT_TRUE(iox2::bb::testing::has_assert_violation())

#define IOX2_TESTING_ASSERT_ENFORCE_VIOLATION() ASSERT_TRUE(iox2::bb::testing::has_enforce_violation())

// EXPECT_* continues with test if the check fails.

#define IOX2_TESTING_EXPECT_OK() EXPECT_TRUE(iox2::bb::testing::is_in_normal_state())

#define IOX2_TESTING_EXPECT_NO_PANIC() EXPECT_FALSE(iox2::bb::testing::has_panicked())

#define IOX2_TESTING_EXPECT_PANIC() EXPECT_TRUE(iox2::bb::testing::has_panicked())

#define IOX2_TESTING_EXPECT_ERROR(code) EXPECT_TRUE(iox2::bb::testing::has_error(code))

#define IOX2_TESTING_EXPECT_NO_ERROR() EXPECT_FALSE(iox2::bb::testing::has_error())

#define IOX2_TESTING_EXPECT_VIOLATION() EXPECT_TRUE(iox2::bb::testing::has_violation())

#define IOX2_TESTING_EXPECT_NO_VIOLATION() EXPECT_FALSE(iox2::bb::testing::has_violation())

#define IOX2_TESTING_EXPECT_ASSERT_VIOLATION() EXPECT_TRUE(iox2::bb::testing::has_assert_violation())

#define IOX2_TESTING_EXPECT_ENFORCE_VIOLATION() EXPECT_TRUE(iox2::bb::testing::has_enforce_violation())

// NOLINTEND(cppcoreguidelines-macro-usage)

#endif
