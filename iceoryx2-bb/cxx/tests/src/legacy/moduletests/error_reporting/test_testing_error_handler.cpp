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

#include "iox2/legacy/error_reporting/error_kind.hpp"
#include "iox2/legacy/error_reporting/types.hpp"

#include <gtest/gtest.h>

#include "iox2/bb/detail/source_location.hpp"
#include "iox2/bb/testing/testing_error_handler.hpp"

// NOLINTNEXTLINE(hicpp-deprecated-headers) required to work on some platforms
#include <setjmp.h>
#include <thread>

namespace {
using namespace ::testing;
using namespace iox2::bb::detail;
using namespace iox2::legacy::er;
using namespace iox2::bb::testing;
using iox2::legacy::er::ErrorDescriptor;

constexpr ErrorCode CODE1 { 73 };
constexpr ErrorCode CODE2 { 37 };
constexpr ErrorCode CODE3 { 21 };
constexpr ErrorCode VIOLATION { 12 };

constexpr ModuleId MODULE { 66 };

class TestingErrorHandler_test : public Test {
  public:
    void SetUp() override {
    }

    void TearDown() override {
    }

    TestingErrorHandler sut;

    bool has_panicked() const {
        return sut.has_panicked();
    }

    bool has_error() const {
        return sut.has_error();
    }

    bool has_error(ErrorCode code) const {
        return sut.has_error(code);
    }

    bool has_violation() const {
        return sut.has_violation(ErrorCode(VIOLATION));
    }

    bool hasAnyError() const {
        return has_panicked() || has_error() || has_violation();
    }
};

TEST_F(TestingErrorHandler_test, constructionAndDestructionWorks) {
    ::testing::Test::RecordProperty("TEST_ID", "09f24453-aea1-4128-83f3-929337b9892a");
    EXPECT_FALSE(hasAnyError());
}

TEST_F(TestingErrorHandler_test, panicWorks) {
    ::testing::Test::RecordProperty("TEST_ID", "e2c5e639-722f-4bab-85c7-98268345b033");
    sut.on_panic();
    EXPECT_TRUE(sut.has_panicked());
    EXPECT_FALSE(sut.has_error());

    sut.reset();
    EXPECT_FALSE(hasAnyError());
}

TEST_F(TestingErrorHandler_test, reportErrorWorks) {
    ::testing::Test::RecordProperty("TEST_ID", "90bd13cf-ece2-4221-8cce-7b2a99568a6a");
    sut.on_report_error(ErrorDescriptor { SourceLocation::current(), CODE1, MODULE });
    EXPECT_FALSE(sut.has_panicked());
    EXPECT_TRUE(sut.has_error());
    EXPECT_TRUE(sut.has_error(CODE1, MODULE));

    sut.reset();
    EXPECT_FALSE(hasAnyError());
    EXPECT_FALSE(has_error(CODE1)); // checked for consistency
}

TEST_F(TestingErrorHandler_test, reportViolationWorks) {
    ::testing::Test::RecordProperty("TEST_ID", "5746886e-7309-4435-9e0a-2e6856a318f5");
    sut.on_report_violation(ErrorDescriptor { SourceLocation::current(), VIOLATION, MODULE });

    EXPECT_TRUE(has_violation());

    sut.reset();
    EXPECT_FALSE(hasAnyError());
}

TEST_F(TestingErrorHandler_test, hasErrorDetectsOnlyreportErroredErrors) {
    ::testing::Test::RecordProperty("TEST_ID", "0ee52915-88b7-4041-9f63-93ec5c882e95");
    sut.on_report_error(ErrorDescriptor { SourceLocation::current(), CODE1, MODULE });
    sut.on_report_error(ErrorDescriptor { SourceLocation::current(), CODE2, MODULE });

    EXPECT_FALSE(sut.has_panicked());
    EXPECT_TRUE(sut.has_error(CODE1, MODULE));
    EXPECT_TRUE(sut.has_error(CODE2, MODULE));
    EXPECT_FALSE(sut.has_error(CODE3, MODULE));

    sut.reset();
    EXPECT_FALSE(sut.has_error(CODE1, MODULE));
    EXPECT_FALSE(sut.has_error(CODE2, MODULE));
    EXPECT_FALSE(sut.has_error(CODE3, MODULE));
}

TEST_F(TestingErrorHandler_test, resettingMultipleErrorsWorks) {
    ::testing::Test::RecordProperty("TEST_ID", "9715c394-5576-4fd8-a0f6-24560f60c161");
    sut.on_report_error(ErrorDescriptor { SourceLocation::current(), CODE1, MODULE });
    sut.on_report_error(ErrorDescriptor { SourceLocation::current(), CODE2, MODULE });
    sut.on_report_violation(ErrorDescriptor { SourceLocation::current(), VIOLATION, MODULE });

    sut.on_panic();

    sut.reset();
    EXPECT_FALSE(hasAnyError());
}

TEST_F(TestingErrorHandler_test, fatal_failure_test_contextWorksAndDoesNotPanic) {
    ::testing::Test::RecordProperty("TEST_ID", "df6356a6-9e9e-4ee3-8a7c-7eb68cfe2516");
    EXPECT_TRUE(sut.fatal_failure_test_context([] { }));
    EXPECT_FALSE(sut.has_panicked());
}

TEST_F(TestingErrorHandler_test, fatal_failure_test_contextCanOnlyBeCalledOnce) {
    ::testing::Test::RecordProperty("TEST_ID", "45ad9ab9-0f79-4b7c-8e36-76da3067c0fd");
    EXPECT_TRUE(sut.fatal_failure_test_context([] { }));
    EXPECT_FALSE(sut.fatal_failure_test_context([] { }));
}

TEST_F(TestingErrorHandler_test, fatal_failure_test_contextWorksAfterReset) {
    ::testing::Test::RecordProperty("TEST_ID", "1ff7942e-dd6a-4774-a162-0ec7050e4df1");
    EXPECT_TRUE(sut.fatal_failure_test_context([] { }));
    sut.reset();
    EXPECT_TRUE(sut.fatal_failure_test_context([] { }));
}

TEST_F(TestingErrorHandler_test, panicTriggersJump) {
    ::testing::Test::RecordProperty("TEST_ID", "2d99e382-ed43-4357-86f2-ef8d70c6acd8");

    std::thread t([&] {
        // regular control flow panics
        sut.fatal_failure_test_context([&] {
            sut.on_panic();
            GTEST_FAIL() << "EXPECTED longjmp but control flow continued!";
        });
    });

    if (!t.joinable()) {
        GTEST_FAIL() << "Thread should be joinable after longjmp but is not!";
    }

    t.join();

    EXPECT_TRUE(sut.has_panicked());
}

} // namespace
