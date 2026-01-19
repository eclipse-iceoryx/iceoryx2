// Copyright (c) 2019 - 2020 by Robert Bosch GmbH. All rights reserved.
// Copyright (c) 2021 - 2023 by Apex.AI Inc. All rights reserved.
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

#include "iox2/bb/std_chrono_support.hpp"

#include <gmock/gmock.h>
#include <gtest/gtest.h>

namespace {
using namespace ::testing;
using namespace iox2::bb;
using namespace iox2::bb::duration_literals;

constexpr uint64_t NANOSECS_PER_MILLISECOND = Duration::NANOSECS_PER_MILLISEC;
constexpr uint64_t NANOSECS_PER_SECOND = Duration::NANOSECS_PER_SEC;

class StdChronoTest : public Test { };

// BEGIN CONSTRUCTION TESTS

TEST(StdChrono, construct_from_chrono_milliseconds_zero) {
    ::testing::Test::RecordProperty("TEST_ID", "40b02547-8a9d-4ae6-90b2-72e76e5143f0");
    constexpr uint64_t EXPECTED_MILLISECONDS { 0U };
    const Duration sut = into<Duration>(std::chrono::milliseconds(EXPECTED_MILLISECONDS));
    EXPECT_THAT(sut.as_nanos(), Eq(0U));
}

TEST(StdChrono, construct_from_chrono_milliseconds_less_than_one_second) {
    ::testing::Test::RecordProperty("TEST_ID", "ccbcd8df-c146-48ea-a9a0-40abaff31e68");
    constexpr uint64_t EXPECTED_MILLISECONDS { 44U };
    const Duration sut = into<Duration>(std::chrono::milliseconds(EXPECTED_MILLISECONDS));
    EXPECT_THAT(sut.as_nanos(), Eq(EXPECTED_MILLISECONDS * NANOSECS_PER_MILLISECOND));
}

TEST(StdChrono, construct_from_chrono_milliseconds_more_than_one_second) {
    ::testing::Test::RecordProperty("TEST_ID", "04313e7e-2954-4741-ad0e-6f5a9e5aebce");
    constexpr uint64_t EXPECTED_MILLISECONDS { 1001 };
    const Duration sut = into<Duration>(std::chrono::milliseconds(EXPECTED_MILLISECONDS));
    EXPECT_THAT(sut.as_nanos(), Eq(EXPECTED_MILLISECONDS * NANOSECS_PER_MILLISECOND));
}

TEST(StdChrono, construct_from_chrono_milliseconds_max) {
    ::testing::Test::RecordProperty("TEST_ID", "553d4ab1-ff8a-437e-8be8-8105e62850c6");
    constexpr uint64_t EXPECTED_MILLISECONDS { std::numeric_limits<int64_t>::max() };
    const Duration sut = into<Duration>(std::chrono::milliseconds(EXPECTED_MILLISECONDS));
    EXPECT_THAT(sut.as_millis(), Eq(EXPECTED_MILLISECONDS));
}

TEST(StdChrono, construct_from_negative_chrono_milliseconds_is_zero) {
    ::testing::Test::RecordProperty("TEST_ID", "181bc67d-1674-44eb-8784-710306efde30");
    const Duration sut = into<Duration>(std::chrono::milliseconds(-1));
    EXPECT_THAT(sut.as_nanos(), Eq(0U));
}

TEST(StdChrono, construct_from_chrono_nanoseconds_zero) {
    ::testing::Test::RecordProperty("TEST_ID", "3044b00f-a765-417b-a16b-da01c16f7ed0");
    constexpr uint64_t EXPECTED_NANOSECONDS { 0U };
    const Duration sut = into<Duration>(std::chrono::nanoseconds(EXPECTED_NANOSECONDS));
    EXPECT_THAT(sut.as_nanos(), Eq(EXPECTED_NANOSECONDS));
}

TEST(StdChrono, construct_from_chrono_nanoseconds_less_than_one_second) {
    ::testing::Test::RecordProperty("TEST_ID", "a81475d5-5732-44f4-91fc-950792808d7a");
    constexpr uint64_t EXPECTED_NANOSECONDS { 424242U };
    const Duration sut = into<Duration>(std::chrono::nanoseconds(EXPECTED_NANOSECONDS));
    EXPECT_THAT(sut.as_nanos(), Eq(EXPECTED_NANOSECONDS));
}

TEST(StdChrono, construct_from_chrono_nanoseconds_more_than_one_second) {
    ::testing::Test::RecordProperty("TEST_ID", "43c49a79-28a6-4df4-862d-fc3aaa363191");
    constexpr uint64_t EXPECTED_NANOSECONDS { NANOSECS_PER_SECOND + 42U };
    const Duration sut = into<Duration>(std::chrono::nanoseconds(EXPECTED_NANOSECONDS));
    EXPECT_THAT(sut.as_nanos(), Eq(EXPECTED_NANOSECONDS));
}

TEST(StdChrono, construct_from_chrono_nanoseconds_max) {
    ::testing::Test::RecordProperty("TEST_ID", "a2520ffb-ecc9-4bbb-9f7e-0fbe05383efc");
    constexpr uint64_t EXPECTED_NANOSECONDS { std::numeric_limits<int64_t>::max() };
    const Duration sut = into<Duration>(std::chrono::nanoseconds(EXPECTED_NANOSECONDS));
    EXPECT_THAT(sut.as_nanos(), Eq(EXPECTED_NANOSECONDS));
}

TEST(StdChrono, construct_from_negative_chrono_nanoseconds_is_zero) {
    ::testing::Test::RecordProperty("TEST_ID", "11993efd-9f4f-41b0-a07b-84732d7e6f71");
    const Duration sut = into<Duration>(std::chrono::nanoseconds(-1));
    EXPECT_THAT(sut.as_nanos(), Eq(0U));
}

// END CONSTRUCTION TESTS

} // namespace
