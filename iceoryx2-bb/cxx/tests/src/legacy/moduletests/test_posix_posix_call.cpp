// Copyright (c) 2021 by Apex.AI Inc. All rights reserved.
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

#include "iox2/legacy/posix_call.hpp"
#include "iox2/legacy/testing/testing_logger.hpp"

#include <gmock/gmock.h>
#include <gtest/gtest.h>

using namespace ::testing;

namespace {
// NOLINTJUSTIFICATION only used in testing
// NOLINTNEXTLINE(bugprone-easily-swappable-parameters)
int testFunction(int returnValue, int errnoValue) {
    errno = errnoValue;
    return returnValue;
}

int32_t eintrRepetition = 0;
int testEintr() {
    if (0 < --eintrRepetition) {
        errno = EINTR;
        return 1;
    }
    return 0;
}

int returnValueIsErrno(int returnValue) {
    errno = 0;
    return returnValue;
}

class PosixCall_test : public Test {
  public:
};
} // namespace

TEST_F(PosixCall_test, CallingFunctionWithSuccessReturnValue_GoodCase) {
    ::testing::Test::RecordProperty("TEST_ID", "a01759f9-bd81-4223-9313-91f2c12acd2c");

    constexpr int RETURN_VALUE = 1;
    constexpr int ERRNO_VALUE = 2;

    auto res = IOX2_POSIX_CALL(testFunction)(RETURN_VALUE, ERRNO_VALUE).successReturnValue(RETURN_VALUE).evaluate();
    if (res.has_value()) {
        EXPECT_THAT(res->value, Eq(RETURN_VALUE));
        EXPECT_THAT(res->errnum, Eq(ERRNO_VALUE));
    } else {
        GTEST_FAIL();
    }

    iox2::legacy::testing::TestingLogger::checkLogMessageIfLogLevelIsSupported(
        iox2::legacy::log::LogLevel::Error, [](const auto& logMessages) { ASSERT_EQ(logMessages.size(), 0); });
}

TEST_F(PosixCall_test, CallingFunctionWithSuccessReturnValue_BadCase) {
    ::testing::Test::RecordProperty("TEST_ID", "aea9317e-1bf8-47bd-bc6c-971439f93a43");

    constexpr int RETURN_VALUE = 3;
    constexpr int ERRNO_VALUE = 4;

    auto res = IOX2_POSIX_CALL(testFunction)(RETURN_VALUE, ERRNO_VALUE).successReturnValue(RETURN_VALUE + 1).evaluate();
    if (res.has_value()) {
        GTEST_FAIL();
    } else {
        EXPECT_THAT(res.error().value, Eq(RETURN_VALUE));
        EXPECT_THAT(res.error().errnum, Eq(ERRNO_VALUE));
    }

    // we expect an error message via stderr to the console, details are not
    // verified since it depends on the target and where the source code is
    // stored
    iox2::legacy::testing::TestingLogger::checkLogMessageIfLogLevelIsSupported(
        iox2::legacy::log::LogLevel::Error, [](const auto& logMessages) { ASSERT_GT(logMessages.size(), 0); });
}

TEST_F(PosixCall_test, CallingFunctionWithFailureReturnValue_GoodCase) {
    ::testing::Test::RecordProperty("TEST_ID", "ee9814a8-b646-4a8c-b1ba-5923c5331a7a");

    constexpr int RETURN_VALUE = 5;
    constexpr int ERRNO_VALUE = 6;

    auto res = IOX2_POSIX_CALL(testFunction)(RETURN_VALUE, ERRNO_VALUE).failureReturnValue(RETURN_VALUE + 1).evaluate();
    if (res.has_value()) {
        EXPECT_THAT(res->value, Eq(RETURN_VALUE));
        EXPECT_THAT(res->errnum, Eq(ERRNO_VALUE));
    } else {
        GTEST_FAIL();
    }

    iox2::legacy::testing::TestingLogger::checkLogMessageIfLogLevelIsSupported(
        iox2::legacy::log::LogLevel::Error, [](const auto& logMessages) { ASSERT_EQ(logMessages.size(), 0); });
}

TEST_F(PosixCall_test, CallingFunctionWithFailureReturnValue_BadCase) {
    ::testing::Test::RecordProperty("TEST_ID", "de9ec4b8-72ac-43c2-a3a2-962571c8039d");

    constexpr int RETURN_VALUE = 7;
    constexpr int ERRNO_VALUE = 8;

    auto res = IOX2_POSIX_CALL(testFunction)(RETURN_VALUE, ERRNO_VALUE).failureReturnValue(RETURN_VALUE).evaluate();
    if (res.has_value()) {
        GTEST_FAIL();
    } else {
        EXPECT_THAT(res.error().value, Eq(RETURN_VALUE));
        EXPECT_THAT(res.error().errnum, Eq(ERRNO_VALUE));
    }

    // we expect an error message via stderr to the console, details are not
    // verified since it depends on the target and where the source code is
    // stored
    iox2::legacy::testing::TestingLogger::checkLogMessageIfLogLevelIsSupported(
        iox2::legacy::log::LogLevel::Error, [](const auto& logMessages) { ASSERT_GT(logMessages.size(), 0); });
}

