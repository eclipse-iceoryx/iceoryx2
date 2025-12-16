// Copyright (c) 2023 by Apex.AI Inc. All rights reserved.
// Copyright (c) 2024 by ekxide IO GmbH. All rights reserved.
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

#ifndef IOX2_BB_TESTING_FATAL_FAILURE_INL
#define IOX2_BB_TESTING_FATAL_FAILURE_INL

#include "iox2/legacy/testing/fatal_failure.hpp"

namespace iox2 {
namespace legacy {
namespace testing {
template <typename ErrorType, std::enable_if_t<std::is_same<ErrorType, iox2::legacy::er::FatalKind>::value, bool>>
// NOLINTJUSTIFICATION The complexity comes from the expanded macros; without the expansions the function is quite readable
// NOLINTNEXTLINE(readability-function-size, readability-function-cognitive-complexity)
inline bool IOX2_EXPECT_FATAL_FAILURE(const bb::StaticFunction<void()> testFunction,
                                      const ErrorType expectedError IOX2_MAYBE_UNUSED) {
    iox2::legacy::testing::ErrorHandler::instance().reset();
    runInTestThread([&] { testFunction(); });
    IOX2_TESTING_EXPECT_PANIC();
    auto hasPanicked = iox2::legacy::testing::hasPanicked();

    auto hasExpectedError { false };
    hasExpectedError = hasPanicked;
    if (!hasExpectedError) {
        IOX2_LOG(Error, "Expected '" << iox2::legacy::er::FatalKind::name << "' but it did not happen!");
    }

    EXPECT_TRUE(hasExpectedError);
    return hasExpectedError && hasPanicked;
}

template <typename ErrorType,
          std::enable_if_t<std::is_same<ErrorType, iox2::legacy::er::EnforceViolationKind>::value, bool>>
// NOLINTJUSTIFICATION The complexity comes from the expanded macros; without the expansions the function is quite readable
// NOLINTNEXTLINE(readability-function-size, readability-function-cognitive-complexity)
inline bool IOX2_EXPECT_FATAL_FAILURE(const bb::StaticFunction<void()> testFunction,
                                      const ErrorType expectedError IOX2_MAYBE_UNUSED) {
    iox2::legacy::testing::ErrorHandler::instance().reset();
    runInTestThread([&] { testFunction(); });
    IOX2_TESTING_EXPECT_PANIC();
    auto hasPanicked = iox2::legacy::testing::hasPanicked();

    auto hasExpectedError { false };
    hasExpectedError = iox2::legacy::testing::hasEnforceViolation();
    if (!hasExpectedError) {
        IOX2_LOG(Error, "Expected '" << iox2::legacy::er::EnforceViolationKind::name << "' but it did not happen!");
    }

    EXPECT_TRUE(hasExpectedError);
    return hasExpectedError && hasPanicked;
}

template <typename ErrorType,
          std::enable_if_t<std::is_same<ErrorType, iox2::legacy::er::AssertViolationKind>::value, bool>>
// NOLINTJUSTIFICATION The complexity comes from the expanded macros; without the expansions the function is quite readable
// NOLINTNEXTLINE(readability-function-size, readability-function-cognitive-complexity)
inline bool IOX2_EXPECT_FATAL_FAILURE(const bb::StaticFunction<void()> testFunction,
                                      const ErrorType expectedError IOX2_MAYBE_UNUSED) {
    iox2::legacy::testing::ErrorHandler::instance().reset();
    runInTestThread([&] { testFunction(); });
    IOX2_TESTING_EXPECT_PANIC();
    auto hasPanicked = iox2::legacy::testing::hasPanicked();

    auto hasExpectedError { false };
    hasExpectedError = iox2::legacy::testing::hasAssertViolation();
    if (!hasExpectedError) {
        IOX2_LOG(Error, "Expected '" << iox2::legacy::er::AssertViolationKind::name << "' but it did not happen!");
    }

    EXPECT_TRUE(hasExpectedError);
    return hasExpectedError && hasPanicked;
}

template <typename ErrorType,
          std::enable_if_t<!std::is_same<ErrorType, iox2::legacy::er::FatalKind>::value
                               && !std::is_same<ErrorType, iox2::legacy::er::EnforceViolationKind>::value
                               && !std::is_same<ErrorType, iox2::legacy::er::AssertViolationKind>::value,
                           bool>>
// NOLINTJUSTIFICATION The complexity comes from the expanded macros; without the expansions the function is quite readable
// NOLINTNEXTLINE(readability-function-size, readability-function-cognitive-complexity)
inline bool IOX2_EXPECT_FATAL_FAILURE(const bb::StaticFunction<void()> testFunction, const ErrorType expectedError) {
    iox2::legacy::testing::ErrorHandler::instance().reset();
    runInTestThread([&] { testFunction(); });
    IOX2_TESTING_EXPECT_PANIC();
    auto hasPanicked = iox2::legacy::testing::hasPanicked();

    auto hasExpectedError { false };
    hasExpectedError = iox2::legacy::testing::hasError(expectedError);
    if (!hasExpectedError) {
        IOX2_LOG(Error, "Expected an '" << expectedError << "' error but it did not happen!");
    }

    EXPECT_TRUE(hasExpectedError);
    return hasExpectedError && hasPanicked;
}

inline bool IOX2_EXPECT_NO_FATAL_FAILURE(const bb::StaticFunction<void()> testFunction) {
    runInTestThread([&] { testFunction(); });
    return !iox2::legacy::testing::hasPanicked();
}

} // namespace testing
} // namespace legacy
} // namespace iox2

#endif // IOX2_BB_TESTING_FATAL_FAILURE_INL
