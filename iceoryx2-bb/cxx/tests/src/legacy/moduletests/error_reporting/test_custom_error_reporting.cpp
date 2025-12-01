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

#include "iox2/legacy/error_reporting/custom/error_kind.hpp"
#include "iox2/legacy/error_reporting/error_kind.hpp"
#include "iox2/legacy/error_reporting/source_location.hpp"
#include "iox2/legacy/error_reporting/violation.hpp"
#include "module_a/errors.hpp"

#include <gtest/gtest.h>

#include "iox2/legacy/error_reporting/custom/error_reporting.hpp"
#include "iox2/legacy/testing/error_reporting/testing_support.hpp"

#include "module_a/errors.hpp"

namespace {

using namespace ::testing;
using namespace iox2::legacy::er;

constexpr auto ERROR_CODE { module_a::errors::Code::OutOfBounds };
constexpr module_a::errors::Error ERROR_MODULE { ERROR_CODE };

// Here we test the custom API that the public API forwards to.
// To observe the side effects, this requires using the TestingErrorHandler (similar to the public API).

class ErrorReporting_test : public Test {
  public:
    void SetUp() override {
    }

    void TearDown() override {
    }
};

TEST_F(ErrorReporting_test, panicWorks) {
    ::testing::Test::RecordProperty("TEST_ID", "27f25cec-c815-4541-9f7d-fd2aa02474c1");

#if defined _WIN32
    GTEST_SKIP() << "The 'panicWorks' test is disabled on Windows";
#else
    auto f = []() { panic(); };

    iox2::legacy::testing::runInTestThread(f);

    IOX2_TESTING_EXPECT_PANIC();
#endif
}

TEST_F(ErrorReporting_test, panicWithLocationWorks) {
    ::testing::Test::RecordProperty("TEST_ID", "5aca0c31-1c1b-4004-bd41-b4b400258c12");

#if defined _WIN32
    GTEST_SKIP() << "The 'panicWithLocationWorks' test is disabled on Windows";
#else
    auto f = []() { panic(IOX2_CURRENT_SOURCE_LOCATION); };

    iox2::legacy::testing::runInTestThread(f);

    IOX2_TESTING_EXPECT_PANIC();
#endif
}

TEST_F(ErrorReporting_test, panicWithMessageWorks) {
    ::testing::Test::RecordProperty("TEST_ID", "f0e44332-ea9b-4041-88f4-8155ccf7538d");

#if defined _WIN32
    GTEST_SKIP() << "The 'panicWithMessageWorks' test is disabled on Windows";
#else
    auto f = []() { panic(IOX2_CURRENT_SOURCE_LOCATION, "message"); };

    iox2::legacy::testing::runInTestThread(f);

    IOX2_TESTING_EXPECT_PANIC();
#endif
}

TEST_F(ErrorReporting_test, reportNonFatalErrorWorks) {
    ::testing::Test::RecordProperty("TEST_ID", "1a1cec1b-5297-487a-bb95-e80af99886b6");

    auto f = []() {
        constexpr const char* STRINGIFIED_CONDITION { "" };
        report(IOX2_CURRENT_SOURCE_LOCATION, RUNTIME_ERROR, ERROR_MODULE, STRINGIFIED_CONDITION);
    };

    iox2::legacy::testing::runInTestThread(f);

    IOX2_TESTING_EXPECT_NO_PANIC();
    IOX2_TESTING_EXPECT_ERROR(ERROR_CODE);
}

TEST_F(ErrorReporting_test, reportFatalErrorWorks) {
    ::testing::Test::RecordProperty("TEST_ID", "");

    auto f = []() {
        constexpr const char* STRINGIFIED_CONDITION { "" };
        report(IOX2_CURRENT_SOURCE_LOCATION, FATAL, ERROR_MODULE, STRINGIFIED_CONDITION);
    };

    iox2::legacy::testing::runInTestThread(f);

    // panic is not required at this level as we cannot trust the custom API to enforce it
    // While we could also call panic in the custom API, there should only be one decison point
    // for it at a higher level
    IOX2_TESTING_EXPECT_ERROR(ERROR_CODE);
}

TEST_F(ErrorReporting_test, reportAssertViolatonWorks) {
    ::testing::Test::RecordProperty("TEST_ID", "feb63aa0-1921-408a-a887-abbb99522b31");

    auto f = []() {
        auto v = Violation::createAssertViolation();
        constexpr const char* STRINGIFIED_CONDITION { "" };
        report(IOX2_CURRENT_SOURCE_LOCATION, ASSERT_VIOLATION, v, STRINGIFIED_CONDITION);
    };

    iox2::legacy::testing::runInTestThread(f);

    IOX2_TESTING_EXPECT_ASSERT_VIOLATION();
}

// the message is printed but otherwise lost, so we cannot check for it
TEST_F(ErrorReporting_test, reportAssertViolatonWithMessageWorks) {
    ::testing::Test::RecordProperty("TEST_ID", "9228c696-d555-49c5-ade1-b65d16159e8c");

    auto f = []() {
        auto v = Violation::createAssertViolation();
        constexpr const char* STRINGIFIED_CONDITION { "" };
        report(IOX2_CURRENT_SOURCE_LOCATION, ASSERT_VIOLATION, v, STRINGIFIED_CONDITION, "message");
    };

    iox2::legacy::testing::runInTestThread(f);

    IOX2_TESTING_EXPECT_ASSERT_VIOLATION();
}

TEST_F(ErrorReporting_test, reportEnforceViolatonWorks) {
    ::testing::Test::RecordProperty("TEST_ID", "f866b43a-3a88-4097-adde-4704fc1a5e8f");

    auto f = []() {
        auto v = Violation::createEnforceViolation();
        constexpr const char* STRINGIFIED_CONDITION { "" };
        report(IOX2_CURRENT_SOURCE_LOCATION, ENFORCE_VIOLATION, v, STRINGIFIED_CONDITION);
    };

    iox2::legacy::testing::runInTestThread(f);

    IOX2_TESTING_EXPECT_ENFORCE_VIOLATION();
}

// the message is printed but otherwise lost, so we cannot check for it
TEST_F(ErrorReporting_test, reportEnforceViolatonWithMessageWorks) {
    ::testing::Test::RecordProperty("TEST_ID", "1cccd0f7-c944-4904-bf64-6f575ea13b85");

    auto f = []() {
        auto v = Violation::createEnforceViolation();
        constexpr const char* STRINGIFIED_CONDITION { "" };
        report(IOX2_CURRENT_SOURCE_LOCATION, ENFORCE_VIOLATION, v, STRINGIFIED_CONDITION, "message");
    };

    iox2::legacy::testing::runInTestThread(f);

    IOX2_TESTING_EXPECT_ENFORCE_VIOLATION();
}

} // namespace