TEST_F(PosixCall_test, CallingFunctionWithSuccessReturnValueAndIgnoredErrno_GoodCase) {
    ::testing::Test::RecordProperty("TEST_ID", "4cf87f69-d694-423a-98bf-d5658758f7f0");

    constexpr int RETURN_VALUE = 9;
    constexpr int ERRNO_VALUE = 10;

    auto res = IOX2_POSIX_CALL(testFunction)(RETURN_VALUE, ERRNO_VALUE)
                   .successReturnValue(RETURN_VALUE + 1)
                   .ignoreErrnos(ERRNO_VALUE)
                   .evaluate();
    if (res.has_value()) {
        EXPECT_THAT(res->value, Eq(RETURN_VALUE));
        EXPECT_THAT(res->errnum, Eq(ERRNO_VALUE));
    } else {
        GTEST_FAIL();
    }

    iox2::legacy::testing::TestingLogger::checkLogMessageIfLogLevelIsSupported(
        iox2::legacy::log::LogLevel::Error, [](const auto& logMessages) { ASSERT_EQ(logMessages.size(), 0); });
}

TEST_F(PosixCall_test, CallingFunctionWithSuccessReturnValueAndIgnoredErrno_BadCase) {
    ::testing::Test::RecordProperty("TEST_ID", "29cd753e-9b89-43dd-a754-d9bde42d7ff3");

    constexpr int RETURN_VALUE = 11;
    constexpr int ERRNO_VALUE = 12;

    auto res = IOX2_POSIX_CALL(testFunction)(RETURN_VALUE, ERRNO_VALUE)
                   .successReturnValue(RETURN_VALUE + 1)
                   .ignoreErrnos(ERRNO_VALUE + 1)
                   .evaluate();
    if (res.has_value()) {
        GTEST_FAIL();
    } else {
        EXPECT_THAT(res.error().value, Eq(RETURN_VALUE));
        EXPECT_THAT(res.error().errnum, Eq(ERRNO_VALUE));
    }

    // we expect an error message via stderr to the console, details are not
    // verified since it depends on the target and where the source code is
    // stored
    iox2::legacy::testing::TestingLogger::checkLogMessageIfLogLevelIsSupported(
        iox2::legacy::log::LogLevel::Error, [](const auto& logMessages) { ASSERT_GT(logMessages.size(), 0); });
}

TEST_F(PosixCall_test, CallingFunctionWithFailureReturnValueAndIgnoredErrno_GoodCase) {
    ::testing::Test::RecordProperty("TEST_ID", "f9cc178d-9d74-4458-8cc7-5086b2359511");

    constexpr int RETURN_VALUE = 13;
    constexpr int ERRNO_VALUE = 14;

    auto res = IOX2_POSIX_CALL(testFunction)(RETURN_VALUE, ERRNO_VALUE)
                   .failureReturnValue(RETURN_VALUE)
                   .ignoreErrnos(ERRNO_VALUE)
                   .evaluate();
    if (res.has_value()) {
        EXPECT_THAT(res->value, Eq(RETURN_VALUE));
        EXPECT_THAT(res->errnum, Eq(ERRNO_VALUE));
    } else {
        GTEST_FAIL();
    }

    iox2::legacy::testing::TestingLogger::checkLogMessageIfLogLevelIsSupported(
        iox2::legacy::log::LogLevel::Error, [](const auto& logMessages) { ASSERT_EQ(logMessages.size(), 0); });
}

TEST_F(PosixCall_test, CallingFunctionWithFailureReturnValueAndIgnoredErrno_BadCase) {
    ::testing::Test::RecordProperty("TEST_ID", "7d872f26-b303-4f01-817d-857e5ee2353a");

    constexpr int RETURN_VALUE = 15;
    constexpr int ERRNO_VALUE = 16;

    auto res = IOX2_POSIX_CALL(testFunction)(RETURN_VALUE, ERRNO_VALUE)
                   .failureReturnValue(RETURN_VALUE)
                   .ignoreErrnos(ERRNO_VALUE + 1)
                   .evaluate();
    if (res.has_value()) {
        GTEST_FAIL();
    } else {
        EXPECT_THAT(res.error().value, Eq(RETURN_VALUE));
        EXPECT_THAT(res.error().errnum, Eq(ERRNO_VALUE));
    }

    // we expect an error message via stderr to the console, details are not
    // verified since it depends on the target and where the source code is
    // stored
    iox2::legacy::testing::TestingLogger::checkLogMessageIfLogLevelIsSupported(
        iox2::legacy::log::LogLevel::Error, [](const auto& logMessages) { ASSERT_GT(logMessages.size(), 0); });
}

TEST_F(PosixCall_test, IgnoringMultipleErrnosWorks) {
    ::testing::Test::RecordProperty("TEST_ID", "382eeb72-03a8-480f-8fe1-51e749370d44");

    constexpr int RETURN_VALUE = 17;
    constexpr int ERRNO_VALUE = 18;

    auto res = IOX2_POSIX_CALL(testFunction)(RETURN_VALUE, ERRNO_VALUE)
                   .successReturnValue(1)
                   .ignoreErrnos(ERRNO_VALUE - 10, ERRNO_VALUE, ERRNO_VALUE + 17)
                   .evaluate();
    if (res.has_value()) {
        EXPECT_THAT(res->value, Eq(RETURN_VALUE));
        EXPECT_THAT(res->errnum, Eq(ERRNO_VALUE));
    } else {
        GTEST_FAIL();
    }

    iox2::legacy::testing::TestingLogger::checkLogMessageIfLogLevelIsSupported(
        iox2::legacy::log::LogLevel::Error, [](const auto& logMessages) { ASSERT_EQ(logMessages.size(), 0); });
}

