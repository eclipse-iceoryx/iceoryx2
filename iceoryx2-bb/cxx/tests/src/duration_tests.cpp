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

#include "iox2/bb/duration.hpp"
#include "iox2/legacy/testing/mocks/logger_mock.hpp"

#include <gmock/gmock.h>
#include <gtest/gtest.h>

#include <iostream>
#include <limits>
#include <ostream>

namespace {
using namespace ::testing;
using namespace iox2::bb;
using namespace iox2::bb::duration_literals;

constexpr uint64_t SECONDS_PER_MINUTE = Duration::SECS_PER_MINUTE;
constexpr uint64_t SECONDS_PER_HOUR = Duration::SECS_PER_HOUR;
constexpr uint64_t HOURS_PER_DAY = Duration::HOURS_PER_DAY;

constexpr uint64_t MILLISECS_PER_SECOND = Duration::MILLISECS_PER_SEC;
constexpr uint64_t MICROSECS_PER_SECONDS = Duration::MICROSECS_PER_SEC;

constexpr uint64_t NANOSECS_PER_MICROSECOND = Duration::NANOSECS_PER_MICROSEC;
constexpr uint64_t NANOSECS_PER_MILLISECOND = Duration::NANOSECS_PER_MILLISEC;
constexpr uint64_t NANOSECS_PER_SECOND = Duration::NANOSECS_PER_SEC;

struct DurationAccessor : public Duration {
    using Duration::create_duration;
    using Duration::max;

