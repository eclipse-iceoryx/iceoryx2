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

#ifndef IOX2_BB_TESTING_FATAL_FAILURE_HPP
#define IOX2_BB_TESTING_FATAL_FAILURE_HPP

#include "iox2/bb/detail/attributes.hpp"
#include "iox2/bb/static_function.hpp"
#include "iox2/legacy/error_reporting/error_kind.hpp"
#include "iox2/legacy/logging.hpp"

#include "iox2/bb/testing/testing_support.hpp"

#include <gmock/gmock.h>
#include <gtest/gtest.h>

namespace iox2 {
namespace bb {
namespace testing {
/// @brief This function is used in cases a fatal failure is expected. The function only works in combination with the
/// iceoryx error handler. It is not recommended to use this function directly but the
/// IOX2_TESTING_EXPECT_FATAL_FAILURE macro instead
/// @code
/// TEST(MyTest, valueOnNulloptIsFatal) {
///     iox2::bb::Optional<bool> sut;
///     IOX2_TESTING_EXPECT_FATAL_FAILURE([&] { sut.value(); }, iox2::legacy::er::ENFORCE_VIOLATION));
/// }
/// @endcode
/// @tparam[in] ErrorType The error type which is expected, e.g. 'iox2::legacy::HoofsError'
/// @param[in] test_function This function will be executed as SUT and is expected to call the error handler
/// @param[in] expected_error The error value which triggered the fatal failure
/// @return true if a fatal failure occurs, false otherwise
template <typename ErrorType,
          std::enable_if_t<!std::is_same<ErrorType, legacy::er::FatalKind>::value
                               && !std::is_same<ErrorType, legacy::er::EnforceViolationKind>::value
                               && !std::is_same<ErrorType, legacy::er::AssertViolationKind>::value,
                           bool> = true>
inline auto expect_fatal_failure(const bb::StaticFunction<void()>& test_function, const ErrorType expected_error)
    -> bool {
    ErrorHandler::instance().reset();
    run_in_test_thread([&]() -> auto { test_function(); });
    IOX2_TESTING_EXPECT_PANIC();
    const auto has_panicked = testing::has_panicked();

    const auto has_expected_error = has_error(expected_error);
    if (!has_expected_error) {
        IOX2_LOG(Error, "Expected an '" << expected_error << "' error but it did not happen!");
    }

    EXPECT_TRUE(has_expected_error);
    return has_expected_error && has_panicked;
}

template <typename ErrorType, std::enable_if_t<std::is_same<ErrorType, legacy::er::FatalKind>::value, bool> = true>
inline auto expect_fatal_failure(const bb::StaticFunction<void()>& test_function,
                                 const ErrorType expected_error IOX2_MAYBE_UNUSED) -> bool {
    ErrorHandler::instance().reset();
    run_in_test_thread([&]() -> auto { test_function(); });
    IOX2_TESTING_EXPECT_PANIC();
    const auto has_panicked = testing::has_panicked();

    const auto has_expected_error = has_panicked;
    if (!has_expected_error) {
        IOX2_LOG(Error, "Expected '" << legacy::er::FatalKind::name << "' but it did not happen!");
    }

    EXPECT_TRUE(has_expected_error);
    return has_expected_error && has_panicked;
}

template <typename ErrorType,
          std::enable_if_t<std::is_same<ErrorType, legacy::er::EnforceViolationKind>::value, bool> = true>
inline auto expect_fatal_failure(const bb::StaticFunction<void()>& test_function,
                                 const ErrorType expected_error IOX2_MAYBE_UNUSED) -> bool {
    ErrorHandler::instance().reset();
    run_in_test_thread([&]() -> auto { test_function(); });
    IOX2_TESTING_EXPECT_PANIC();
    const auto has_panicked = testing::has_panicked();

    const auto has_expected_error = has_enforce_violation();
    if (!has_expected_error) {
        IOX2_LOG(Error, "Expected '" << legacy::er::EnforceViolationKind::name << "' but it did not happen!");
    }

    EXPECT_TRUE(has_expected_error);
    return has_expected_error && has_panicked;
}

template <typename ErrorType,
          std::enable_if_t<std::is_same<ErrorType, legacy::er::AssertViolationKind>::value, bool> = true>
inline auto expect_fatal_failure(const bb::StaticFunction<void()>& test_function,
                                 const ErrorType expected_error IOX2_MAYBE_UNUSED) -> bool {
    ErrorHandler::instance().reset();
    run_in_test_thread([&]() -> auto { test_function(); });
    IOX2_TESTING_EXPECT_PANIC();
    const auto has_panicked = testing::has_panicked();

    const auto has_expected_error = has_assert_violation();
    if (!has_expected_error) {
        IOX2_LOG(Error, "Expected '" << legacy::er::AssertViolationKind::name << "' but it did not happen!");
    }

    EXPECT_TRUE(has_expected_error);
    return has_expected_error && has_panicked;
}

/// @brief This function is used in cases no fatal failure is expected but could potentially occur. The function only
/// works in combination with the iceoryx error handler. It is not recommended to use this function directly but the
/// IOX2_TESTING_EXPECT_NO_FATAL_FAILURE macro instead
/// @code
/// TEST(MyTest, valueIsNotFatal) {
///     iox2::bb::Optional<bool> sut{false};
///     IOX2_TESTING_EXPECT_NO_FATAL_FAILURE([&] { sut.value(); });
/// }
/// @endcode
/// @param[in] test_function This function will be executed as SUT and is not expected to call the error handler
/// @return true if no fatal failure occurs, false otherwise
inline auto expect_no_fatal_failure(const bb::StaticFunction<void()>& test_function) -> bool {
    run_in_test_thread([&]() -> auto { test_function(); });
    return !has_panicked();
}

} // namespace testing
} // namespace bb
} // namespace iox2


// NOLINTBEGIN(cppcoreguidelines-macro-usage) this is meant to blend in with the gTest macros

#define IOX2_TESTING_EXPECT_FATAL_FAILURE(test_function, expected_error)                                               \
    iox2::bb::testing::expect_fatal_failure(test_function, expected_error)

#define IOX2_TESTING_EXPECT_NO_FATAL_FAILURE(test_function) iox2::bb::testing::expect_no_fatal_failure(test_function)

// NOLINTEND(cppcoreguidelines-macro-usage)


#endif // IOX2_BB_TESTING_FATAL_FAILURE_HPP