TEST_F(PosixCall_test, IgnoringMultipleErrnosWhereOccurringErrnoIsNotListedFails) {
    ::testing::Test::RecordProperty("TEST_ID", "a1f11d1b-7eb4-4a61-b977-ec0813a55fe1");

    constexpr int RETURN_VALUE = 19;
    constexpr int ERRNO_VALUE = 20;

    auto res = IOX2_POSIX_CALL(testFunction)(RETURN_VALUE, ERRNO_VALUE)
                   .successReturnValue(1)
                   .ignoreErrnos(ERRNO_VALUE - 10, ERRNO_VALUE + 17, ERRNO_VALUE + 1337, ERRNO_VALUE - 2)
                   .evaluate();
    if (res.has_value()) {
        GTEST_FAIL();
    } else {
        EXPECT_THAT(res.error().value, Eq(RETURN_VALUE));
        EXPECT_THAT(res.error().errnum, Eq(ERRNO_VALUE));
    }

    iox2::legacy::testing::TestingLogger::checkLogMessageIfLogLevelIsSupported(
        iox2::legacy::log::LogLevel::Error, [](const auto& logMessages) { ASSERT_GT(logMessages.size(), 0); });
}

TEST_F(PosixCall_test, IgnoringMultipleErrnosWhereOccurringErrnoIsFirstInListSucceeds) {
    ::testing::Test::RecordProperty("TEST_ID", "f6937559-185e-481d-9186-4c332cafd700");

    constexpr int RETURN_VALUE = 21;
    constexpr int ERRNO_VALUE = 22;

    auto res = IOX2_POSIX_CALL(testFunction)(RETURN_VALUE, ERRNO_VALUE)
                   .successReturnValue(1)
                   .ignoreErrnos(ERRNO_VALUE, ERRNO_VALUE - 91, ERRNO_VALUE + 137, ERRNO_VALUE + 17, ERRNO_VALUE - 29)
                   .evaluate();
    if (res.has_value()) {
        EXPECT_THAT(res->value, Eq(RETURN_VALUE));
        EXPECT_THAT(res->errnum, Eq(ERRNO_VALUE));
    } else {
        GTEST_FAIL();
    }

    iox2::legacy::testing::TestingLogger::checkLogMessageIfLogLevelIsSupported(
        iox2::legacy::log::LogLevel::Error, [](const auto& logMessages) { ASSERT_EQ(logMessages.size(), 0); });
}

TEST_F(PosixCall_test, IgnoringMultipleErrnosWhereOccurringErrnoIsLastInListSucceeds) {
    ::testing::Test::RecordProperty("TEST_ID", "3e9817ef-ad3a-4c73-94bc-1032448972e3");

    constexpr int RETURN_VALUE = 23;
    constexpr int ERRNO_VALUE = 24;

    auto res =
        IOX2_POSIX_CALL(testFunction)(RETURN_VALUE, ERRNO_VALUE)
            .successReturnValue(1)
            .ignoreErrnos(ERRNO_VALUE - 918, ERRNO_VALUE + 8137, ERRNO_VALUE + 187, ERRNO_VALUE - 289, ERRNO_VALUE)
            .evaluate();
    if (res.has_value()) {
        EXPECT_THAT(res->value, Eq(RETURN_VALUE));
        EXPECT_THAT(res->errnum, Eq(ERRNO_VALUE));
    } else {
        GTEST_FAIL();
    }

    iox2::legacy::testing::TestingLogger::checkLogMessageIfLogLevelIsSupported(
        iox2::legacy::log::LogLevel::Error, [](const auto& logMessages) { ASSERT_EQ(logMessages.size(), 0); });
}

TEST_F(PosixCall_test, IgnoringErrnosByMultipleIgnoreErrnosCallsWorksWhenErrnoIsFirst) {
    ::testing::Test::RecordProperty("TEST_ID", "fbf80e39-2f7c-4eff-9bd2-079526f83059");

    constexpr int RETURN_VALUE = 117;
    constexpr int ERRNO_VALUE = 118;

    auto res = IOX2_POSIX_CALL(testFunction)(RETURN_VALUE, ERRNO_VALUE)
                   .successReturnValue(1)
                   .ignoreErrnos(ERRNO_VALUE)
                   .ignoreErrnos(ERRNO_VALUE - 10)
                   .ignoreErrnos(ERRNO_VALUE + 17)
                   .evaluate();
    if (res.has_value()) {
        EXPECT_THAT(res->value, Eq(RETURN_VALUE));
        EXPECT_THAT(res->errnum, Eq(ERRNO_VALUE));
    } else {
        GTEST_FAIL();
    }

    iox2::legacy::testing::TestingLogger::checkLogMessageIfLogLevelIsSupported(
        iox2::legacy::log::LogLevel::Error, [](const auto& logMessages) { ASSERT_EQ(logMessages.size(), 0); });
}

