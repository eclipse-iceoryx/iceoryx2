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

#include "iox2/legacy/testing/error_reporting/testing_support.hpp"

namespace iox2 {
namespace legacy {
namespace testing {

bool hasPanicked() {
    return ErrorHandler::instance().hasPanicked();
}

bool hasError() {
    return ErrorHandler::instance().hasError();
}

bool hasAssertViolation() {
    auto code = iox2::legacy::er::Violation(iox2::legacy::er::ViolationErrorCode::ASSERT_VIOLATION).code();
    return ErrorHandler::instance().hasViolation(code);
}

bool hasEnforceViolation() {
    auto code = iox2::legacy::er::Violation(iox2::legacy::er::ViolationErrorCode::ENFORCE_VIOLATION).code();
    return ErrorHandler::instance().hasViolation(code);
}

bool hasViolation() {
    return hasEnforceViolation() || hasAssertViolation();
}

bool isInNormalState() {
    return !(hasPanicked() || hasError() || hasViolation());
}

void runInTestThread(const bb::StaticFunction<void()> testFunction) {
    auto t = std::thread([&]() {
        auto successfullRun = ErrorHandler::instance().fatalFailureTestContext(testFunction);
        if (!successfullRun) {
            GTEST_FAIL() << "This should not fail! Incorrect usage!";
        }
    });

    if (t.joinable()) {
        t.join();
    }
}

} // namespace testing
} // namespace legacy
} // namespace iox2
