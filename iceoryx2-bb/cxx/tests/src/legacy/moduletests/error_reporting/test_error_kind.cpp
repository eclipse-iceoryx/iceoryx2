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

#include <gtest/gtest.h>

#include "iox2/legacy/error_reporting/custom/error_kind.hpp"
#include "iox2/legacy/error_reporting/error_kind.hpp"

namespace {

using namespace ::testing;
using namespace iox2::legacy::er;

// check the type traits of the error kinds

// tautologies are always true ...
TEST(ErrorKind_test, fatalErrorsAreFatal) {
    ::testing::Test::RecordProperty("TEST_ID", "2524192f-a29c-45bc-b950-ae29deb8b3ae");

    FatalKind sut;
    EXPECT_TRUE(IsFatal<FatalKind>::value);
    EXPECT_TRUE(isFatal(sut));
}

TEST(ErrorKind_test, enforceViolationsAreFatal) {
    ::testing::Test::RecordProperty("TEST_ID", "21b79757-e46b-44fe-854a-7579b7f2243b");

    EnforceViolationKind sut;
    EXPECT_TRUE(IsFatal<EnforceViolationKind>::value);
    EXPECT_TRUE(isFatal(sut));
    EXPECT_TRUE(isFatal(ENFORCE_VIOLATION));
}

TEST(ErrorKind_test, assertViolationsAreFatal) {
    ::testing::Test::RecordProperty("TEST_ID", "b502e70e-157d-45a0-9654-61ada213531d");

    AssertViolationKind sut;
    EXPECT_TRUE(IsFatal<AssertViolationKind>::value);
    EXPECT_TRUE(isFatal(sut));
    EXPECT_TRUE(isFatal(ASSERT_VIOLATION));
}

TEST(ErrorKind_test, runtimeErrorsAreNotFatal) {
    ::testing::Test::RecordProperty("TEST_ID", "22c69c24-5082-4e81-8b3f-306e624731a5");

    RuntimeErrorKind sut;
    EXPECT_FALSE(IsFatal<RuntimeErrorKind>::value);
    EXPECT_FALSE(isFatal(sut));
}

} // namespace