TEST_F(PosixCall_test, IgnoringErrnosByMultipleIgnoreErrnosCallsWorksWhenErrnoIsMiddle) {
    ::testing::Test::RecordProperty("TEST_ID", "06eba974-df54-44ff-accb-e0d4e3a64895");

    constexpr int RETURN_VALUE = 217;
    constexpr int ERRNO_VALUE = 218;

    auto res = IOX2_POSIX_CALL(testFunction)(RETURN_VALUE, ERRNO_VALUE)
                   .successReturnValue(1)
                   .ignoreErrnos(ERRNO_VALUE - 10)
                   .ignoreErrnos(ERRNO_VALUE)
                   .ignoreErrnos(ERRNO_VALUE + 17)
                   .evaluate();
    if (res.has_value()) {
        EXPECT_THAT(res->value, Eq(RETURN_VALUE));
        EXPECT_THAT(res->errnum, Eq(ERRNO_VALUE));
    } else {
        GTEST_FAIL();
    }

    iox2::legacy::testing::TestingLogger::checkLogMessageIfLogLevelIsSupported(
        iox2::legacy::log::LogLevel::Error, [](const auto& logMessages) { ASSERT_EQ(logMessages.size(), 0); });
}

TEST_F(PosixCall_test, IgnoringErrnosByMultipleIgnoreErrnosCallsWorksWhenErrnoIsLast) {
    ::testing::Test::RecordProperty("TEST_ID", "5c3e55ad-5665-47be-ba01-3c7096075858");

    constexpr int RETURN_VALUE = 317;
    constexpr int ERRNO_VALUE = 318;

    auto res = IOX2_POSIX_CALL(testFunction)(RETURN_VALUE, ERRNO_VALUE)
                   .successReturnValue(1)
                   .ignoreErrnos(ERRNO_VALUE - 10)
                   .ignoreErrnos(ERRNO_VALUE + 17)
                   .ignoreErrnos(ERRNO_VALUE)
                   .evaluate();
    if (res.has_value()) {
        EXPECT_THAT(res->value, Eq(RETURN_VALUE));
        EXPECT_THAT(res->errnum, Eq(ERRNO_VALUE));
    } else {
        GTEST_FAIL();
    }

    iox2::legacy::testing::TestingLogger::checkLogMessageIfLogLevelIsSupported(
        iox2::legacy::log::LogLevel::Error, [](const auto& logMessages) { ASSERT_EQ(logMessages.size(), 0); });
}

TEST_F(PosixCall_test, IgnoringErrnosByMultipleIgnoreErrnosCallsFails) {
    ::testing::Test::RecordProperty("TEST_ID", "83f52825-fd6c-457c-859f-d3cccd8800bd");

    constexpr int RETURN_VALUE = 417;
    constexpr int ERRNO_VALUE = 418;

    auto res = IOX2_POSIX_CALL(testFunction)(RETURN_VALUE, ERRNO_VALUE)
                   .successReturnValue(1)
                   .ignoreErrnos(ERRNO_VALUE - 10)
                   .ignoreErrnos(ERRNO_VALUE + 13)
                   .ignoreErrnos(ERRNO_VALUE + 17)
                   .evaluate();
    if (res.has_value()) {
        GTEST_FAIL();
    } else {
        EXPECT_THAT(res.error().value, Eq(RETURN_VALUE));
        EXPECT_THAT(res.error().errnum, Eq(ERRNO_VALUE));
    }

    iox2::legacy::testing::TestingLogger::checkLogMessageIfLogLevelIsSupported(
        iox2::legacy::log::LogLevel::Error, [](const auto& logMessages) { ASSERT_GT(logMessages.size(), 0); });
}

TEST_F(PosixCall_test, SuppressErrnoLoggingWithNonPresentErrnoPrintsErrorMessage) {
    ::testing::Test::RecordProperty("TEST_ID", "f7447e69-44bd-45ba-9dda-4c90107fc73e");

    constexpr int RETURN_VALUE = 111;
    constexpr int ERRNO_VALUE = 112;

    auto res = IOX2_POSIX_CALL(testFunction)(RETURN_VALUE, ERRNO_VALUE)
                   .successReturnValue(1)
                   .suppressErrorMessagesForErrnos(ERRNO_VALUE - 10)
                   .evaluate();
    if (res.has_value()) {
        GTEST_FAIL();
    } else {
        EXPECT_THAT(res.error().value, Eq(RETURN_VALUE));
        EXPECT_THAT(res.error().errnum, Eq(ERRNO_VALUE));
    }

    iox2::legacy::testing::TestingLogger::checkLogMessageIfLogLevelIsSupported(
        iox2::legacy::log::LogLevel::Error, [](const auto& logMessages) { ASSERT_GT(logMessages.size(), 0); });
}