    using Duration::NanosecondsT;
    using Duration::SecondsT;
};

constexpr auto create_duration(DurationAccessor::SecondsT seconds, DurationAccessor::NanosecondsT nanoseconds)
    -> Duration {
    return DurationAccessor::create_duration(seconds, nanoseconds);
}

TEST(Duration, conversion_constants) {
    ::testing::Test::RecordProperty("TEST_ID", "1d9090fc-438c-41dc-9350-04910ef9b27d");
    static_assert(Duration::SECS_PER_MINUTE == 60U, "Mismatch for conversion constants!"); // NOLINT
    static_assert(Duration::SECS_PER_HOUR == 3600U, "Mismatch for conversion constants!"); // NOLINT
    static_assert(Duration::HOURS_PER_DAY == 24U, "Mismatch for conversion constants!");   // NOLINT

    static_assert(Duration::MILLISECS_PER_SEC == 1000U, "Mismatch for conversion constants!");    // NOLINT
    static_assert(Duration::MICROSECS_PER_SEC == 1000000U, "Mismatch for conversion constants!"); // NOLINT

    static_assert(Duration::NANOSECS_PER_MICROSEC == 1000U, "Mismatch for conversion constants!");    // NOLINT
    static_assert(Duration::NANOSECS_PER_MILLISEC == 1000000U, "Mismatch for conversion constants!"); // NOLINT
    static_assert(Duration::NANOSECS_PER_SEC == 1000000000U, "Mismatch for conversion constants!");   // NOLINT
}

// BEGIN CONSTRUCTOR TESTS

TEST(Duration, construct_duration_with_zero_time) {
    ::testing::Test::RecordProperty("TEST_ID", "d2daaadc-b756-4c40-8bdf-e648160d7168");
    constexpr uint64_t SECONDS { 0U };
    constexpr uint64_t NANOSECONDS { 0U };
    constexpr uint64_t EXPECTED_DURATION_IN_NANOSECONDS { 0U };

    auto sut = create_duration(SECONDS, NANOSECONDS);

    EXPECT_THAT(sut.as_nanos(), Eq(EXPECTED_DURATION_IN_NANOSECONDS));
}

TEST(Duration, construct_duration_with_result_of_less_nanoseconds_than_one_second) {
    ::testing::Test::RecordProperty("TEST_ID", "2e4d5165-9a01-49fd-8dca-ca4914f3604b");
    constexpr uint64_t SECONDS { 0U };
    constexpr uint64_t NANOSECONDS { 7337U };
    constexpr uint64_t EXPECTED_DURATION_IN_NANOSECONDS { NANOSECONDS };

    auto sut = create_duration(SECONDS, NANOSECONDS);

    EXPECT_THAT(sut.as_nanos(), Eq(EXPECTED_DURATION_IN_NANOSECONDS));
}

TEST(Duration, construct_duration_with_nanoseconds_less_than_one_second) {
    ::testing::Test::RecordProperty("TEST_ID", "7de47e88-c4b3-4655-bdeb-3c51f3b34d3e");
    constexpr uint64_t SECONDS { 37U };
    constexpr uint64_t NANOSECONDS { 73U };
    constexpr uint64_t EXPECTED_DURATION_IN_NANOSECONDS { (SECONDS * NANOSECS_PER_SECOND) + NANOSECONDS };

    auto sut = create_duration(SECONDS, NANOSECONDS);

    EXPECT_THAT(sut.as_nanos(), Eq(EXPECTED_DURATION_IN_NANOSECONDS));
}

TEST(Duration, construct_duration_with_nanoseconds_equal_to_one_second) {
    ::testing::Test::RecordProperty("TEST_ID", "617e61f1-6322-4f9b-9814-3a645a9a11ea");
    constexpr uint64_t SECONDS { 13U };
    constexpr uint64_t NANOSECONDS { NANOSECS_PER_SECOND };
    constexpr uint64_t EXPECTED_DURATION_IN_NANOSECONDS { (SECONDS + 1U) * NANOSECS_PER_SECOND };

    auto sut = create_duration(SECONDS, NANOSECONDS);

    EXPECT_THAT(sut.as_nanos(), Eq(EXPECTED_DURATION_IN_NANOSECONDS));
}

TEST(Duration, construct_duration_with_nanoseconds_more_than_one_second) {
    ::testing::Test::RecordProperty("TEST_ID", "f1bc3127-279c-4cf2-abfb-03a71a5e7e48");
    constexpr uint64_t SECONDS { 37U };
    constexpr uint64_t NANOSECONDS { 42U };
    constexpr uint64_t MORE_THAN_ONE_SECOND_NANOSECONDS { NANOSECS_PER_SECOND + NANOSECONDS };
    constexpr uint64_t EXPECTED_DURATION_IN_NANOSECONDS { ((SECONDS + 1U) * NANOSECS_PER_SECOND) + NANOSECONDS };

    auto sut = create_duration(SECONDS, MORE_THAN_ONE_SECOND_NANOSECONDS);

    EXPECT_THAT(sut.as_nanos(), Eq(EXPECTED_DURATION_IN_NANOSECONDS));
}

TEST(Duration, construct_duration_with_nanoseconds_max_value) {
    ::testing::Test::RecordProperty("TEST_ID", "fe8152ca-cd14-4d99-aa68-0f56ab3007de");
    constexpr uint64_t SECONDS { 37U };
    constexpr uint64_t MAX_NANOSECONDS_FOR_CTOR { std::numeric_limits<DurationAccessor::NanosecondsT>::max() };
    constexpr uint64_t EXPECTED_SECONDS = SECONDS + (MAX_NANOSECONDS_FOR_CTOR / NANOSECS_PER_SECOND);
    constexpr uint64_t REMAINING_NANOSECONDS = MAX_NANOSECONDS_FOR_CTOR % NANOSECS_PER_SECOND;
    constexpr uint64_t EXPECTED_DURATION_IN_NANOSECONDS { ((EXPECTED_SECONDS) *NANOSECS_PER_SECOND)
                                                          + REMAINING_NANOSECONDS };

    auto sut = create_duration(SECONDS, MAX_NANOSECONDS_FOR_CTOR);

    EXPECT_THAT(sut.as_nanos(), Eq(EXPECTED_DURATION_IN_NANOSECONDS));
}

TEST(Duration, construct_duration_with_seconds_and_nanoseconds_max_values) {
    ::testing::Test::RecordProperty("TEST_ID", "bf672288-064f-4033-9b6f-0345e38a066f");
    constexpr uint64_t MAX_SECONDS_FOR_CTOR { std::numeric_limits<DurationAccessor::SecondsT>::max() };
    constexpr uint64_t MAX_NANOSECONDS_FOR_CTOR { std::numeric_limits<DurationAccessor::NanosecondsT>::max() };

    auto sut = create_duration(MAX_SECONDS_FOR_CTOR, MAX_NANOSECONDS_FOR_CTOR);

    EXPECT_THAT(sut, Eq(DurationAccessor::max()));
}

TEST(Duration, construct_duration_with_one_nanosecond_results_not_in_zero_nanoseconds) {
    ::testing::Test::RecordProperty("TEST_ID", "3f1948aa-e008-4aa0-b574-b2a068c1aabf");
    constexpr uint64_t SECONDS { 0U };
    constexpr uint64_t NANOSECONDS { 1U };
    constexpr uint64_t EXPECTED_DURATION_IN_NANOSECONDS { NANOSECONDS };

    auto sut = create_duration(SECONDS, NANOSECONDS);

    EXPECT_THAT(sut.as_nanos(), Eq(EXPECTED_DURATION_IN_NANOSECONDS));
}

// END CONSTRUCTOR TESTS

// BEGIN CREATION FROM LITERAL TESTS

TEST(Duration, create_duration_from_days_literal) {
    ::testing::Test::RecordProperty("TEST_ID", "0e1ff08d-4969-46a8-b781-1ad92ef1d792");
    constexpr uint64_t EXPECTED_DURATION_IN_NANOSECONDS { 2U * HOURS_PER_DAY * SECONDS_PER_HOUR * NANOSECS_PER_SECOND };
    auto sut = 2_d;

    EXPECT_THAT(sut.as_nanos(), Eq(EXPECTED_DURATION_IN_NANOSECONDS));
}

TEST(Duration, create_duration_from_hours_literal) {
    ::testing::Test::RecordProperty("TEST_ID", "834cbe33-711d-4ea1-b3d3-06246f588efe");
    constexpr uint64_t EXPECTED_DURATION_IN_NANOSECONDS { 3U * SECONDS_PER_HOUR * NANOSECS_PER_SECOND };
    auto sut = 3_h;

    EXPECT_THAT(sut.as_nanos(), Eq(EXPECTED_DURATION_IN_NANOSECONDS));
}

TEST(Duration, create_duration_from_minutes_literal) {
    ::testing::Test::RecordProperty("TEST_ID", "58dcec41-09e3-4995-a594-dcdf13d2a670");
    constexpr uint64_t EXPECTED_DURATION_IN_NANOSECONDS { 4U * SECONDS_PER_MINUTE * NANOSECS_PER_SECOND };
    auto sut = 4_m;

    EXPECT_THAT(sut.as_nanos(), Eq(EXPECTED_DURATION_IN_NANOSECONDS));
}

TEST(Duration, create_duration_from_seconds_literal) {
    ::testing::Test::RecordProperty("TEST_ID", "56640d8b-dde0-4355-84af-ded9be418375");
    constexpr uint64_t EXPECTED_DURATION_IN_NANOSECONDS { 5U * NANOSECS_PER_SECOND };
    const auto sut = 5_s;

    EXPECT_THAT(sut.as_nanos(), Eq(EXPECTED_DURATION_IN_NANOSECONDS));
}

TEST(Duration, create_duration_from_milliseconds_literal) {
    ::testing::Test::RecordProperty("TEST_ID", "e19dec13-b5d4-46b7-9d23-9797a2163b5a");
    constexpr uint64_t EXPECTED_DURATION_IN_NANOSECONDS { 6U * NANOSECS_PER_MILLISECOND };
    const auto sut = 6_ms;

    EXPECT_THAT(sut.as_nanos(), Eq(EXPECTED_DURATION_IN_NANOSECONDS));
}

TEST(Duration, create_duration_from_microseconds_literal) {
    ::testing::Test::RecordProperty("TEST_ID", "6138cd81-034d-46cd-8063-1eb0989823cd");
    constexpr uint64_t EXPECTED_DURATION_IN_NANOSECONDS { 7U * NANOSECS_PER_MICROSECOND };
    const auto sut = 7_us;

    EXPECT_THAT(sut.as_nanos(), Eq(EXPECTED_DURATION_IN_NANOSECONDS));
}

TEST(Duration, create_duration_from_nanoseconds_literal) {
    ::testing::Test::RecordProperty("TEST_ID", "c05c8562-84fe-430e-9e3a-04b047aff48c");
    constexpr uint64_t EXPECTED_DURATION_IN_NANOSECONDS { 8U };
    const auto sut = 8_ns;

    EXPECT_THAT(sut.as_nanos(), Eq(EXPECTED_DURATION_IN_NANOSECONDS));
}

// END CREATION FROM LITERAL TESTS

// BEGIN CREATION FROM STATIC FUNCTION TESTS

TEST(Duration, create_duration_from_days_function_with_zero_days) {
    ::testing::Test::RecordProperty("TEST_ID", "bdac8676-3d59-4dff-922a-5eea4bf850b9");
    auto sut1 = Duration::from_days(0);
    auto sut2 = Duration::from_days(0U);

    EXPECT_THAT(sut1.as_nanos(), Eq(0U));
    EXPECT_THAT(sut2.as_nanos(), Eq(0U));
}

TEST(Duration, create_duration_from_days_function_with_multiple_days) {
    ::testing::Test::RecordProperty("TEST_ID", "25eeee95-1b1c-405e-a066-d1c35a9ec40e");
    constexpr uint64_t EXPECTED_DURATION_IN_NANOSECONDS { static_cast<uint64_t>(2U * 24U) * SECONDS_PER_HOUR
                                                          * NANOSECS_PER_SECOND };
    auto sut1 = Duration::from_days(2);
    auto sut2 = Duration::from_days(2U);

    EXPECT_THAT(sut1.as_nanos(), Eq(EXPECTED_DURATION_IN_NANOSECONDS));
    EXPECT_THAT(sut2.as_nanos(), Eq(EXPECTED_DURATION_IN_NANOSECONDS));
}

TEST(Duration, create_duration_from_days_function_with_days_results_not_yet_in_saturation) {
    ::testing::Test::RecordProperty("TEST_ID", "3f34a190-d9bf-4206-b635-9d170df87053");
    constexpr uint64_t SECONDS_PER_DAY { HOURS_PER_DAY * SECONDS_PER_HOUR };
    constexpr uint64_t MAX_DAYS_BEFORE_OVERFLOW { std::numeric_limits<DurationAccessor::SecondsT>::max()
                                                  / SECONDS_PER_DAY };
    constexpr Duration EXPECTED_DURATION = create_duration(MAX_DAYS_BEFORE_OVERFLOW * SECONDS_PER_DAY, 0U);
    static_assert(EXPECTED_DURATION < DurationAccessor::max(),
                  "EXPECTED_DURATION too large to exclude saturation! Please decrease!");

    auto sut1 = Duration::from_days(static_cast<int64_t>(MAX_DAYS_BEFORE_OVERFLOW));
    auto sut2 = Duration::from_days(MAX_DAYS_BEFORE_OVERFLOW);

    EXPECT_THAT(sut1, Eq(EXPECTED_DURATION));
    EXPECT_THAT(sut2, Eq(EXPECTED_DURATION));
}

TEST(Duration, create_duration_from_days_function_with_max_days_results_in_saturation) {
    ::testing::Test::RecordProperty("TEST_ID", "b802d652-b04c-41b5-a703-36af3a0dd1af");
    auto sut1 = Duration::from_days(std::numeric_limits<int64_t>::max());
    auto sut2 = Duration::from_days(std::numeric_limits<uint64_t>::max());

    EXPECT_THAT(sut1, Eq(DurationAccessor::max()));
    EXPECT_THAT(sut2, Eq(DurationAccessor::max()));
}

TEST(Duration, create_duration_from_days_function_with_negative_values_is_zero) {
    ::testing::Test::RecordProperty("TEST_ID", "f22a4350-b647-4ab1-b2c2-49056503e8f7");
    auto sut = Duration::from_days(-1);
    EXPECT_THAT(sut.as_nanos(), Eq(0U));
}

TEST(Duration, create_duration_from_hours_function_with_zero_hours) {
    ::testing::Test::RecordProperty("TEST_ID", "35e6fd8d-8afe-4f85-a557-abc5e783b055");
    auto sut1 = Duration::from_hours(0);
    auto sut2 = Duration::from_hours(0U);

    EXPECT_THAT(sut1.as_nanos(), Eq(0U));
    EXPECT_THAT(sut2.as_nanos(), Eq(0U));
}

TEST(Duration, create_duration_from_hours_function_with_multiple_hours) {
    ::testing::Test::RecordProperty("TEST_ID", "dd12fcff-2d2f-4790-a867-a62399115145");
    constexpr uint64_t EXPECTED_DURATION_IN_NANOSECONDS { 3U * SECONDS_PER_HOUR * NANOSECS_PER_SECOND };
    auto sut1 = Duration::from_hours(3);
    auto sut2 = Duration::from_hours(3U);

    EXPECT_THAT(sut1.as_nanos(), Eq(EXPECTED_DURATION_IN_NANOSECONDS));
    EXPECT_THAT(sut2.as_nanos(), Eq(EXPECTED_DURATION_IN_NANOSECONDS));
}

TEST(Duration, create_duration_from_hours_function_with_hours_results_not_yet_in_saturation) {
    ::testing::Test::RecordProperty("TEST_ID", "90d340f5-09e2-4b8b-a916-f2da6a28377f");
    constexpr uint64_t MAX_HOURS_BEFORE_OVERFLOW { std::numeric_limits<DurationAccessor::SecondsT>::max()
                                                   / SECONDS_PER_HOUR };
    constexpr Duration EXPECTED_DURATION = create_duration(MAX_HOURS_BEFORE_OVERFLOW * SECONDS_PER_HOUR, 0U);
    static_assert(EXPECTED_DURATION < DurationAccessor::max(),
                  "EXPECTED_DURATION too large to exclude saturation! Please decrease!");

    auto sut1 = Duration::from_hours(static_cast<int64_t>(MAX_HOURS_BEFORE_OVERFLOW));
    auto sut2 = Duration::from_hours(MAX_HOURS_BEFORE_OVERFLOW);

    EXPECT_THAT(sut1, Eq(EXPECTED_DURATION));
    EXPECT_THAT(sut2, Eq(EXPECTED_DURATION));
}

TEST(Duration, create_duration_from_hours_function_with_max_hours_results_in_saturation) {
    ::testing::Test::RecordProperty("TEST_ID", "dc6c3f1e-0329-48aa-8f0a-d67655a8128f");
    auto sut1 = Duration::from_hours(std::numeric_limits<int64_t>::max());
    auto sut2 = Duration::from_hours(std::numeric_limits<uint64_t>::max());

    EXPECT_THAT(sut1, Eq(DurationAccessor::max()));
    EXPECT_THAT(sut2, Eq(DurationAccessor::max()));
}

TEST(Duration, create_duration_from_hours_function_with_negative_value_is_zero) {
    ::testing::Test::RecordProperty("TEST_ID", "8180c3b7-1a60-40de-a5b5-21f81da46fbf");
    auto sut = Duration::from_hours(-1);
    EXPECT_THAT(sut.as_nanos(), Eq(0U));
}

TEST(Duration, create_duration_from_mins_function_with_zero_minuts) {
    ::testing::Test::RecordProperty("TEST_ID", "9d5b6083-2774-45fa-80d2-e31a38016a38");
    auto sut1 = Duration::from_mins(0);
    auto sut2 = Duration::from_mins(0U);

    EXPECT_THAT(sut1.as_nanos(), Eq(0U));
    EXPECT_THAT(sut2.as_nanos(), Eq(0U));
}

TEST(Duration, create_duration_from_mins_function_with_multiple_minutes) {
    ::testing::Test::RecordProperty("TEST_ID", "fb8eb9da-7bbd-400d-9ba0-132fa9150a90");
    constexpr uint64_t EXPECTED_DURATION_IN_NANOSECONDS { 4U * SECONDS_PER_MINUTE * NANOSECS_PER_SECOND };
    auto sut1 = Duration::from_mins(4);
    auto sut2 = Duration::from_mins(4U);

    EXPECT_THAT(sut1.as_nanos(), Eq(EXPECTED_DURATION_IN_NANOSECONDS));
    EXPECT_THAT(sut2.as_nanos(), Eq(EXPECTED_DURATION_IN_NANOSECONDS));
}

TEST(Duration, create_duration_from_mins_function_with_minutes_results_not_yet_in_saturation) {
    ::testing::Test::RecordProperty("TEST_ID", "21825950-16ff-4ade-b8f6-41fa8d00596a");
    constexpr uint64_t MAX_MINUTES_BEFORE_OVERFLOW { std::numeric_limits<DurationAccessor::SecondsT>::max()
                                                     / SECONDS_PER_MINUTE };
    constexpr Duration EXPECTED_DURATION = create_duration(MAX_MINUTES_BEFORE_OVERFLOW * SECONDS_PER_MINUTE, 0U);
    static_assert(EXPECTED_DURATION < DurationAccessor::max(),
                  "EXPECTED_DURATION too large to exclude saturation! Please decrease!");

    auto sut1 = Duration::from_mins(static_cast<int64_t>(MAX_MINUTES_BEFORE_OVERFLOW));
    auto sut2 = Duration::from_mins(MAX_MINUTES_BEFORE_OVERFLOW);

    EXPECT_THAT(sut1, Eq(EXPECTED_DURATION));
    EXPECT_THAT(sut2, Eq(EXPECTED_DURATION));
}

TEST(Duration, create_duration_from_mins_function_with_max_minutes_results_in_saturation) {
    ::testing::Test::RecordProperty("TEST_ID", "a0ef1468-048b-4a9f-8e96-ef68efc2ae97");
    auto sut1 = Duration::from_mins(std::numeric_limits<int64_t>::max());
    auto sut2 = Duration::from_mins(std::numeric_limits<uint64_t>::max());

    EXPECT_THAT(sut1, Eq(DurationAccessor::max()));
    EXPECT_THAT(sut2, Eq(DurationAccessor::max()));
}

TEST(Duration, create_duration_from_mins_function_with_negative_value_is_zero) {
    ::testing::Test::RecordProperty("TEST_ID", "06cf0c89-7fcb-4b36-8c2a-000753820b74");
    auto sut = Duration::from_mins(-1);
    EXPECT_THAT(sut.as_nanos(), Eq(0U));
}

TEST(Duration, create_duration_from_secs_function_with_zero_seconds) {
    ::testing::Test::RecordProperty("TEST_ID", "4f221bb9-de92-4e59-b807-c4388c6adf72");
    auto sut1 = Duration::from_secs(0);
    auto sut2 = Duration::from_secs(0U);

    EXPECT_THAT(sut1.as_nanos(), Eq(0U));
    EXPECT_THAT(sut2.as_nanos(), Eq(0U));
}

TEST(Duration, create_duration_from_secs_function) {
    ::testing::Test::RecordProperty("TEST_ID", "85905c6d-b07b-4bd0-8d3d-e6a8e5b79c38");
    constexpr uint64_t EXPECTED_DURATION_IN_NANOSECONDS { 5U * NANOSECS_PER_SECOND };
    const auto sut1 = Duration::from_secs(5);
    const auto sut2 = Duration::from_secs(5U);

    EXPECT_THAT(sut1.as_nanos(), Eq(EXPECTED_DURATION_IN_NANOSECONDS));
    EXPECT_THAT(sut2.as_nanos(), Eq(EXPECTED_DURATION_IN_NANOSECONDS));
}

TEST(Duration, create_duration_from_secs_function_with_max_seconds) {
    ::testing::Test::RecordProperty("TEST_ID", "ba61d3c6-29ae-4b78-9a19-4c87bc791dd6");
    constexpr uint64_t MAX_SECONDS_FROM_SIGNED { static_cast<uint64_t>(std::numeric_limits<int64_t>::max()) };
    constexpr Duration EXPECTED_DURATION_FROM_MAX_SIGNED = create_duration(MAX_SECONDS_FROM_SIGNED, 0U);
    constexpr uint64_t MAX_SECONDS_FROM_USIGNED { std::numeric_limits<uint64_t>::max() };
    constexpr Duration EXPECTED_DURATION_FROM_MAX_UNSIGNED = create_duration(MAX_SECONDS_FROM_USIGNED, 0U);

    auto sut1 = Duration::from_secs(std::numeric_limits<int64_t>::max());
    auto sut2 = Duration::from_secs(std::numeric_limits<uint64_t>::max());

    EXPECT_THAT(sut1, Eq(EXPECTED_DURATION_FROM_MAX_SIGNED));
    EXPECT_THAT(sut2, Eq(EXPECTED_DURATION_FROM_MAX_UNSIGNED));
}

TEST(Duration, create_duration_from_secs_function_with_negative_value_is_zero) {
    ::testing::Test::RecordProperty("TEST_ID", "9a9125b2-5ec5-4315-89f5-362e1fddb7c8");
    auto sut = Duration::from_secs(-1);
    EXPECT_THAT(sut.as_nanos(), Eq(0U));
}

TEST(Duration, create_duration_from_millis_function_with_zero_milliseconds) {
    ::testing::Test::RecordProperty("TEST_ID", "710070e8-e83e-4977-9aa3-9335b24a0feb");
    auto sut1 = Duration::from_millis(0);
    auto sut2 = Duration::from_millis(0U);

    EXPECT_THAT(sut1.as_nanos(), Eq(0U));
    EXPECT_THAT(sut2.as_nanos(), Eq(0U));
}

TEST(Duration, create_duration_from_millis_function_with_multiple_milliseconds) {
    ::testing::Test::RecordProperty("TEST_ID", "1bd8f1fd-ca51-4f44-93f4-68680573cc34");
    constexpr uint64_t EXPECTED_DURATION_IN_NANOSECONDS { 6U * NANOSECS_PER_MILLISECOND };
    const auto sut1 = Duration::from_millis(6);
    const auto sut2 = Duration::from_millis(6U);

    EXPECT_THAT(sut1.as_nanos(), Eq(EXPECTED_DURATION_IN_NANOSECONDS));
    EXPECT_THAT(sut2.as_nanos(), Eq(EXPECTED_DURATION_IN_NANOSECONDS));
}

TEST(Duration, create_duration_from_millis_function_with_max_milliseconds) {
    ::testing::Test::RecordProperty("TEST_ID", "cef856b5-31dc-4de7-8421-d6f2968ffde1");
    constexpr uint64_t MAX_MILLISECONDS_FROM_SIGNED { static_cast<uint64_t>(std::numeric_limits<int64_t>::max()) };
    constexpr Duration EXPECTED_DURATION_FROM_MAX_SIGNED =
        create_duration(MAX_MILLISECONDS_FROM_SIGNED / MILLISECS_PER_SECOND,
                        (MAX_MILLISECONDS_FROM_SIGNED % MILLISECS_PER_SECOND) * NANOSECS_PER_MILLISECOND);
    constexpr uint64_t MAX_MILLISECONDS_FROM_USIGNED { std::numeric_limits<uint64_t>::max() };
    constexpr Duration EXPECTED_DURATION_FROM_MAX_UNSIGNED =
        create_duration(MAX_MILLISECONDS_FROM_USIGNED / MILLISECS_PER_SECOND,
                        (MAX_MILLISECONDS_FROM_USIGNED % MILLISECS_PER_SECOND) * NANOSECS_PER_MILLISECOND);

    auto sut1 = Duration::from_millis(std::numeric_limits<int64_t>::max());
    auto sut2 = Duration::from_millis(std::numeric_limits<uint64_t>::max());

    EXPECT_THAT(sut1, Eq(EXPECTED_DURATION_FROM_MAX_SIGNED));
    EXPECT_THAT(sut2, Eq(EXPECTED_DURATION_FROM_MAX_UNSIGNED));
}

TEST(Duration, create_duration_from_millis_function_with_negative_value_is_zero) {
    ::testing::Test::RecordProperty("TEST_ID", "8578176b-7b7d-493d-8781-5146d4b408ee");
    const auto sut = Duration::from_millis(-1);
    EXPECT_THAT(sut.as_nanos(), Eq(0U));
}

TEST(Duration, create_duration_from_micros_function_with_zero_microseconds) {
    ::testing::Test::RecordProperty("TEST_ID", "9f33fa38-9ce9-4e65-a63a-2b4353e0f346");
    auto sut1 = Duration::from_micros(0);
    auto sut2 = Duration::from_micros(0U);

    EXPECT_THAT(sut1.as_nanos(), Eq(0U));
    EXPECT_THAT(sut2.as_nanos(), Eq(0U));
}

TEST(Duration, create_duration_from_micros_function_with_multiple_microseconds) {
    ::testing::Test::RecordProperty("TEST_ID", "2f504b85-b810-4d59-b033-366a9964ba10");
    constexpr uint64_t EXPECTED_DURATION_IN_NANOSECONDS { 7U * NANOSECS_PER_MICROSECOND };
    const auto sut1 = Duration::from_micros(7);
    const auto sut2 = Duration::from_micros(7U);

    EXPECT_THAT(sut1.as_nanos(), Eq(EXPECTED_DURATION_IN_NANOSECONDS));
    EXPECT_THAT(sut2.as_nanos(), Eq(EXPECTED_DURATION_IN_NANOSECONDS));
}

TEST(Duration, create_duration_from_micros_function_with_max_microseconds) {
    ::testing::Test::RecordProperty("TEST_ID", "70a9e513-bedb-4c84-ba54-cee5c97ca049");
    constexpr uint64_t MAX_MICROSECONDS_FROM_SIGNED { static_cast<uint64_t>(std::numeric_limits<int64_t>::max()) };
    constexpr Duration EXPECTED_DURATION_FROM_MAX_SIGNED =
        create_duration(MAX_MICROSECONDS_FROM_SIGNED / MICROSECS_PER_SECONDS,
                        (MAX_MICROSECONDS_FROM_SIGNED % MICROSECS_PER_SECONDS) * NANOSECS_PER_MICROSECOND);
    constexpr uint64_t MAX_MICROSECONDS_FROM_USIGNED { std::numeric_limits<uint64_t>::max() };
    constexpr Duration EXPECTED_DURATION_FROM_MAX_UNSIGNED =
        create_duration(MAX_MICROSECONDS_FROM_USIGNED / MICROSECS_PER_SECONDS,
                        (MAX_MICROSECONDS_FROM_USIGNED % MICROSECS_PER_SECONDS) * NANOSECS_PER_MICROSECOND);

    const auto sut1 = Duration::from_micros(std::numeric_limits<int64_t>::max());
    const auto sut2 = Duration::from_micros(std::numeric_limits<uint64_t>::max());

    EXPECT_THAT(sut1, Eq(EXPECTED_DURATION_FROM_MAX_SIGNED));
    EXPECT_THAT(sut2, Eq(EXPECTED_DURATION_FROM_MAX_UNSIGNED));
}

TEST(Duration, create_duration_from_micros_function_with_negative_value_is_zero) {
    ::testing::Test::RecordProperty("TEST_ID", "9f5adb66-f70b-4cad-b7b2-2f174853a1f5");
    const auto sut = Duration::from_micros(-1);
    EXPECT_THAT(sut.as_nanos(), Eq(0U));
}

TEST(Duration, create_duration_from_nanos_function_with_zero_nanoseconds) {
    ::testing::Test::RecordProperty("TEST_ID", "b92b6fc8-8aa9-451d-94fc-b228cc19fbbe");
    const auto sut1 = Duration::from_nanos(0);
    const auto sut2 = Duration::from_nanos(0U);

    EXPECT_THAT(sut1.as_nanos(), Eq(0U));
    EXPECT_THAT(sut2.as_nanos(), Eq(0U));
}

TEST(Duration, create_duration_from_nanos_function_with_multiple_nanoseconds) {
    ::testing::Test::RecordProperty("TEST_ID", "ef647589-11f9-4dd5-8919-078a0906f36c");
    constexpr uint64_t EXPECTED_DURATION_IN_NANOSECONDS { 8U };
    const auto sut1 = Duration::from_nanos(8);
    const auto sut2 = Duration::from_nanos(8U);

    EXPECT_THAT(sut1.as_nanos(), Eq(EXPECTED_DURATION_IN_NANOSECONDS));
    EXPECT_THAT(sut2.as_nanos(), Eq(EXPECTED_DURATION_IN_NANOSECONDS));
}

TEST(Duration, create_duration_from_nanos_function) {
    ::testing::Test::RecordProperty("TEST_ID", "9217dfe1-3b73-4e61-8abe-14ecfc9fc196");
    constexpr uint64_t MAX_NANOSECONDS_FROM_SIGNED { static_cast<uint64_t>(std::numeric_limits<int64_t>::max()) };
    constexpr Duration EXPECTED_DURATION_FROM_MAX_SIGNED = create_duration(
        MAX_NANOSECONDS_FROM_SIGNED / NANOSECS_PER_SECOND, MAX_NANOSECONDS_FROM_SIGNED % NANOSECS_PER_SECOND);
    constexpr uint64_t MAX_NANOSECONDS_FROM_USIGNED { std::numeric_limits<uint64_t>::max() };
    constexpr Duration EXPECTED_DURATION_FROM_MAX_UNSIGNED = create_duration(
        MAX_NANOSECONDS_FROM_USIGNED / NANOSECS_PER_SECOND, MAX_NANOSECONDS_FROM_USIGNED % NANOSECS_PER_SECOND);

    auto sut1 = Duration::from_nanos(std::numeric_limits<int64_t>::max());
    auto sut2 = Duration::from_nanos(std::numeric_limits<uint64_t>::max());

    EXPECT_THAT(sut1, Eq(EXPECTED_DURATION_FROM_MAX_SIGNED));
    EXPECT_THAT(sut2, Eq(EXPECTED_DURATION_FROM_MAX_UNSIGNED));
}

TEST(Duration, create_duration_from_nanos_function_with_negative_value_is_zero) {
    ::testing::Test::RecordProperty("TEST_ID", "bebf67a5-1cc5-4073-85f5-95fd62fd4c15");
    auto sut = Duration::from_nanos(-1);
    EXPECT_THAT(sut.as_nanos(), Eq(0U));
}

// END CREATION FROM STATIC FUNCTION TESTS

// BEGIN CONVERSION FUNCTION TESTS

TEST(Duration, convert_days_from_zero_duration) {
    ::testing::Test::RecordProperty("TEST_ID", "9b5080da-7d39-4d17-9ad5-27b99cd940c6");
    auto sut = 0_s;
    EXPECT_THAT(sut.as_days(), Eq(0U));
}

TEST(Duration, convert_days_from_duration_less_than_one_day) {
    ::testing::Test::RecordProperty("TEST_ID", "04b6a134-0dbb-4787-935a-0774055c3402");
    const auto sut = 3473_s;
    EXPECT_THAT(sut.as_days(), Eq(0U));
}

TEST(Duration, convert_days_from_duration_more_than_one_day) {
    ::testing::Test::RecordProperty("TEST_ID", "b3159329-de5c-43b4-b515-3fff00dfbfda");
    const auto sut = 7_d + 3066_s;
    EXPECT_THAT(sut.as_days(), Eq(7U));
}

TEST(Duration, convert_days_from_max_duration) {
    ::testing::Test::RecordProperty("TEST_ID", "32d1bc55-d2c5-4d83-8386-d199dfffb66d");
    constexpr uint64_t SECONDS_PER_DAY { static_cast<uint64_t>(60U * 60U * 24U) };
    constexpr uint64_t EXPECTED_DAYS { std::numeric_limits<DurationAccessor::SecondsT>::max() / SECONDS_PER_DAY };
    const auto sut = DurationAccessor::max();
    EXPECT_THAT(sut.as_days(), Eq(EXPECTED_DAYS));
}

TEST(Duration, convert_hours_from_zero_duration) {
    ::testing::Test::RecordProperty("TEST_ID", "40c1c2dc-9700-45b8-a0d1-a7898d5a1588");
    const auto sut = 0_s;
    EXPECT_THAT(sut.as_hours(), Eq(0U));
}

TEST(Duration, convert_hours_from_duration_less_than_one_hour) {
    ::testing::Test::RecordProperty("TEST_ID", "bac778e6-03b0-4704-9bd2-40bd5f872de4");
    const auto sut = 37_m;
    EXPECT_THAT(sut.as_hours(), Eq(0U));
}

TEST(Duration, convert_hours_from_duration_more_than_one_hour) {
    ::testing::Test::RecordProperty("TEST_ID", "f52e35ce-8d3b-46db-a5ee-084c2d56c809");
    const auto sut = 73_h + 42_m;
    EXPECT_THAT(sut.as_hours(), Eq(73U));
}

TEST(Duration, convert_hours_from_max_duration) {
    ::testing::Test::RecordProperty("TEST_ID", "622e78aa-cdc6-4f8c-8de7-3f9433752893");
    constexpr uint64_t SECONDS_PER_HOUR { static_cast<uint64_t>(60U * 60U) };
    constexpr uint64_t EXPECTED_HOURS { std::numeric_limits<DurationAccessor::SecondsT>::max() / SECONDS_PER_HOUR };
    const auto sut = DurationAccessor::max();
    EXPECT_THAT(sut.as_hours(), Eq(EXPECTED_HOURS));
}

TEST(Duration, convert_minutes_from_zero_duration) {
    ::testing::Test::RecordProperty("TEST_ID", "bff59c1e-260d-4bc2-9bae-95311b5085fa");
    const auto sut = 0_s;
    EXPECT_THAT(sut.as_mins(), Eq(0U));
}

TEST(Duration, convert_minutes_from_duration_less_than_one_minute) {
    ::testing::Test::RecordProperty("TEST_ID", "19f8b9a7-45cc-4a20-884d-ae52804cd8bd");
    const auto sut = 34_s;
    EXPECT_THAT(sut.as_mins(), Eq(0U));
}

TEST(Duration, convert_minutes_from_duration_more_than_one_minute) {
    ::testing::Test::RecordProperty("TEST_ID", "9f606d01-e9f7-4ab3-a16f-ee4068a4879b");
    const auto sut = 13_m + 42_s;
    EXPECT_THAT(sut.as_mins(), Eq(13U));
}

TEST(Duration, convert_minutes_from_max_duration) {
    ::testing::Test::RecordProperty("TEST_ID", "f64e6a77-e481-4de7-bad0-0571d9aef1ad");
    constexpr uint64_t SECONDS_PER_MINUTE { 60U };
    constexpr uint64_t EXPECTED_MINUTES { std::numeric_limits<DurationAccessor::SecondsT>::max() / SECONDS_PER_MINUTE };
    const auto sut = DurationAccessor::max();
    EXPECT_THAT(sut.as_mins(), Eq(EXPECTED_MINUTES));
}

TEST(Duration, convert_seconds_from_zero_duration) {
    ::testing::Test::RecordProperty("TEST_ID", "1e892aed-b156-45ff-ac72-9b9986d9e388");
    const auto sut = 0_s;
    EXPECT_THAT(sut.as_secs(), Eq(0U));
}

TEST(Duration, convert_seconds_from_duration_less_than_one_second) {
    ::testing::Test::RecordProperty("TEST_ID", "d38be257-c6c9-476d-9c98-aec46da7db56");
    const auto sut = 737_ms;
    EXPECT_THAT(sut.as_secs(), Eq(0U));
}

TEST(Duration, convert_seconds_from_duration_more_than_one_second) {
    ::testing::Test::RecordProperty("TEST_ID", "fd393be4-4339-4138-843d-f386f93a59a2");
    const auto sut = 7_s + 833_ms;
    EXPECT_THAT(sut.as_secs(), Eq(7U));
}

TEST(Duration, convert_seconds_from_max_seconds_minus_one) {
    ::testing::Test::RecordProperty("TEST_ID", "eaa9153d-8503-4bea-8c41-6f702e49914a");
    constexpr uint64_t EXPECTED_SECONDS { std::numeric_limits<DurationAccessor::SecondsT>::max() - 1U };
    const auto sut = DurationAccessor::max() - 1_s;
    EXPECT_THAT(sut.as_secs(), Eq(EXPECTED_SECONDS));
}

TEST(Duration, convert_seconds_from_max_duration) {
    ::testing::Test::RecordProperty("TEST_ID", "3109dfa7-9d2c-4b61-bb6f-e3ad256abed6");
    constexpr uint64_t EXPECTED_SECONDS { std::numeric_limits<DurationAccessor::SecondsT>::max() };
    const auto sut = DurationAccessor::max();
    EXPECT_THAT(sut.as_secs(), Eq(EXPECTED_SECONDS));
}

TEST(Duration, convert_milliseconds_from_zero_duration) {
    ::testing::Test::RecordProperty("TEST_ID", "e56f9f62-7276-4291-a27a-52a2f4225618");
    const auto sut = 0_s;
    EXPECT_THAT(sut.as_millis(), Eq(0U));
}

TEST(Duration, convert_milliseconds_from_duration_less_than_one_millisecond) {
    ::testing::Test::RecordProperty("TEST_ID", "91b3de3d-07f1-42c0-94b6-a40279fe6ee2");
    const auto sut = 637_us;
    EXPECT_THAT(sut.as_millis(), Eq(0U));
}

TEST(Duration, convert_millieconds_from_duration_more_than_one_millisecond) {
    ::testing::Test::RecordProperty("TEST_ID", "7b5339ca-6583-4d9a-9bcd-f23a6c77021d");
    const auto sut = 55_ms + 633_us;
    EXPECT_THAT(sut.as_millis(), Eq(55U));
}

TEST(Duration, convert_milliseconds_from_duration_results_not_yet_in_saturation) {
    ::testing::Test::RecordProperty("TEST_ID", "cff7dee2-7b72-48e2-a1e6-3fb7cebe065a");
    constexpr uint64_t EXPECTED_MILLISECONDS { std::numeric_limits<uint64_t>::max() - 1U };
    const auto sut = Duration::from_millis(EXPECTED_MILLISECONDS);
    EXPECT_THAT(sut.as_millis(), Eq(EXPECTED_MILLISECONDS));
}

TEST(Duration, convert_milliseconds_from_max_duration_results_in_saturation) {
    ::testing::Test::RecordProperty("TEST_ID", "e2ca439c-f27e-4bde-9c45-1c3203106690");
    const auto sut = DurationAccessor::max();
    EXPECT_THAT(sut.as_millis(), Eq(std::numeric_limits<uint64_t>::max()));
}

TEST(Duration, convert_microseconds_from_zero_duration) {
    ::testing::Test::RecordProperty("TEST_ID", "869ab03f-0078-457b-a783-9a5eeeedaddb");
    const auto sut = 0_s;
    EXPECT_THAT(sut.as_micros(), Eq(0U));
}

TEST(Duration, convert_microseconds_from_duration_less_than_one_microsecond) {
    ::testing::Test::RecordProperty("TEST_ID", "810d52f6-3685-4ad3-a5d2-c303ca258ddc");
    const auto sut = 733_ns;
    EXPECT_THAT(sut.as_micros(), Eq(0U));
}

TEST(Duration, convert_microseconds_from_duration_more_than_one_microsecond) {
    ::testing::Test::RecordProperty("TEST_ID", "d87656f2-7f27-49b0-92ad-eb3ac00cb3bf");
    const auto sut = 555_us + 733_ns;
    EXPECT_THAT(sut.as_micros(), Eq(555U));
}

TEST(Duration, convert_microseconds_from_duration_results_not_yet_in_saturation) {
    ::testing::Test::RecordProperty("TEST_ID", "c194616a-fce7-46ef-9a43-cff4118c37c4");
    constexpr uint64_t EXPECTED_MICROSECONDS { std::numeric_limits<uint64_t>::max() - 1U };
    const auto sut = Duration::from_micros(EXPECTED_MICROSECONDS);
    EXPECT_THAT(sut.as_micros(), Eq(EXPECTED_MICROSECONDS));
}

TEST(Duration, convert_microeconds_from_max_duration_results_in_saturation) {
    ::testing::Test::RecordProperty("TEST_ID", "d98850e7-bd92-444f-8d51-3f2513a26924");
    const auto sut = DurationAccessor::max();
    EXPECT_THAT(sut.as_micros(), Eq(std::numeric_limits<uint64_t>::max()));
}

TEST(Duration, convert_nanoseconds_from_zero_duration) {
    ::testing::Test::RecordProperty("TEST_ID", "1dc6de1b-e66e-4e41-ae94-e79c3f7d0fa9");
    const auto sut = 0_s;
    EXPECT_THAT(sut.as_micros(), Eq(0U));
}

TEST(Duration, convert_nanoseconds_from_duration_of_one_nanosecond) {
    ::testing::Test::RecordProperty("TEST_ID", "f2c50d7b-a520-44ea-8fe8-2e63436175f8");
    const auto sut = 1_ns;
    EXPECT_THAT(sut.as_nanos(), Eq(1U));
}

TEST(Duration, convert_nanoseconds_from_duration_multiple_nanoseconds) {
    ::testing::Test::RecordProperty("TEST_ID", "784cb756-3a95-489b-8d2c-b8f3459ebcba");
    const auto sut = 42_ns;
    EXPECT_THAT(sut.as_nanos(), Eq(42U));
}

TEST(Duration, convert_nanoseconds_from_duration_results_not_yet_in_saturation) {
    ::testing::Test::RecordProperty("TEST_ID", "35e83bed-0625-4515-9627-b443dc4e7b98");
    constexpr uint64_t EXPECTED_NANOSECONDS { std::numeric_limits<uint64_t>::max() - 1U };
    const auto sut = Duration::from_nanos(EXPECTED_NANOSECONDS);
    EXPECT_THAT(sut.as_nanos(), Eq(EXPECTED_NANOSECONDS));
}

TEST(Duration, convert_nanoeconds_from_max_duration_results_in_saturation) {
    ::testing::Test::RecordProperty("TEST_ID", "c1376842-8eb7-4878-a60d-19460033f48c");
    const auto sut = DurationAccessor::max();
    EXPECT_THAT(sut.as_nanos(), Eq(std::numeric_limits<uint64_t>::max()));
}

// END CONVERSION FUNCTION TESTS

// BEGIN SUBSECONDS ACCESS TESTS

TEST(Duration, subsec_millis_from_zero_duration) {
    ::testing::Test::RecordProperty("TEST_ID", "b5ab2d0d-de0c-4bf0-b814-1fac235fac46");
    const auto sut = 0_s;
    EXPECT_THAT(sut.subsec_millis(), Eq(0U));
}

TEST(Duration, subsec_millis_from_duration_less_than_one_millisecond) {
    ::testing::Test::RecordProperty("TEST_ID", "edeffc53-7188-4431-a58f-f8c9099d81a4");
    const auto sut = 373_us;
    EXPECT_THAT(sut.subsec_millis(), Eq(0U));
}

TEST(Duration, subsec_millis_from_duration_more_than_one_millisecond) {
    ::testing::Test::RecordProperty("TEST_ID", "bd7283fe-169d-4487-b78b-f53aab39e7a5");
    const auto sut = 444_ms + 337_us;
    EXPECT_THAT(sut.subsec_millis(), Eq(444U));
}

TEST(Duration, subsec_millis_from_duration_more_than_one_second) {
    ::testing::Test::RecordProperty("TEST_ID", "af6bb389-46ed-4dcb-91c1-50c5f68933b6");
    const auto sut = 13_s + 6_ms + 13_us + 42_ns;
    EXPECT_THAT(sut.subsec_millis(), Eq(6U));
}

TEST(Duration, subsec_micros_from_zero_duration) {
    ::testing::Test::RecordProperty("TEST_ID", "48c99ad7-0327-49d2-9ed9-f5afcf501719");
    const auto sut = 0_s;
    EXPECT_THAT(sut.subsec_micros(), Eq(0U));
}

TEST(Duration, subsec_micros_from_duration_less_than_one_microsecond) {
    ::testing::Test::RecordProperty("TEST_ID", "eaf7535a-ab4a-4d3d-a7d7-10d4207a4355");
    const auto sut = 733_ns;
    EXPECT_THAT(sut.subsec_micros(), Eq(0U));
}

TEST(Duration, subsec_micros_from_duration_more_than_one_microsecond) {
    ::testing::Test::RecordProperty("TEST_ID", "1f165cc7-df15-4ad2-a1e3-eabdba3a4e69");
    const auto sut = 555_us + 733_ns;
    EXPECT_THAT(sut.subsec_micros(), Eq(555U));
}

TEST(Duration, subsec_micros_from_duration_more_than_one_second) {
    ::testing::Test::RecordProperty("TEST_ID", "2df33200-88a3-444e-b11a-a58317ac0818");
    const auto sut = 13_s + 4_ms + 123_us + 43_ns;
    EXPECT_THAT(sut.subsec_micros(), Eq(4123U));
}

TEST(Duration, subsec_nanos_from_zero_duration) {
    ::testing::Test::RecordProperty("TEST_ID", "ede2a5ac-3064-499d-8fc5-d21a852d6d9c");
    const auto sut = 0_s;
    EXPECT_THAT(sut.subsec_nanos(), Eq(0U));
}

TEST(Duration, subsec_nanos_from_duration_more_than_zero_nonoseconds) {
    ::testing::Test::RecordProperty("TEST_ID", "9f15755f-ab9c-4472-9c80-74ba9e760239");
    const auto sut = 373_ns;
    EXPECT_THAT(sut.subsec_nanos(), Eq(373U));
}

TEST(Duration, subsec_nanos_from_duration_more_than_one_microsecond) {
    ::testing::Test::RecordProperty("TEST_ID", "e8298cdb-e9e0-4ca0-b506-416a5bf99ea0");
    const auto sut = 42_ms + 666_us + 337_ns;
    EXPECT_THAT(sut.subsec_nanos(), Eq(42666337U));
}

TEST(Duration, subsec_nanos_from_duration_more_than_one_second) {
    ::testing::Test::RecordProperty("TEST_ID", "f39764e1-4a05-4968-b0c4-674b4a5b5b64");
    const auto sut = 13_s + 9_ms + 321_us + 73_ns;
    EXPECT_THAT(sut.subsec_nanos(), Eq(9321073U));
}

// END SUBSECONDS ACCESS TESTS

// BEGIN COMPARISON TESTS

TEST(Duration, compare_two_equal_durations_for_equality) {
    ::testing::Test::RecordProperty("TEST_ID", "3b233585-559d-4baf-a8ac-6a291641c4b1");
    const auto time1 = 200_us;
    const auto time2 = 200000_ns;
    EXPECT_TRUE(time1 == time2);
}

TEST(Duration, compare_two_non_equal_durations_for_equality) {
    ::testing::Test::RecordProperty("TEST_ID", "cb531d5b-bce9-4cf1-bfb9-8e250b2e0068");
    const auto time1 = 1_s + 200_us;
    const auto time2 = 1_s + 1_ns;
    const auto time3 = 1_ns;
    EXPECT_FALSE(time1 == time2);
    EXPECT_FALSE(time2 == time1);
    EXPECT_FALSE(time2 == time3);
    EXPECT_FALSE(time3 == time2);
}

TEST(Duration, compare_two_non_equal_durations_for_inequality) {
    ::testing::Test::RecordProperty("TEST_ID", "49364301-ec17-4f27-8f1b-99da5d8f269e");
    const auto time1 = 1_s + 200_us;
    const auto time2 = 1_ns;
    EXPECT_TRUE(time1 != time2);
    EXPECT_TRUE(time2 != time1);
}

TEST(Duration, compare_two_equal_durations_for_inequality) {
    ::testing::Test::RecordProperty("TEST_ID", "a434aaa6-7549-405b-90bf-b46526c37c2e");
    const auto time1 = 200_us;
    const auto time2 = 200000_ns;
    EXPECT_FALSE(time1 != time2);
}

TEST(Duration, compare_two_equal_durations_are_not_less_than) {
    ::testing::Test::RecordProperty("TEST_ID", "29257e38-cdcb-45ae-b5d1-b1d2cbf9cf46");
    const auto time1 = 1_s + 200_us;
    const auto time2 = 1_s + 200_us;
    EXPECT_FALSE(time1 < time2);
}

TEST(Duration, compare_two_equal_durations_are_not_greater_than) {
    ::testing::Test::RecordProperty("TEST_ID", "b541d47f-58f5-46b6-af22-dc393c8b366d");
    const auto time1 = 1_s + 200_us;
    const auto time2 = 1_s + 200_us;
    EXPECT_FALSE(time1 > time2);
}

TEST(Duration, compare_two_equal_durations_are_less_than_or_equal_to) {
    ::testing::Test::RecordProperty("TEST_ID", "6090f9f9-9caf-4a12-9e25-ea8c71d6fe78");
    const auto time1 = 1_s + 200_us;
    const auto time2 = 1_s + 200_us;
    EXPECT_TRUE(time1 <= time2);
}

TEST(Duration, compare_two_equal_durations_are_greater_than_or_equal_to) {
    ::testing::Test::RecordProperty("TEST_ID", "653aabbc-06ae-4fd5-912d-b270d888fab1");
    const auto time1 = 1_s + 200_us;
    const auto time2 = 1_s + 200_us;
    EXPECT_TRUE(time1 >= time2);
}

TEST(Duration, compare_duration_is_less_than_other) {
    ::testing::Test::RecordProperty("TEST_ID", "41f62afb-da29-46dc-93fa-2ad62a8a9b4a");
    const auto time1 = 100_us;
    const auto time2 = 400_us;
    const auto time3 = 1_s + 200_us;
    const auto time4 = 1_s + 300_us;
    EXPECT_TRUE(time1 < time2);
    EXPECT_TRUE(time1 < time3);
    EXPECT_TRUE(time2 < time3);
    EXPECT_TRUE(time3 < time4);
}

TEST(Duration, compare_duration_is_not_less_than_other) {
    ::testing::Test::RecordProperty("TEST_ID", "6399a6e1-523b-4ef1-864f-3a01d06214e1");
    const auto time1 = 100_us;
    const auto time2 = 400_us;
    const auto time3 = 1_s + 200_us;
    const auto time4 = 1_s + 300_us;
    EXPECT_FALSE(time2 < time1);
    EXPECT_FALSE(time3 < time1);
    EXPECT_FALSE(time3 < time2);
    EXPECT_FALSE(time4 < time3);
}

TEST(Duration, compare_duration_is_less_than_or_equal_to_other) {
    ::testing::Test::RecordProperty("TEST_ID", "1f958f6a-5092-4592-a07f-bd358180006e");
    const auto time1 = 100_us;
    const auto time2 = 400_us;
    const auto time3 = 1_s + 200_us;
    const auto time4 = 1_s + 300_us;
    EXPECT_TRUE(time1 <= time2);
    EXPECT_TRUE(time1 <= time3);
    EXPECT_TRUE(time2 <= time3);
    EXPECT_TRUE(time3 <= time4);
}

TEST(Duration, compare_duration_is_not_less_than_or_equal_to_other) {
    ::testing::Test::RecordProperty("TEST_ID", "2a31439d-0fe8-4883-aa61-0b74c1706481");
    const auto time1 = 100_us;
    const auto time2 = 400_us;
    const auto time3 = 1_s + 200_us;
    const auto time4 = 1_s + 300_us;
    EXPECT_FALSE(time2 <= time1);
    EXPECT_FALSE(time3 <= time1);
    EXPECT_FALSE(time3 <= time2);
    EXPECT_FALSE(time4 <= time3);
}

TEST(Duration, compare_duration_is_greater_than_other) {
    ::testing::Test::RecordProperty("TEST_ID", "24719355-ccee-4307-b9fd-932ab5ee4b62");
    const auto time1 = 1_s + 300_us;
    const auto time2 = 1_s + 200_us;
    const auto time3 = 400_us;
    const auto time4 = 100_us;
    EXPECT_TRUE(time1 > time2);
    EXPECT_TRUE(time1 > time3);
    EXPECT_TRUE(time2 > time3);
    EXPECT_TRUE(time3 > time4);
}

TEST(Duration, compare_duration_is_not_greater_than_other) {
    ::testing::Test::RecordProperty("TEST_ID", "e38e9982-40ca-4ec6-8097-57e45bb618f8");
    const auto time1 = 1_s + 300_us;
    const auto time2 = 1_s + 200_us;
    const auto time3 = 400_us;
    const auto time4 = 100_us;
    EXPECT_FALSE(time2 > time1);
    EXPECT_FALSE(time3 > time1);
    EXPECT_FALSE(time3 > time2);
    EXPECT_FALSE(time4 > time3);
}

TEST(Duration, compare_duration_is_greater_than_or_equal_to_other) {
    ::testing::Test::RecordProperty("TEST_ID", "6d9d9976-e438-4c23-8056-7e6476dd0330");
    const auto time1 = 1_s + 300_us;
    const auto time2 = 1_s + 200_us;
    const auto time3 = 400_us;
    const auto time4 = 100_us;
    EXPECT_TRUE(time1 >= time2);
    EXPECT_TRUE(time1 >= time3);
    EXPECT_TRUE(time2 >= time3);
    EXPECT_TRUE(time3 >= time4);
}

TEST(Duration, compare_duration_is_not_greater_than_or_equal_to_other) {
    ::testing::Test::RecordProperty("TEST_ID", "cf28e711-5fec-4176-8209-7c23acf848f3");
    const auto time1 = 1_s + 300_us;
    const auto time2 = 1_s + 200_us;
    const auto time3 = 400_us;
    const auto time4 = 100_us;
    EXPECT_FALSE(time2 >= time1);
    EXPECT_FALSE(time3 >= time1);
    EXPECT_FALSE(time3 >= time2);
    EXPECT_FALSE(time4 >= time3);
}

// END COMPARISON TESTS

// BEGIN ARITHMETIC TESTS

TEST(Duration, add_duration_does_not_change_original_object) {
    ::testing::Test::RecordProperty("TEST_ID", "d146983f-6d39-420c-9398-e80fa6880887");
    constexpr Duration EXPECTED_DURATION { 13_s + 42_ns };

    auto sut1 = EXPECTED_DURATION;
    const auto result1 IOX2_MAYBE_UNUSED = sut1 + 15_s;

    auto sut2 = EXPECTED_DURATION;
    const auto result2 IOX2_MAYBE_UNUSED = 15_s + sut1;

    EXPECT_THAT(sut1, Eq(EXPECTED_DURATION));
    EXPECT_THAT(sut2, Eq(EXPECTED_DURATION));
}

TEST(Duration, add_duration_with_two_zero_durations_results_in_zero_duration) {
    ::testing::Test::RecordProperty("TEST_ID", "64aa64f9-0105-43a1-bdcb-7b62f12fd86b");
    constexpr Duration EXPECTED_DURATION { 0_s };
    auto duration1 = 0_s;
    auto duration2 = 0_s;

    auto sut = duration1 + duration2;

    EXPECT_THAT(sut, Eq(EXPECTED_DURATION));
}

TEST(Duration, add_duration_with_one_zero_durations_results_in_none_zero_duration) {
    ::testing::Test::RecordProperty("TEST_ID", "32f4f923-057c-4481-b50e-78a8fd4bceed");
    constexpr Duration EXPECTED_DURATION { 10_ns };
    const auto duration1 = 0_s;
    const auto duration2 = 10_ns;

    auto sut1 = duration1 + duration2;
    auto sut2 = duration2 + duration1;

    EXPECT_THAT(sut1, Eq(EXPECTED_DURATION));
    EXPECT_THAT(sut2, Eq(EXPECTED_DURATION));
}

TEST(Duration, add_duration_with_sum_of_durations_less_than_one_seconds_results_in_less_than_one_second) {
    ::testing::Test::RecordProperty("TEST_ID", "b3704e3c-588a-4c1f-a09b-9244c210d853");
    constexpr Duration EXPECTED_DURATION = create_duration(0U, (100 * NANOSECS_PER_MICROSECOND) + 10U);
    const auto duration1 = 100_us;
    const auto duration2 = 10_ns;

    auto sut1 = duration1 + duration2;
    auto sut2 = duration2 + duration1;

    EXPECT_THAT(sut1, Eq(EXPECTED_DURATION));
    EXPECT_THAT(sut2, Eq(EXPECTED_DURATION));
}

TEST(Duration, add_duration_with_sum_of_durations_more_than_one_seconds_results_in_more_than_one_second) {
    ::testing::Test::RecordProperty("TEST_ID", "62ad6686-990f-4c3e-b1ff-d61238a2d8c2");
    constexpr Duration EXPECTED_DURATION = create_duration(1U, 700 * NANOSECS_PER_MILLISECOND);
    const auto duration1 = 800_ms;
    const auto duration2 = 900_ms;

    auto sut1 = duration1 + duration2;
    auto sut2 = duration2 + duration1;

    EXPECT_THAT(sut1, Eq(EXPECTED_DURATION));
    EXPECT_THAT(sut2, Eq(EXPECTED_DURATION));
}

TEST(Duration, add_duration_with_one_duration_more_than_one_seconds_results_in_more_than_one_second) {
    ::testing::Test::RecordProperty("TEST_ID", "435175e4-33d1-4829-a526-4b01268233fc");
    constexpr Duration EXPECTED_DURATION = create_duration(2U, 700 * NANOSECS_PER_MILLISECOND);
    const auto duration1 = create_duration(1U, 800 * NANOSECS_PER_MILLISECOND);
    const auto duration2 = 900_ms;

    auto sut1 = duration1 + duration2;
    auto sut2 = duration2 + duration1;

    EXPECT_THAT(sut1, Eq(EXPECTED_DURATION));
    EXPECT_THAT(sut2, Eq(EXPECTED_DURATION));
}

TEST(Duration, add_duration_with_durations_more_than_one_seconds_results_in_more_than_one_second) {
    ::testing::Test::RecordProperty("TEST_ID", "a509667d-507a-4521-abae-f24879367a0a");
    constexpr Duration EXPECTED_DURATION = create_duration(3U, 700 * NANOSECS_PER_MILLISECOND);
    const auto duration1 = create_duration(1U, 800 * NANOSECS_PER_MILLISECOND);
    const auto duration2 = create_duration(1U, 900 * NANOSECS_PER_MILLISECOND);

    auto sut1 = duration1 + duration2;
    auto sut2 = duration2 + duration1;

    EXPECT_THAT(sut1, Eq(EXPECTED_DURATION));
    EXPECT_THAT(sut2, Eq(EXPECTED_DURATION));
}

TEST(Duration, add_duration_results_not_yet_in_saturation) {
    ::testing::Test::RecordProperty("TEST_ID", "289d9f13-01e0-4635-875b-5b239dc6f102");
    constexpr Duration EXPECTED_DURATION =
        create_duration(std::numeric_limits<DurationAccessor::SecondsT>::max(), NANOSECS_PER_SECOND - 2U);
    auto duration1 =
        create_duration(std::numeric_limits<DurationAccessor::SecondsT>::max() - 1U, NANOSECS_PER_SECOND - 1U);
    auto duration2 = create_duration(0U, NANOSECS_PER_SECOND - 1U);

    auto sut1 = duration1 + duration2;
    auto sut2 = duration2 + duration1;

    EXPECT_THAT(sut1, Eq(EXPECTED_DURATION));
    EXPECT_THAT(sut2, Eq(EXPECTED_DURATION));
}

TEST(Duration, add_duration_results_in_saturation_from_nanoseconds) {
    ::testing::Test::RecordProperty("TEST_ID", "179b2630-1c16-4955-95e2-8dbd69429162");
    auto duration1 = create_duration(std::numeric_limits<DurationAccessor::SecondsT>::max(), NANOSECS_PER_SECOND - 2U);
    auto duration2 = create_duration(0U, 2U);

    auto sut1 = duration1 + duration2;
    auto sut2 = duration2 + duration1;

    EXPECT_THAT(sut1, Eq(DurationAccessor::max()));
    EXPECT_THAT(sut2, Eq(DurationAccessor::max()));
}

TEST(Duration, add_duration_results_in_saturation_from_seconds) {
    ::testing::Test::RecordProperty("TEST_ID", "11766b56-cf0e-4fbe-a8a2-a167b6e4a36a");
    auto duration1 =
        create_duration(std::numeric_limits<DurationAccessor::SecondsT>::max() - 1U, NANOSECS_PER_SECOND - 1U);
    auto duration2 = create_duration(2U, 0U);

    auto sut1 = duration1 + duration2;
    auto sut2 = duration2 + duration1;

    EXPECT_THAT(sut1, Eq(DurationAccessor::max()));
    EXPECT_THAT(sut2, Eq(DurationAccessor::max()));
}

TEST(Duration, add_assign_seconds_to_duration_results_in_seconds_addition_to_lhs) {
    ::testing::Test::RecordProperty("TEST_ID", "c223f8b3-d396-4c45-8c2d-b6b7d7b7f57a");

    constexpr Duration EXPECTED_DURATION = create_duration(3U, 0U);
    auto sut = create_duration(2U, 0U);
    auto otherDuration = create_duration(1U, 0U); // NOLINT

    sut += otherDuration;

    EXPECT_EQ(sut, EXPECTED_DURATION);
}

TEST(Duration, add_assign_nanoseconds_to_duration_results_in_nanoseconds_addition_to_lhs) {
    ::testing::Test::RecordProperty("TEST_ID", "a0cd4bcf-59f5-4d84-88e6-0d11d34142a1");

    constexpr Duration EXPECTED_DURATION = create_duration(0U, 100U);
    auto sut = create_duration(0U, 50U);           // NOLINT
    auto otherDuration = create_duration(0U, 50U); // NOLINT

    sut += otherDuration;

    EXPECT_THAT(sut, Eq(EXPECTED_DURATION));
}

TEST(Duration, add_assign_duration_past_nanosecond_boundary_results_in_second_increment_to_lhs) {
    ::testing::Test::RecordProperty("TEST_ID", "97ec3b19-31c9-41bb-8a1f-442cfdf5ed66");

    constexpr Duration EXPECTED_DURATION = create_duration(1U, 5U);
    auto sut = create_duration(0U, NANOSECS_PER_SECOND - 5); // NOLINT
    auto otherDuration = create_duration(0U, 10U);           // NOLINT

    sut += otherDuration;

    EXPECT_THAT(sut, Eq(EXPECTED_DURATION));
}


TEST(Duration, add_assign_duration_results_in_saturation_from_seconds) {
    ::testing::Test::RecordProperty("TEST_ID", "940e6a59-9b1c-4928-8fe1-af290beebb5d");
    auto sut = create_duration(std::numeric_limits<DurationAccessor::SecondsT>::max() - 1U, NANOSECS_PER_SECOND - 1U);
    auto otherDuration = create_duration(2U, 0U); // NOLINT

    sut += otherDuration;

    EXPECT_THAT(sut, Eq(DurationAccessor::max()));
}

TEST(Duration, add_assign_duration_results_in_saturation_from_nanoseconds) {
    ::testing::Test::RecordProperty("TEST_ID", "c8d972a8-cbb1-41e8-bf72-8d05963f95f7");
    auto sut = create_duration(std::numeric_limits<DurationAccessor::SecondsT>::max(), NANOSECS_PER_SECOND - 2U);
    auto other_duration = create_duration(0U, 2U);

    sut += other_duration;

    EXPECT_THAT(sut, Eq(DurationAccessor::max()));
}

TEST(Duration, subtract_duration_does_not_change_original_object) {
    ::testing::Test::RecordProperty("TEST_ID", "c61d7d6e-4164-4c00-b264-7ed62ad22748");
    constexpr Duration EXPECTED_DURATION { 13_s + 42_ns };

    auto sut1 = EXPECTED_DURATION;
    const auto result1 IOX2_MAYBE_UNUSED = sut1 - 5_s;

    auto sut2 = EXPECTED_DURATION;
    const auto result2 IOX2_MAYBE_UNUSED = 35_s + sut1;

    EXPECT_THAT(sut1, Eq(EXPECTED_DURATION));
    EXPECT_THAT(sut2, Eq(EXPECTED_DURATION));
}

TEST(Duration, subtract_duration_with_two_zero_durations_results_in_zero_duration) {
    ::testing::Test::RecordProperty("TEST_ID", "e1bc9ccf-2702-498a-b069-750f49022e17");
    constexpr Duration EXPECTED_DURATION { 0_s };
    auto duration1 = 0_s;
    auto duration2 = 0_s;

    auto sut = duration1 - duration2;

    EXPECT_THAT(sut, Eq(EXPECTED_DURATION));
}

TEST(Duration, subtract_duration_with_durations_with_same_value_results_in_zero_duration) {
    ::testing::Test::RecordProperty("TEST_ID", "20aa9e6a-880d-4839-b31d-77efcc3daa9d");
    constexpr Duration EXPECTED_DURATION { 0_s };
    const auto duration1 = create_duration(10U, 123U);
    const auto duration2 = create_duration(10U, 123U);

    auto sut = duration1 - duration2;

    EXPECT_THAT(sut, Eq(EXPECTED_DURATION));
}

TEST(Duration, subtract_duration_from_zero_durations_results_in_zero_duration) {
    ::testing::Test::RecordProperty("TEST_ID", "17f83a4a-a1b7-4f34-9eb8-c1ffaf40ffde");
    constexpr Duration EXPECTED_DURATION { 0_s };
    const auto duration0 = 0_s;
    const auto duration1 = 10_ns;
    const auto duration2 = 10_s;

    auto sut1 = duration0 - duration1;
    auto sut2 = duration0 - duration2;

    EXPECT_THAT(sut1, Eq(EXPECTED_DURATION));
    EXPECT_THAT(sut2, Eq(EXPECTED_DURATION));
}

TEST(Duration, subtract_duration_with_larger_durations_results_in_zero_duration_from_nanoseconds) {
    ::testing::Test::RecordProperty("TEST_ID", "6c60f928-50d1-4309-b01c-ffda9ebd5594");
    constexpr Duration EXPECTED_DURATION { 0_s };
    const auto duration1 = 10_ns;
    const auto duration2 = 110_ns;

    auto sut = duration1 - duration2;

    EXPECT_THAT(sut, Eq(EXPECTED_DURATION));
}

TEST(Duration, subtract_duration_with_larger_durations_results_in_zero_duration_from_seconds) {
    ::testing::Test::RecordProperty("TEST_ID", "4b0eded3-616e-42b2-9143-0407b5687728");
    constexpr Duration EXPECTED_DURATION { 0_s };
    const auto duration1 = create_duration(10U, 123U);
    const auto duration2 = create_duration(100U, 123U);

    auto sut = duration1 - duration2;

    EXPECT_THAT(sut, Eq(EXPECTED_DURATION));
}

TEST(Duration, subtract_duration_with_zero_durations_results_in_origina_duration) {
    ::testing::Test::RecordProperty("TEST_ID", "c661946d-ad4f-4a64-bdc2-b7e5b98be1fd");
    constexpr Duration EXPECTED_DURATION = create_duration(10U, 42U);
    auto duration1 = EXPECTED_DURATION;
    auto duration2 = 0_s;

    auto sut = duration1 + duration2;

    EXPECT_THAT(sut, Eq(EXPECTED_DURATION));
}

TEST(Duration, subtract_duration_more_than_one_second_with_less_than_one_second_results_in_more_than_one_second) {
    ::testing::Test::RecordProperty("TEST_ID", "83838524-402c-46b5-a1ee-2dd09c950148");
    constexpr Duration EXPECTED_DURATION = create_duration(1U, 36U);
    const auto duration1 = create_duration(1U, 73U);
    const auto duration2 = create_duration(0U, 37U);

    auto sut = duration1 - duration2;

    EXPECT_THAT(sut, Eq(EXPECTED_DURATION));
}

TEST(Duration, subtract_duration_more_than_one_second_with_less_than_one_second_results_in_less_than_one_second) {
    ::testing::Test::RecordProperty("TEST_ID", "51df6a6c-7c22-4b9e-b406-229a8ddbf49d");
    constexpr Duration EXPECTED_DURATION = create_duration(0U, NANOSECS_PER_SECOND - 36U);
    const auto duration1 = create_duration(1U, 37U);
    const auto duration2 = create_duration(0U, 73U);

    auto sut = duration1 - duration2;

    EXPECT_THAT(sut, Eq(EXPECTED_DURATION));
}

TEST(Duration, subtract_duration_more_than_one_second_with_more_than_one_second_results_in_less_than_one_second) {
    ::testing::Test::RecordProperty("TEST_ID", "3416550c-2cb9-4cbc-9746-6eedab32cca3");
    constexpr Duration EXPECTED_DURATION = create_duration(0U, 36U);
    const auto duration1 = create_duration(1U, 73U);
    const auto duration2 = create_duration(1U, 37U);

    auto sut = duration1 - duration2;

    EXPECT_THAT(sut, Eq(EXPECTED_DURATION));
}

TEST(Duration, subtract_duration_with_seconds_and_nanoseconds_causing_reduction_of_seconds) {
    ::testing::Test::RecordProperty("TEST_ID", "fb9ca012-f5cc-4ab1-a3be-a63d48f1319f");
    constexpr Duration EXPECTED_DURATION = create_duration(0U, NANOSECS_PER_SECOND - 36U);
    const auto duration1 = create_duration(2U, 37U);
    const auto duration2 = create_duration(1U, 73U);

    auto sut = duration1 - duration2;

    EXPECT_THAT(sut, Eq(EXPECTED_DURATION));
}

TEST(Duration, subtract_assign_seconds_from_duration_results_in_second_subtractraction_to_lhs) {
    ::testing::Test::RecordProperty("TEST_ID", "ab917854-0a97-4529-b408-c51c30b9375f");

    constexpr Duration EXPECTED_DURATION = create_duration(1U, 0U);
    auto sut = create_duration(2U, 0U);
    auto other_duration = create_duration(1U, 0U);

    sut -= other_duration;

    EXPECT_THAT(sut, Eq(EXPECTED_DURATION));
}

TEST(Duration, subtract_assign_nanoseconds_from_duration_results_in_nanosecond_subtractraction_to_lhs) {
    ::testing::Test::RecordProperty("TEST_ID", "b6051fe8-37d5-4225-b1ea-5038b56889ce");

    constexpr Duration EXPECTED_DURATION = create_duration(0U, 50U);
    auto sut = create_duration(0U, 100U);          // NOLINT
    auto otherDuration = create_duration(0U, 50U); // NOLINT

    sut -= otherDuration;

    EXPECT_THAT(sut, Eq(EXPECTED_DURATION));
}

TEST(Duration, subtract_assign_duration_past_zero_nanoseconds_results_in_decremented_seconds) {
    ::testing::Test::RecordProperty("TEST_ID", "cba9f397-9f25-4e88-9608-0d51eb1e4ccc");

    constexpr Duration EXPECTED_DURATION = create_duration(0U, NANOSECS_PER_SECOND - 2);
    auto sut = create_duration(1U, 0U);
    auto other_duration = create_duration(0U, 2U);

    sut -= other_duration;

    EXPECT_THAT(sut, Eq(EXPECTED_DURATION));
}

TEST(Duration, subtract_assign_larger_duration_results_in_zero) {
    ::testing::Test::RecordProperty("TEST_ID", "eb2bace9-751c-48bd-82eb-2e5679512503");

    constexpr Duration EXPECTED_DURATION = create_duration(0U, 0U);
    auto sut = create_duration(1U, 0U);
    auto other_duration = create_duration(2U, 0U);

    sut -= other_duration;

    EXPECT_THAT(sut, Eq(EXPECTED_DURATION));
}


TEST(Duration, multiply_duration_does_not_change_original_object) {
    ::testing::Test::RecordProperty("TEST_ID", "c243775f-7a9b-42bf-8bd7-78bc843f0953");
    constexpr Duration EXPECTED_DURATION { 13_s + 42_ns };

    auto sut1 = EXPECTED_DURATION;
    auto result1 IOX2_MAYBE_UNUSED = sut1 * 0;

    auto sut2 = EXPECTED_DURATION;
    auto result2 IOX2_MAYBE_UNUSED = sut2 * 0;

    EXPECT_THAT(sut1, Eq(EXPECTED_DURATION));
    EXPECT_THAT(sut2, Eq(EXPECTED_DURATION));
}

template <typename T>
void multiply(Duration duration, T multiplicator, const Duration expected_duration) {
    auto sut1 = duration * multiplicator;
    auto sut2 = multiplicator * duration;

    EXPECT_THAT(sut1, Eq(expected_duration));
    EXPECT_THAT(sut2, Eq(expected_duration));
}

TEST(Duration, multiply_zero_duration_with_zero_signed_multiplicator_results_in_zero_duration) {
    ::testing::Test::RecordProperty("TEST_ID", "95c85fb3-c8a8-4170-9c7a-28f7dc9b8523");
    constexpr Duration EXPECTED_DURATION { 0_s };
    auto duration = 0_s;

    multiply(duration, 0, EXPECTED_DURATION);
}

TEST(Duration, multiply_zero_duration_with_zero_unsigned_multiplicator_results_in_zero_duration) {
    ::testing::Test::RecordProperty("TEST_ID", "b9568756-0c13-49aa-883a-94f4720fb945");
    constexpr Duration EXPECTED_DURATION { 0_s };
    auto duration = 0_s;

    multiply(duration, 0U, EXPECTED_DURATION);
}

TEST(Duration, multiply_zero_duration_with_zero_float_multiplicator_results_in_zero_duration) {
    ::testing::Test::RecordProperty("TEST_ID", "5090f23c-eb59-4e6d-9399-f62940af5e50");
    constexpr Duration EXPECTED_DURATION { 0_s };
    auto duration = 0_s;

    multiply(duration, 0.0, EXPECTED_DURATION);
}

TEST(Duration, multiply_duration_with_zero_signed_multiplicator_results_in_zero_duration) {
    ::testing::Test::RecordProperty("TEST_ID", "88fc2bf9-6748-4bbd-aa9e-0d7e73afbeff");
    constexpr Duration EXPECTED_DURATION { 0_s };
    const auto duration = 1_s + 12_ns;

    multiply(duration, 0, EXPECTED_DURATION);
}

TEST(Duration, multiply_duration_with_zero_unsigned_multiplicator_results_in_zero_duration) {
    ::testing::Test::RecordProperty("TEST_ID", "5d224e46-3055-4b88-8f65-26850fb3ec53");
    constexpr Duration EXPECTED_DURATION { 0_s };
    const auto duration = 1_s + 12_ns;

    multiply(duration, 0U, EXPECTED_DURATION);
}

TEST(Duration, multiply_duration_with_zero_float_multiplicator_results_in_zero_duration) {
    ::testing::Test::RecordProperty("TEST_ID", "b867e055-64f2-44fb-a85d-69c11af75c69");
    constexpr Duration EXPECTED_DURATION { 0_s };
    const auto duration = 1_s + 12_ns;

    multiply(duration, 0.0, EXPECTED_DURATION);
}

TEST(Duration, multiply_duration_less_than_one_second_with_signed_results_in_less_than_one_second) {
    ::testing::Test::RecordProperty("TEST_ID", "73ecd04c-1054-4aed-b4d6-bf782f7f9cb5");
    constexpr int64_t MULTIPLICATOR { 3 };
    constexpr Duration EXPECTED_DURATION { 36_ns };
    const auto duration = 12_ns;

    multiply(duration, MULTIPLICATOR, EXPECTED_DURATION);
}

TEST(Duration, multiply_duration_less_than_one_second_with_unsigned_results_in_less_than_one_second) {
    ::testing::Test::RecordProperty("TEST_ID", "54c235b2-cc18-4007-be85-32847d94fe54");
    constexpr uint64_t MULTIPLICATOR { 3U };
    constexpr Duration EXPECTED_DURATION { 36_ns };
    const auto duration = 12_ns;

    multiply(duration, MULTIPLICATOR, EXPECTED_DURATION);
}

TEST(Duration, multiply_duration_less_than_one_second_with_float_results_in_less_than_one_second) {
    ::testing::Test::RecordProperty("TEST_ID", "6193219f-06cb-4f63-b223-d06157eaf559");
    constexpr float MULTIPLICATOR { 3.5 };
    constexpr Duration EXPECTED_DURATION { 42_ns };
    const auto duration = 12_ns;

    multiply(duration, MULTIPLICATOR, EXPECTED_DURATION);
}

TEST(Duration, multiply_duration_less_than_one_second_with_signed_results_in_more_than_one_second) {
    ::testing::Test::RecordProperty("TEST_ID", "29c9f01c-7a4f-462f-8293-fc08c7e27f64");
    constexpr int64_t MULTIPLICATOR { 3 };
    constexpr Duration EXPECTED_DURATION { 1_s + 800_ms };
    const auto duration = 600_ms;

    multiply(duration, MULTIPLICATOR, EXPECTED_DURATION);
}

TEST(Duration, multiply_duration_less_than_one_second_with_unsigned_results_in_more_than_one_second) {
    ::testing::Test::RecordProperty("TEST_ID", "6fdb2c21-fc0c-43b2-8120-b459e6e9f571");
    constexpr uint64_t MULTIPLICATOR { 3U };
    constexpr Duration EXPECTED_DURATION { 1_s + 800_ms };
    const auto duration = 600_ms;

    multiply(duration, MULTIPLICATOR, EXPECTED_DURATION);
}

TEST(Duration, multiply_duration_less_than_one_second_with_float_results_in_more_than_one_second) {
    ::testing::Test::RecordProperty("TEST_ID", "39759894-77e0-4b0d-9279-d787367a76ab");
    constexpr float MULTIPLICATOR { 3.5 };
    constexpr Duration EXPECTED_DURATION { 2_s + 100_ms };
    const auto duration = 600_ms;

    multiply(duration, MULTIPLICATOR, EXPECTED_DURATION);
}

TEST(Duration, multiply_duration_more_than_one_second_with_signed_results_in_more_than_one_second) {
    ::testing::Test::RecordProperty("TEST_ID", "6467c466-1720-4336-acec-06bd11a43efc");
    constexpr int64_t MULTIPLICATOR { 3 };
    constexpr Duration EXPECTED_DURATION { 13_s + 800_ms };
    const auto duration = 4_s + 600_ms;

    multiply(duration, MULTIPLICATOR, EXPECTED_DURATION);
}

TEST(Duration, multiply_duration_more_than_one_second_with_unsigned_results_in_more_than_one_second) {
    ::testing::Test::RecordProperty("TEST_ID", "9db6e3ec-bf24-4d31-a88a-18077d1b1674");
    constexpr uint64_t MULTIPLICATOR { 3U };
    constexpr Duration EXPECTED_DURATION { 13_s + 800_ms };
    const auto duration = 4_s + 600_ms;

    multiply(duration, MULTIPLICATOR, EXPECTED_DURATION);
}

TEST(Duration, multiply_duration_more_than_one_second_with_float_results_in_more_than_one_second) {
    ::testing::Test::RecordProperty("TEST_ID", "4b42fbe9-7255-4669-bd4c-893765b83a4a");
    constexpr float MULTIPLICATOR { 3.5 };
    constexpr Duration EXPECTED_DURATION { 16_s + 100_ms };
    const auto duration = 4_s + 600_ms;

    multiply(duration, MULTIPLICATOR, EXPECTED_DURATION);
}

TEST(Duration, multiply_duration_with_self_assign_operator_works) {
    ::testing::Test::RecordProperty("TEST_ID", "ac7e2f7e-984b-4aca-a472-9dc1f1c1f30c");
    constexpr int64_t MULTIPLICATOR { 3 };
    constexpr Duration EXPECTED_DURATION { 6_s + 36_ns };
    auto duration = 2_s + 12_ns; // NOLINT

    duration *= MULTIPLICATOR;

    EXPECT_THAT(duration, Eq(EXPECTED_DURATION));
}

TEST(Duration, multiply_duration_with_fractional_float) {
    ::testing::Test::RecordProperty("TEST_ID", "3adcaec4-06fb-4ae5-a05b-70764fc00d64");
    constexpr float MULTIPLICATOR { 0.5 };
    constexpr Duration EXPECTED_DURATION { 2_s + 800_ms };
    const auto duration = 5_s + 600_ms;

    multiply(duration, MULTIPLICATOR, EXPECTED_DURATION);
}

TEST(Duration, multiply_duration_with_negativ_multiplicator_results_in_zero) {
    ::testing::Test::RecordProperty("TEST_ID", "2bfc4c50-1673-4338-be38-9d9b6d058cc8");
    constexpr Duration EXPECTED_DURATION { 0_s };
    const auto duration = 4_s + 60_ms;

    multiply(duration, -1, EXPECTED_DURATION);
    multiply(duration, -1.0, EXPECTED_DURATION);
}

TEST(Duration, multiply_duration_less_than_one_second_results_in_more_nanoseconds_than64_bit_can_represent) {
    ::testing::Test::RecordProperty("TEST_ID", "2f1bbcb3-3692-4895-a36f-38eed7775b6f");
    constexpr uint64_t MULTIPLICATOR { ((1ULL << 32U) * 42U) + 73U };
    constexpr Duration DURATION = 473_ms + 578_us + 511_ns;
    const auto expected_result = create_duration(85428177141U, 573034055U);

    auto result = MULTIPLICATOR * DURATION;
    EXPECT_THAT(result, Eq(expected_result));
    EXPECT_THAT(result.as_nanos(), Eq(std::numeric_limits<uint64_t>::max()));
    EXPECT_THAT(DURATION * MULTIPLICATOR, Eq(expected_result));
}

TEST(Duration, multiply_duration_results_not_yet_in_saturation) {
    ::testing::Test::RecordProperty("TEST_ID", "b50e3ebf-fa29-4498-9318-491ba030a0cc");
    constexpr uint64_t MULTIPLICATOR { 1343535617188545796U };
    constexpr Duration DURATION = 13_s + 730_ms + 37_ns;
    constexpr Duration EXPECTED_DURATION =
        create_duration(std::numeric_limits<DurationAccessor::SecondsT>::max(), 56194452U);
    static_assert(EXPECTED_DURATION < DurationAccessor::max(),
                  "EXPECTED_DURATION too large to exclude saturation! Please decrease!");

    EXPECT_THAT(MULTIPLICATOR * DURATION, Eq(EXPECTED_DURATION));
    EXPECT_THAT(DURATION * MULTIPLICATOR, Eq(EXPECTED_DURATION));
}

TEST(Duration, multiply_duration_results_in_saturation_due_to_seconds) {
    ::testing::Test::RecordProperty("TEST_ID", "ba59abd8-cdb8-462d-b0b5-c45f7dc80657");
    constexpr uint64_t MULTIPLICATOR { 1343535617188545797U };
    constexpr Duration DURATION = 14_s;

    EXPECT_THAT(MULTIPLICATOR * DURATION, Eq(DurationAccessor::max()));
    EXPECT_THAT(DURATION * MULTIPLICATOR, Eq(DurationAccessor::max()));
}

TEST(Duration, multiply_duration_results_in_saturation_due_to_nanoseconds) {
    ::testing::Test::RecordProperty("TEST_ID", "c0cd4e22-29c5-440f-bad5-fe4d86ebe8ed");
    constexpr uint64_t MULTIPLICATOR { 1343535617188545797U };
    constexpr Duration DURATION = 13_s + 730_ms + 37_ns;

    EXPECT_THAT(MULTIPLICATOR * DURATION, Eq(DurationAccessor::max()));
    EXPECT_THAT(DURATION * MULTIPLICATOR, Eq(DurationAccessor::max()));
}

TEST(Duration, multiply_zero_duration_with_quiet_na_n_results_in_zero_duration) {
    ::testing::Test::RecordProperty("TEST_ID", "4639ac39-639f-4df9-b591-05714f1cbc30");
    EXPECT_THAT(0_s * std::numeric_limits<float>::quiet_NaN(), Eq(0_s));
    EXPECT_THAT(0_s * std::numeric_limits<double>::quiet_NaN(), Eq(0_s));
    EXPECT_THAT(0_s * std::numeric_limits<long double>::quiet_NaN(), Eq(0_s));
}

TEST(Duration, multiply_max_duration_with_quiet_na_n_results_in_max_duration) {
    ::testing::Test::RecordProperty("TEST_ID", "e09e4248-284d-4ab7-993b-1474ac5e3b11");
    EXPECT_THAT(DurationAccessor::max() * std::numeric_limits<float>::quiet_NaN(), Eq(DurationAccessor::max()));
    EXPECT_THAT(DurationAccessor::max() * std::numeric_limits<double>::quiet_NaN(), Eq(DurationAccessor::max()));
    EXPECT_THAT(DurationAccessor::max() * std::numeric_limits<long double>::quiet_NaN(), Eq(DurationAccessor::max()));
}

TEST(Duration, multiply_zero_duration_with_signaling_na_n_results_in_zero_duration) {
    ::testing::Test::RecordProperty("TEST_ID", "5007488d-43e8-456b-97b5-f00832a6b5cf");
    EXPECT_THAT(0_s * std::numeric_limits<float>::signaling_NaN(), Eq(0_s));
    EXPECT_THAT(0_s * std::numeric_limits<double>::signaling_NaN(), Eq(0_s));
    EXPECT_THAT(0_s * std::numeric_limits<long double>::signaling_NaN(), Eq(0_s));
}

TEST(Duration, multiply_max_duration_with_signaling_na_n_results_in_max_duration) {
    ::testing::Test::RecordProperty("TEST_ID", "80e0e30c-6fc2-41d6-a46e-63f6d5ade869");
    EXPECT_THAT(DurationAccessor::max() * std::numeric_limits<float>::signaling_NaN(), Eq(DurationAccessor::max()));
    EXPECT_THAT(DurationAccessor::max() * std::numeric_limits<double>::signaling_NaN(), Eq(DurationAccessor::max()));
    EXPECT_THAT(DurationAccessor::max() * std::numeric_limits<long double>::signaling_NaN(),
                Eq(DurationAccessor::max()));
}

TEST(Duration, multiply_zero_duration_with_pos_inf_results_in_zero_duration) {
    ::testing::Test::RecordProperty("TEST_ID", "71dea32f-1200-4df2-8eff-ea607f1b6a01");
    EXPECT_THAT(0_s * std::numeric_limits<float>::infinity(), Eq(0_ns));
    EXPECT_THAT(0_s * std::numeric_limits<double>::infinity(), Eq(0_ns));
    EXPECT_THAT(0_s * std::numeric_limits<long double>::infinity(), Eq(0_ns));
}

TEST(Duration, multiply_max_duration_with_pos_inf_results_in_max_duration) {
    ::testing::Test::RecordProperty("TEST_ID", "5f66a93a-2df1-4f7d-abbf-03d5424a1534");
    EXPECT_THAT(DurationAccessor::max() * std::numeric_limits<float>::infinity(), Eq(DurationAccessor::max()));
    EXPECT_THAT(DurationAccessor::max() * std::numeric_limits<double>::infinity(), Eq(DurationAccessor::max()));
    EXPECT_THAT(DurationAccessor::max() * std::numeric_limits<long double>::infinity(), Eq(DurationAccessor::max()));
}

TEST(Duration, multiply_zero_duration_with_neg_inf_results_in_zero_duration) {
    ::testing::Test::RecordProperty("TEST_ID", "3931a3df-e09f-414a-8a1e-64bcb1e010b7");
    EXPECT_THAT(0_s * (std::numeric_limits<float>::infinity() * -1.0), Eq(0_ns));
    EXPECT_THAT(0_s * (std::numeric_limits<double>::infinity() * -1.0), Eq(0_ns));
    EXPECT_THAT(0_s * (std::numeric_limits<long double>::infinity() * -1.0), Eq(0_ns));
}

TEST(Duration, multiply_max_duration_with_neg_inf_results_in_zero_duration) {
    ::testing::Test::RecordProperty("TEST_ID", "9f563db0-85fa-4558-8928-8fe730f3ff47");
    EXPECT_THAT(DurationAccessor::max() * (std::numeric_limits<float>::infinity() * -1.0), Eq(0_ns));
    EXPECT_THAT(DurationAccessor::max() * (std::numeric_limits<double>::infinity() * -1.0), Eq(0_ns));
    EXPECT_THAT(DurationAccessor::max() * (std::numeric_limits<long double>::infinity() * -1.0), Eq(0_ns));
}

TEST(Duration, multiply_duration_with_minimal_float_results_in_zero) {
    ::testing::Test::RecordProperty("TEST_ID", "e2e9ffc7-a33d-4c2e-ac93-57f621fd66f9");
    constexpr float MULTIPLICATOR { std::numeric_limits<float>::min() };
    constexpr Duration DURATION = 13_s + 730_ms + 37_ns;
    auto expected_duration = create_duration(0U, 0U);

    EXPECT_THAT(MULTIPLICATOR * DURATION, Eq(expected_duration));
    EXPECT_THAT(DURATION * MULTIPLICATOR, Eq(expected_duration));
}

TEST(Duration, multiply_duration_with_minimal_double_results_in_zero) {
    ::testing::Test::RecordProperty("TEST_ID", "29be653f-8ec7-4ab6-b2ea-501bf7e78c1d");
    constexpr double MULTIPLICATOR { std::numeric_limits<double>::min() };
    constexpr Duration DURATION = 13_s + 730_ms + 37_ns;
    auto expected_duration = create_duration(0U, 0U);

    EXPECT_THAT(MULTIPLICATOR * DURATION, Eq(expected_duration));
    EXPECT_THAT(DURATION * MULTIPLICATOR, Eq(expected_duration));
}

TEST(Duration, multiply_max_duration_with_float_one_results_in_max_duration) {
    ::testing::Test::RecordProperty("TEST_ID", "c700d04c-8847-4234-ad94-a08bb57bac9a");
    EXPECT_THAT(DurationAccessor::max() * 1.0, Eq(DurationAccessor::max()));
}

TEST(Duration, multiply_max_duration_with_double_one_results_in_max_duration) {
    ::testing::Test::RecordProperty("TEST_ID", "c78a9d3a-2613-4b67-ba8b-a7a7b368a4ed");
    EXPECT_THAT(DurationAccessor::max() * 1.0, Eq(DurationAccessor::max()));
}

TEST(Duration, multiply_duration_with_float_results_in_saturation_due_to_seconds) {
    ::testing::Test::RecordProperty("TEST_ID", "085de609-5f38-4a63-8719-f15263ca448b");
    constexpr float MULTIPLICATOR { 1343535617188545797.0F };
    constexpr Duration DURATION = 14_s;

    EXPECT_THAT(MULTIPLICATOR * DURATION, Eq(DurationAccessor::max()));
    EXPECT_THAT(DURATION * MULTIPLICATOR, Eq(DurationAccessor::max()));
}

TEST(Duration, multiply_duration_with_double_results_in_saturation_due_to_seconds) {
    ::testing::Test::RecordProperty("TEST_ID", "3cc27397-5e7f-45bd-b5a1-bbdb78920daf");
    constexpr double MULTIPLICATOR { 1343535617188545797.0 };
    constexpr Duration DURATION = 14_s;

    EXPECT_THAT(MULTIPLICATOR * DURATION, Eq(DurationAccessor::max()));
    EXPECT_THAT(DURATION * MULTIPLICATOR, Eq(DurationAccessor::max()));
}

TEST(Duration, multiply_duration_with_float_results_in_saturation_due_to_nanoseconds) {
    ::testing::Test::RecordProperty("TEST_ID", "38ad7f5d-480a-4c4b-8ff0-8a24f3d6f2a6");
    constexpr float MULTIPLICATOR { 1343535617188545797.0F };
    constexpr Duration DURATION = 13_s + 930_ms + 37_ns;

    EXPECT_THAT(MULTIPLICATOR * DURATION, Eq(DurationAccessor::max()));
    EXPECT_THAT(DURATION * MULTIPLICATOR, Eq(DurationAccessor::max()));
}

TEST(Duration, multiply_duration_with_double_results_in_saturation_due_to_nanoseconds) {
    ::testing::Test::RecordProperty("TEST_ID", "2c9f05a8-8392-4924-8a76-fd1447aa675e");
    constexpr double MULTIPLICATOR { 1343535617188545797.0 };
    constexpr Duration DURATION = 13_s + 930_ms + 37_ns;

    EXPECT_THAT(MULTIPLICATOR * DURATION, Eq(DurationAccessor::max()));
    EXPECT_THAT(DURATION * MULTIPLICATOR, Eq(DurationAccessor::max()));
}

TEST(Duration, std_streaming_operator) {
    ::testing::Test::RecordProperty("TEST_ID", "526d6cf3-b3df-44ce-8668-a0a170c94919");
    std::stringstream capture;
    auto* clog_buffer = std::clog.rdbuf();
    std::clog.rdbuf(capture.rdbuf());

    capture.str("");
    std::clog << 0_s;
    EXPECT_STREQ(capture.str().c_str(), "0s 0ns");

    capture.str("");
    const auto less_than_one_second = 42_ns;
    std::clog << less_than_one_second;
    EXPECT_STREQ(capture.str().c_str(), "0s 42ns");

    capture.str("");
    const auto more_than_one_second = 13_s + 73_ms + 37_us + 42_ns;
    std::clog << more_than_one_second;
    EXPECT_STREQ(capture.str().c_str(), "13s 73037042ns");

    std::clog.rdbuf(clog_buffer);
}

TEST(Duration, log_streaming_operator) {
    ::testing::Test::RecordProperty("TEST_ID", "2ce98e19-17be-47fa-b5e7-f4d9dacd0855");

    iox2::legacy::testing::Logger_Mock logger_mock {};

    {
        IOX2_LOGSTREAM_MOCK(logger_mock) << 0_s;
    }
    ASSERT_THAT(logger_mock.logs.size(), Eq(1U));
    EXPECT_THAT(logger_mock.logs[0].message, StrEq("0s 0ns"));
    logger_mock.logs.clear();

    {
        const auto less_than_one_second = 42_ns;
        IOX2_LOGSTREAM_MOCK(logger_mock) << less_than_one_second;
    }
    ASSERT_THAT(logger_mock.logs.size(), Eq(1U));
    EXPECT_THAT(logger_mock.logs[0].message, StrEq("0s 42ns"));
    logger_mock.logs.clear();

    {
        const auto more_than_one_second = 13_s + 73_ms + 37_us + 42_ns;
        IOX2_LOGSTREAM_MOCK(logger_mock) << more_than_one_second;
    }
    ASSERT_THAT(logger_mock.logs.size(), Eq(1U));
    EXPECT_THAT(logger_mock.logs[0].message, StrEq("13s 73037042ns"));
    logger_mock.logs.clear();
}


// END ARITHMETIC TESTS
} // namespace
