// Copyright (c) 2022 by Apex.AI Inc. All rights reserved.
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

#include "iox2/bb/detail/assertions.hpp"
#include "iox2/bb/testing/fatal_failure.hpp"

#include <gmock/gmock.h>
#include <gtest/gtest.h>

namespace {
using namespace ::testing;
using namespace ::iox2::bb::testing;

TEST(FatalFailure, UsingExpectFatalFailureWorks) {
    ::testing::Test::RecordProperty("TEST_ID", "26393210-9738-462f-9d35-dbd53fbae9d2");

#ifdef _WIN32
    GTEST_SKIP() << "The 'UsingExpectFatalFailureWorks' test is disabled on Windows";
#else
    auto has_fatal_failure = IOX2_TESTING_EXPECT_FATAL_FAILURE([&]() -> auto { IOX2_ENFORCE(false, ""); },
                                                               iox2::legacy::er::ENFORCE_VIOLATION);

    EXPECT_TRUE(has_fatal_failure);
#endif
}

TEST(FatalFailure, UsingExpectNoFatalFailureWorks) {
    ::testing::Test::RecordProperty("TEST_ID", "80bf8050-bfaa-4482-b69c-d0c80699bd4b");

    auto has_no_fatal_failure = IOX2_TESTING_EXPECT_NO_FATAL_FAILURE([&]() -> auto { });

    EXPECT_TRUE(has_no_fatal_failure);
}
} // namespace