TEST_F(PosixCall_test, SuppressErrnoLoggingWithPresentErrnoDoesNotPrintErrorMessage) {
    ::testing::Test::RecordProperty("TEST_ID", "bc7bc0f5-8d31-4254-a61e-6a5c43ab87ee");

    constexpr int RETURN_VALUE = 113;
    constexpr int ERRNO_VALUE = 114;

    auto res = IOX2_POSIX_CALL(testFunction)(RETURN_VALUE, ERRNO_VALUE)
                   .successReturnValue(1)
                   .suppressErrorMessagesForErrnos(ERRNO_VALUE)
                   .evaluate();
    if (res.has_value()) {
        GTEST_FAIL();
    } else {
        EXPECT_THAT(res.error().value, Eq(RETURN_VALUE));
        EXPECT_THAT(res.error().errnum, Eq(ERRNO_VALUE));
    }

    iox2::legacy::testing::TestingLogger::checkLogMessageIfLogLevelIsSupported(
        iox2::legacy::log::LogLevel::Error, [](const auto& logMessages) { ASSERT_EQ(logMessages.size(), 0); });
}

TEST_F(PosixCall_test, SuppressMultipleErrnoLoggingWithNoPresentErrnoPrintsErrorMessage) {
    ::testing::Test::RecordProperty("TEST_ID", "207f2148-f0f1-464b-bf64-0f8a820a5b70");

    constexpr int RETURN_VALUE = 115;
    constexpr int ERRNO_VALUE = 116;

    auto res = IOX2_POSIX_CALL(testFunction)(RETURN_VALUE, ERRNO_VALUE)
                   .successReturnValue(1)
                   .suppressErrorMessagesForErrnos(ERRNO_VALUE - 10, ERRNO_VALUE + 16, ERRNO_VALUE + 17)
                   .evaluate();
    if (res.has_value()) {
        GTEST_FAIL();
    } else {
        EXPECT_THAT(res.error().value, Eq(RETURN_VALUE));
        EXPECT_THAT(res.error().errnum, Eq(ERRNO_VALUE));
    }

    iox2::legacy::testing::TestingLogger::checkLogMessageIfLogLevelIsSupported(
        iox2::legacy::log::LogLevel::Error, [](const auto& logMessages) { ASSERT_GT(logMessages.size(), 0); });
}

TEST_F(PosixCall_test, SuppressMultipleErrnoLoggingWithPresentErrnoDoesNotPrintErrorMessage) {
    ::testing::Test::RecordProperty("TEST_ID", "1f26ada9-ba40-4a6f-a572-c911dff36ebb");

    constexpr int RETURN_VALUE = 117;
    constexpr int ERRNO_VALUE = 118;

    auto res = IOX2_POSIX_CALL(testFunction)(RETURN_VALUE, ERRNO_VALUE)
                   .successReturnValue(1)
                   .suppressErrorMessagesForErrnos(ERRNO_VALUE - 10, ERRNO_VALUE, ERRNO_VALUE + 17)
                   .evaluate();
    if (res.has_value()) {
        GTEST_FAIL();
    } else {
        EXPECT_THAT(res.error().value, Eq(RETURN_VALUE));
        EXPECT_THAT(res.error().errnum, Eq(ERRNO_VALUE));
    }

    iox2::legacy::testing::TestingLogger::checkLogMessageIfLogLevelIsSupported(
        iox2::legacy::log::LogLevel::Error, [](const auto& logMessages) { ASSERT_EQ(logMessages.size(), 0); });
}

TEST_F(PosixCall_test, SuppressErrnoLoggingByMultipleCallsWithNonPresentErrnoPrintsErrorMessage) {
    ::testing::Test::RecordProperty("TEST_ID", "49e41c5c-9a95-47c8-a522-245a3885003b");

    constexpr int RETURN_VALUE = 119;
    constexpr int ERRNO_VALUE = 120;

    auto res = IOX2_POSIX_CALL(testFunction)(RETURN_VALUE, ERRNO_VALUE)
                   .successReturnValue(1)
                   .suppressErrorMessagesForErrnos(ERRNO_VALUE - 10)
                   .suppressErrorMessagesForErrnos(ERRNO_VALUE + 13)
                   .suppressErrorMessagesForErrnos(ERRNO_VALUE + 17)
                   .evaluate();
    if (res.has_value()) {
        GTEST_FAIL();
    } else {
        EXPECT_THAT(res.error().value, Eq(RETURN_VALUE));
        EXPECT_THAT(res.error().errnum, Eq(ERRNO_VALUE));
    }

    iox2::legacy::testing::TestingLogger::checkLogMessageIfLogLevelIsSupported(
        iox2::legacy::log::LogLevel::Error, [](const auto& logMessages) { ASSERT_GT(logMessages.size(), 0); });
}

TEST_F(PosixCall_test, SuppressErrnoLoggingByMultipleCallsWithPresentErrnoDoesNotPrintErrorMessage) {
    ::testing::Test::RecordProperty("TEST_ID", "0624a93c-8589-44e2-94f5-ba284486f220");

    constexpr int RETURN_VALUE = 121;
    constexpr int ERRNO_VALUE = 122;

    auto res = IOX2_POSIX_CALL(testFunction)(RETURN_VALUE, ERRNO_VALUE)
                   .successReturnValue(1)
                   .suppressErrorMessagesForErrnos(ERRNO_VALUE - 10)
                   .suppressErrorMessagesForErrnos(ERRNO_VALUE)
                   .suppressErrorMessagesForErrnos(ERRNO_VALUE + 17)
                   .evaluate();
    if (res.has_value()) {
        GTEST_FAIL();
    } else {
        EXPECT_THAT(res.error().value, Eq(RETURN_VALUE));
        EXPECT_THAT(res.error().errnum, Eq(ERRNO_VALUE));
    }

    iox2::legacy::testing::TestingLogger::checkLogMessageIfLogLevelIsSupported(
        iox2::legacy::log::LogLevel::Error, [](const auto& logMessages) { ASSERT_EQ(logMessages.size(), 0); });
}

TEST_F(PosixCall_test, SuppressErrnoLoggingOfIgnoredErrnoDoesNotPrintErrorMessage) {
    ::testing::Test::RecordProperty("TEST_ID", "3292eff4-6f91-4a84-a9ba-05e77b63a5f0");

    constexpr int RETURN_VALUE = 123;
    constexpr int ERRNO_VALUE = 124;

    auto res = IOX2_POSIX_CALL(testFunction)(RETURN_VALUE, ERRNO_VALUE)
                   .successReturnValue(1)
                   .ignoreErrnos(ERRNO_VALUE)
                   .suppressErrorMessagesForErrnos(ERRNO_VALUE)
                   .evaluate();
    if (res.has_value()) {
        EXPECT_THAT(res->value, Eq(RETURN_VALUE));
        EXPECT_THAT(res->errnum, Eq(ERRNO_VALUE));
    } else {
        GTEST_FAIL();
    }

    iox2::legacy::testing::TestingLogger::checkLogMessageIfLogLevelIsSupported(
        iox2::legacy::log::LogLevel::Error, [](const auto& logMessages) { ASSERT_EQ(logMessages.size(), 0); });
}

TEST_F(PosixCall_test, SuppressErrnoLoggingOfNotIgnoredErrnoDoesNotPrintErrorMessage) {
    ::testing::Test::RecordProperty("TEST_ID", "9d8142b9-f993-46a2-ba45-2f103d3c6ee8");

    constexpr int RETURN_VALUE = 123;
    constexpr int ERRNO_VALUE = 124;

    auto res = IOX2_POSIX_CALL(testFunction)(RETURN_VALUE, ERRNO_VALUE)
                   .successReturnValue(1)
                   .ignoreErrnos(ERRNO_VALUE + 10)
                   .suppressErrorMessagesForErrnos(ERRNO_VALUE)
                   .evaluate();
    if (res.has_value()) {
        GTEST_FAIL();
    } else {
        EXPECT_THAT(res.error().value, Eq(RETURN_VALUE));
        EXPECT_THAT(res.error().errnum, Eq(ERRNO_VALUE));
    }

    iox2::legacy::testing::TestingLogger::checkLogMessageIfLogLevelIsSupported(
        iox2::legacy::log::LogLevel::Error, [](const auto& logMessages) { ASSERT_EQ(logMessages.size(), 0); });
}

TEST_F(PosixCall_test, RecallingFunctionWithEintrWorks) {
    ::testing::Test::RecordProperty("TEST_ID", "c613542f-dead-409e-9630-f05486faa8f3");

    eintrRepetition = iox2::legacy::POSIX_CALL_EINTR_REPETITIONS;
    auto res = IOX2_POSIX_CALL(testEintr)().successReturnValue(0).evaluate();
    if (res.has_value()) {
        EXPECT_THAT(res->value, Eq(0));
        EXPECT_THAT(res->errnum, Eq(0));
    } else {
        GTEST_FAIL();
    }

    EXPECT_THAT(eintrRepetition, Eq(0));
    iox2::legacy::testing::TestingLogger::checkLogMessageIfLogLevelIsSupported(
        iox2::legacy::log::LogLevel::Error, [](const auto& logMessages) { ASSERT_EQ(logMessages.size(), 0); });
}


TEST_F(PosixCall_test, FunctionReturnsEINTRTooOftenResultsInFailure) {
    ::testing::Test::RecordProperty("TEST_ID", "a63b36d3-bccc-4d5c-9fad-502dd6f163f1");

    eintrRepetition = iox2::legacy::POSIX_CALL_EINTR_REPETITIONS + 1;
    auto res = IOX2_POSIX_CALL(testEintr)().successReturnValue(0).evaluate();
    if (res.has_value()) {
        GTEST_FAIL();
    } else {
        EXPECT_THAT(res.error().value, Eq(1));
        EXPECT_THAT(res.error().errnum, Eq(EINTR));
    }

    EXPECT_THAT(eintrRepetition, Eq(1));
    iox2::legacy::testing::TestingLogger::checkLogMessageIfLogLevelIsSupported(
        iox2::legacy::log::LogLevel::Error, [](const auto& logMessages) { ASSERT_GT(logMessages.size(), 0); });
}

TEST_F(PosixCall_test, CallingFunctionWithMultipleSuccessReturnValuesWhereGoodValueIsFirst) {
    ::testing::Test::RecordProperty("TEST_ID", "776689ef-8289-44f0-a509-a195dae025c0");

    constexpr int RETURN_VALUE = 25;
    constexpr int ERRNO_VALUE = 26;

    auto res = IOX2_POSIX_CALL(testFunction)(RETURN_VALUE, ERRNO_VALUE)
                   .successReturnValue(RETURN_VALUE, RETURN_VALUE - 1, RETURN_VALUE + 1, RETURN_VALUE + 2)
                   .evaluate();
    if (res.has_value()) {
        EXPECT_THAT(res->value, Eq(RETURN_VALUE));
        EXPECT_THAT(res->errnum, Eq(ERRNO_VALUE));
    } else {
        GTEST_FAIL();
    }

    iox2::legacy::testing::TestingLogger::checkLogMessageIfLogLevelIsSupported(
        iox2::legacy::log::LogLevel::Error, [](const auto& logMessages) { ASSERT_EQ(logMessages.size(), 0); });
}

TEST_F(PosixCall_test, CallingFunctionWithMultipleSuccessReturnValuesWhereGoodValueIsCenter) {
    ::testing::Test::RecordProperty("TEST_ID", "1b45e355-f809-4214-b227-18aa0c1585c5");

    constexpr int RETURN_VALUE = 27;
    constexpr int ERRNO_VALUE = 28;

    auto res = IOX2_POSIX_CALL(testFunction)(RETURN_VALUE, ERRNO_VALUE)
                   .successReturnValue(RETURN_VALUE - 1, RETURN_VALUE + 1, RETURN_VALUE, RETURN_VALUE + 2)
                   .evaluate();
    if (res.has_value()) {
        EXPECT_THAT(res->value, Eq(RETURN_VALUE));
        EXPECT_THAT(res->errnum, Eq(ERRNO_VALUE));
    } else {
        GTEST_FAIL();
    }

    iox2::legacy::testing::TestingLogger::checkLogMessageIfLogLevelIsSupported(
        iox2::legacy::log::LogLevel::Error, [](const auto& logMessages) { ASSERT_EQ(logMessages.size(), 0); });
}

TEST_F(PosixCall_test, CallingFunctionWithMultipleSuccessReturnValuesWhereGoodValueIsLast) {
    ::testing::Test::RecordProperty("TEST_ID", "0de42576-bb17-4596-9e91-e80f6bf2bdb5");

    constexpr int RETURN_VALUE = 29;
    constexpr int ERRNO_VALUE = 30;

    auto res = IOX2_POSIX_CALL(testFunction)(RETURN_VALUE, ERRNO_VALUE)
                   .successReturnValue(RETURN_VALUE - 1, RETURN_VALUE + 1, RETURN_VALUE + 2, RETURN_VALUE)
                   .evaluate();
    if (res.has_value()) {
        EXPECT_THAT(res->value, Eq(RETURN_VALUE));
        EXPECT_THAT(res->errnum, Eq(ERRNO_VALUE));
    } else {
        GTEST_FAIL();
    }

    iox2::legacy::testing::TestingLogger::checkLogMessageIfLogLevelIsSupported(
        iox2::legacy::log::LogLevel::Error, [](const auto& logMessages) { ASSERT_EQ(logMessages.size(), 0); });
}

TEST_F(PosixCall_test, CallingFunctionWithMultipleSuccessReturnValuesWhereGoodValueIsNotPresent) {
    ::testing::Test::RecordProperty("TEST_ID", "f8d0036f-8f99-467a-8d74-f8e736296e80");

    constexpr int RETURN_VALUE = 31;
    constexpr int ERRNO_VALUE = 32;

    auto res = IOX2_POSIX_CALL(testFunction)(RETURN_VALUE, ERRNO_VALUE)
                   .successReturnValue(RETURN_VALUE - 1, RETURN_VALUE + 1, RETURN_VALUE + 2)
                   .evaluate();
    if (res.has_value()) {
        GTEST_FAIL();
    } else {
        EXPECT_THAT(res.error().value, Eq(RETURN_VALUE));
        EXPECT_THAT(res.error().errnum, Eq(ERRNO_VALUE));
    }

    iox2::legacy::testing::TestingLogger::checkLogMessageIfLogLevelIsSupported(
        iox2::legacy::log::LogLevel::Error, [](const auto& logMessages) { ASSERT_GT(logMessages.size(), 0); });
}

TEST_F(PosixCall_test, CallingFunctionWithMultipleFailureReturnValuesWhereFailureValueIsFirst) {
    ::testing::Test::RecordProperty("TEST_ID", "d3000e92-5585-43d1-a857-085df9b3e890");

    constexpr int RETURN_VALUE = 33;
    constexpr int ERRNO_VALUE = 34;

    auto res = IOX2_POSIX_CALL(testFunction)(RETURN_VALUE, ERRNO_VALUE)
                   .failureReturnValue(RETURN_VALUE, RETURN_VALUE - 1, RETURN_VALUE + 1, RETURN_VALUE + 2)
                   .evaluate();
    if (res.has_value()) {
        GTEST_FAIL();
    } else {
        EXPECT_THAT(res.error().value, Eq(RETURN_VALUE));
        EXPECT_THAT(res.error().errnum, Eq(ERRNO_VALUE));
    }

    iox2::legacy::testing::TestingLogger::checkLogMessageIfLogLevelIsSupported(
        iox2::legacy::log::LogLevel::Error, [](const auto& logMessages) { ASSERT_GT(logMessages.size(), 0); });
}

TEST_F(PosixCall_test, CallingFunctionWithMultipleFailureReturnValuesWhereFailureValueIsCenter) {
    ::testing::Test::RecordProperty("TEST_ID", "280f24d2-22b2-4904-bab0-79dbcff0aad2");

    constexpr int RETURN_VALUE = 35;
    constexpr int ERRNO_VALUE = 36;

    auto res = IOX2_POSIX_CALL(testFunction)(RETURN_VALUE, ERRNO_VALUE)
                   .failureReturnValue(RETURN_VALUE - 1, RETURN_VALUE, RETURN_VALUE + 1, RETURN_VALUE + 2)
                   .evaluate();
    if (res.has_value()) {
        GTEST_FAIL();
    } else {
        EXPECT_THAT(res.error().value, Eq(RETURN_VALUE));
        EXPECT_THAT(res.error().errnum, Eq(ERRNO_VALUE));
    }

    iox2::legacy::testing::TestingLogger::checkLogMessageIfLogLevelIsSupported(
        iox2::legacy::log::LogLevel::Error, [](const auto& logMessages) { ASSERT_GT(logMessages.size(), 0); });
}

TEST_F(PosixCall_test, CallingFunctionWithMultipleFailureReturnValuesWhereFailureValueIsLast) {
    ::testing::Test::RecordProperty("TEST_ID", "21f34570-8d2f-45d1-aa9f-4c74bcdd7296");

    constexpr int RETURN_VALUE = 37;
    constexpr int ERRNO_VALUE = 38;

    auto res = IOX2_POSIX_CALL(testFunction)(RETURN_VALUE, ERRNO_VALUE)
                   .failureReturnValue(RETURN_VALUE - 1, RETURN_VALUE + 1, RETURN_VALUE + 2, RETURN_VALUE)
                   .evaluate();
    if (res.has_value()) {
        GTEST_FAIL();
    } else {
        EXPECT_THAT(res.error().value, Eq(RETURN_VALUE));
        EXPECT_THAT(res.error().errnum, Eq(ERRNO_VALUE));
    }

    iox2::legacy::testing::TestingLogger::checkLogMessageIfLogLevelIsSupported(
        iox2::legacy::log::LogLevel::Error, [](const auto& logMessages) { ASSERT_GT(logMessages.size(), 0); });
}

TEST_F(PosixCall_test, CallingFunctionWithMultipleFailureReturnValuesWhereFailureValueIsNotPresent) {
    ::testing::Test::RecordProperty("TEST_ID", "a8c6d31d-2214-447f-9d13-c60497641cc1");

    constexpr int RETURN_VALUE = 39;
    constexpr int ERRNO_VALUE = 40;

    auto res = IOX2_POSIX_CALL(testFunction)(RETURN_VALUE, ERRNO_VALUE)
                   .failureReturnValue(RETURN_VALUE - 1, RETURN_VALUE + 1, RETURN_VALUE + 2)
                   .evaluate();
    if (res.has_value()) {
        EXPECT_THAT(res->value, Eq(RETURN_VALUE));
        EXPECT_THAT(res->errnum, Eq(ERRNO_VALUE));
    } else {
        GTEST_FAIL();
    }

    iox2::legacy::testing::TestingLogger::checkLogMessageIfLogLevelIsSupported(
        iox2::legacy::log::LogLevel::Error, [](const auto& logMessages) { ASSERT_EQ(logMessages.size(), 0); });
}

TEST_F(PosixCall_test, ErrnoIsSetFromReturnValueWhenFunctionHandlesErrnosInReturnValue_GoodCase) {
    ::testing::Test::RecordProperty("TEST_ID", "b2dc8737-703d-4a85-a370-1c6e53f845a0");

    constexpr int RETURN_VALUE = 0;

    auto res = IOX2_POSIX_CALL(returnValueIsErrno)(RETURN_VALUE).returnValueMatchesErrno().evaluate();
    if (res.has_value()) {
        EXPECT_THAT(res->value, Eq(RETURN_VALUE));
        EXPECT_THAT(res->errnum, Eq(0));
    } else {
        GTEST_FAIL();
    }

    iox2::legacy::testing::TestingLogger::checkLogMessageIfLogLevelIsSupported(
        iox2::legacy::log::LogLevel::Error, [](const auto& logMessages) { ASSERT_EQ(logMessages.size(), 0); });
}

TEST_F(PosixCall_test, ErrnoIsSetFromReturnValueWhenFunctionHandlesErrnosInReturnValue_BadCase) {
    ::testing::Test::RecordProperty("TEST_ID", "ae8aa873-d5f0-4301-8325-506c14a49393");

    constexpr int RETURN_VALUE = 42;

    auto res = IOX2_POSIX_CALL(returnValueIsErrno)(RETURN_VALUE).returnValueMatchesErrno().evaluate();
    if (res.has_value()) {
        GTEST_FAIL();
    } else {
        EXPECT_THAT(res.error().value, Eq(RETURN_VALUE));
        EXPECT_THAT(res.error().errnum, Eq(RETURN_VALUE));
    }

    iox2::legacy::testing::TestingLogger::checkLogMessageIfLogLevelIsSupported(
        iox2::legacy::log::LogLevel::Error, [](const auto& logMessages) { ASSERT_GT(logMessages.size(), 0); });
}
