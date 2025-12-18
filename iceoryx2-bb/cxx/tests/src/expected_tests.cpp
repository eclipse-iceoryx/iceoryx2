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

#include "iox2/bb/stl/expected.hpp"

#include "testing/observable.hpp"

#include <gmock/gmock.h>
#include <gtest/gtest.h>

namespace {
using namespace ::testing;
using namespace ::iox2::bb::stl;

using iox2::bb::testing::Observable;

class ExpectedFixture : public iox2::bb::testing::DetectLeakedObservablesFixture { };

struct Value {
    int32_t val { 0 };
};

struct ObservableConvertible {
    // NOLINTNEXTLINE(hicpp-explicit-conversions) required for test
    operator Observable() {
        return Observable {};
    }
};

struct Error {
    int32_t err { 0 };
};

// BEGIN Unexpected

TEST(ExpectedFixture, unexpected_can_be_constructed_with_error_lvalue) {
    Observable::s_counter.was_initialized = 0;
    auto observable = Observable {};

    const Unexpected<Observable> sut(observable);

    EXPECT_EQ(Observable::s_counter.was_initialized, 1);
}

TEST(ExpectedFixture, unexpected_can_be_constructed_with_error_rvalue) {
    Observable::s_counter.was_initialized = 0;
    auto observable = Observable {};

    const Unexpected<Observable> sut(std::move(observable));

    EXPECT_EQ(Observable::s_counter.was_initialized, 1);
}

TEST(ExpectedFixture, unexpected_can_be_constructed_with_error_in_place) {
    Observable::s_counter.was_initialized = 0;

    const Unexpected<Observable> sut(IN_PLACE, Observable {});

    EXPECT_EQ(Observable::s_counter.was_initialized, 1);
}

TEST(Expected, error_of_lvalue_unexpected_has_correct_error) {
    constexpr uint32_t EXPECTED_ERROR { 23 };
    Unexpected<Error> sut(Error { EXPECTED_ERROR });

    EXPECT_THAT(sut.error().err, Eq(EXPECTED_ERROR));
}

TEST(Expected, error_of_const_lvalue_unexpected_has_correct_error) {
    constexpr uint32_t EXPECTED_ERROR { 37 };
    const Unexpected<Error> sut(Error { EXPECTED_ERROR });

    EXPECT_THAT(sut.error().err, Eq(EXPECTED_ERROR));
}

TEST(Expected, error_of_rvalue_unexpected_has_correct_error) {
    constexpr uint32_t EXPECTED_ERROR { 66 };
    Unexpected<Error> sut(Error { EXPECTED_ERROR });

    EXPECT_THAT(std::move(sut).error().err, Eq(EXPECTED_ERROR));
}

TEST(Expected, error_of_const_rvalue_unexpected_has_correct_error) {
    constexpr uint32_t EXPECTED_ERROR { 101 };
    const Unexpected<Error> sut(Error { EXPECTED_ERROR });

    EXPECT_THAT(std::move(sut).error().err, Eq(EXPECTED_ERROR));
}

// END Unexpected


// BEGIN Expected ctors

TEST(ExpectedFixture, expected_can_be_constructed_with_default_value) {
    Observable::s_counter.was_initialized = 0;

    const Expected<Observable, Error> sut1 {};

    EXPECT_EQ(Observable::s_counter.was_initialized, 1);
    EXPECT_TRUE(sut1.has_value());

    const Expected<void, Error> sut2 {};
    EXPECT_TRUE(sut2.has_value());
}

TEST(ExpectedFixture, expected_can_be_constructed_with_value) {
    Observable::s_counter.was_initialized = 0;

    const Expected<Observable, Error> sut = Observable {};

    EXPECT_EQ(Observable::s_counter.was_initialized, 1);
}

TEST(ExpectedFixture, expected_can_be_constructed_with_value_explicit) {
    Observable::s_counter.was_initialized = 0;

    const Expected<Observable, Error> sut(ObservableConvertible {});

    EXPECT_EQ(Observable::s_counter.was_initialized, 1);
}

TEST(ExpectedFixture, expected_can_be_constructed_with_unexpected_lvalue) {
    Observable::s_counter.was_initialized = 0;
    auto unex = Unexpected<Observable>(Observable {});

    const Expected<Value, Observable> sut(unex);

    EXPECT_EQ(Observable::s_counter.was_initialized, 1);
}

TEST(ExpectedFixture, expected_can_be_constructed_with_unexpected_rvalue) {
    Observable::s_counter.was_initialized = 0;
    auto unex = Unexpected<Observable>(Observable {});

    const Expected<Value, Observable> sut(std::move(unex));

    EXPECT_EQ(Observable::s_counter.was_initialized, 1);
}

TEST(ExpectedFixture, expected_can_be_constructed_with_value_in_place) {
    Observable::s_counter.was_initialized = 0;

    const Expected<Observable, Error> sut(IN_PLACE, Observable {});

    EXPECT_EQ(Observable::s_counter.was_initialized, 1);
}

TEST(Expected, expected_can_be_constructed_with_void_type) {
    const Expected<void, Error> sut(IN_PLACE);

    ASSERT_TRUE(sut.has_value());
}

TEST(ExpectedFixture, expected_can_be_constructed_with_error) {
    Observable::s_counter.was_initialized = 0;

    const Expected<Value, Observable> sut(UNEXPECT, Observable {});

    EXPECT_EQ(Observable::s_counter.was_initialized, 1);
}

TEST(ExpectedFixture, expected_can_be_copy_constructed_with_value_in_place) {
    const Expected<Observable, Error> val(IN_PLACE, Observable {});

    Observable::s_counter.was_initialized = 0;
    Observable::s_counter.was_copy_constructed = 0;
    Observable::s_counter.was_move_constructed = 0;

    // NOLINTNEXTLINE(performance-unnecessary-copy-initialization) test of copy ctor
    const Expected<Observable, Error> sut(val);

    EXPECT_EQ(Observable::s_counter.was_initialized, 0);
    EXPECT_EQ(Observable::s_counter.was_copy_constructed, 1);
    EXPECT_EQ(Observable::s_counter.was_move_constructed, 0);
}

TEST(ExpectedFixture, expected_can_be_copy_constructed_with_error) {
    const Expected<Value, Observable> err(UNEXPECT, Observable {});

    Observable::s_counter.was_initialized = 0;
    Observable::s_counter.was_copy_constructed = 0;
    Observable::s_counter.was_move_constructed = 0;

    // NOLINTNEXTLINE(performance-unnecessary-copy-initialization) test of copy ctor
    const Expected<Value, Observable> sut(err);

    EXPECT_EQ(Observable::s_counter.was_initialized, 0);
    EXPECT_EQ(Observable::s_counter.was_copy_constructed, 1);
    EXPECT_EQ(Observable::s_counter.was_move_constructed, 0);
}

TEST(ExpectedFixture, expected_can_be_move_constructed_with_value_in_place) {
    Expected<Observable, Error> val(IN_PLACE, Observable {});

    Observable::s_counter.was_initialized = 0;
    Observable::s_counter.was_copy_constructed = 0;
    Observable::s_counter.was_move_constructed = 0;

    const Expected<Observable, Error> sut(std::move(val));

    EXPECT_EQ(Observable::s_counter.was_initialized, 0);
    EXPECT_EQ(Observable::s_counter.was_copy_constructed, 0);
    EXPECT_EQ(Observable::s_counter.was_move_constructed, 1);
}

TEST(ExpectedFixture, expected_can_be_move_constructed_with_error) {
    Expected<Value, Observable> err(UNEXPECT, Observable {});

    Observable::s_counter.was_initialized = 0;
    Observable::s_counter.was_copy_constructed = 0;
    Observable::s_counter.was_move_constructed = 0;

    const Expected<Value, Observable> sut(std::move(err));

    EXPECT_EQ(Observable::s_counter.was_initialized, 0);
    EXPECT_EQ(Observable::s_counter.was_copy_constructed, 0);
    EXPECT_EQ(Observable::s_counter.was_move_constructed, 1);
}

// END Expected ctors


// BEGIN Expected assignment operators

TEST(ExpectedFixture, expected_can_be_copy_assigned_with_value) {
    const Expected<Observable, Error> other(IN_PLACE, Observable {});
    Expected<Observable, Error> sut(IN_PLACE, Observable {});

    Observable::s_counter.was_initialized = 0;
    Observable::s_counter.was_copy_assigned = 0;
    Observable::s_counter.was_move_assigned = 0;

    sut = other;

    EXPECT_EQ(Observable::s_counter.was_initialized, 0);
    EXPECT_EQ(Observable::s_counter.was_copy_assigned, 1);
    EXPECT_EQ(Observable::s_counter.was_move_assigned, 0);
}

TEST(ExpectedFixture, expected_can_be_copy_assigned_with_error) {
    const Expected<Value, Observable> other(UNEXPECT, Observable {});
    Expected<Value, Observable> sut(UNEXPECT, Observable {});

    Observable::s_counter.was_initialized = 0;
    Observable::s_counter.was_copy_assigned = 0;
    Observable::s_counter.was_move_assigned = 0;

    sut = other;

    EXPECT_EQ(Observable::s_counter.was_initialized, 0);
    EXPECT_EQ(Observable::s_counter.was_copy_assigned, 1);
    EXPECT_EQ(Observable::s_counter.was_move_assigned, 0);
}

TEST(ExpectedFixture, expected_can_be_move_assigned_with_value) {
    Expected<Observable, Error> other(IN_PLACE, Observable {});
    Expected<Observable, Error> sut(IN_PLACE, Observable {});

    Observable::s_counter.was_initialized = 0;
    Observable::s_counter.was_copy_assigned = 0;
    Observable::s_counter.was_move_assigned = 0;

    sut = std::move(other);

    EXPECT_EQ(Observable::s_counter.was_initialized, 0);
    EXPECT_EQ(Observable::s_counter.was_copy_assigned, 0);
    EXPECT_EQ(Observable::s_counter.was_move_assigned, 1);
}

TEST(ExpectedFixture, expected_can_be_move_assigned_with_error) {
    Expected<Value, Observable> other(UNEXPECT, Observable {});
    Expected<Value, Observable> sut(UNEXPECT, Observable {});

    Observable::s_counter.was_initialized = 0;
    Observable::s_counter.was_copy_assigned = 0;
    Observable::s_counter.was_move_assigned = 0;

    sut = std::move(other);

    EXPECT_EQ(Observable::s_counter.was_initialized, 0);
    EXPECT_EQ(Observable::s_counter.was_copy_assigned, 0);
    EXPECT_EQ(Observable::s_counter.was_move_assigned, 1);
}
// END Expected assignment operators

// BEGIN Expected dtor

TEST(ExpectedFixture, expected_with_value_is_destructed) {
    Observable::s_counter.was_destructed = 0;
    {
        const Expected<Observable, Error> sut(IN_PLACE, Observable {});
        EXPECT_EQ(Observable::s_counter.was_destructed, 1);
        Observable::s_counter.was_destructed = 0;
    }

    EXPECT_EQ(Observable::s_counter.was_destructed, 1);
}

TEST(ExpectedFixture, expected_with_error_is_destructed) {
    Observable::s_counter.was_destructed = 0;
    {
        const Expected<Value, Observable> sut(UNEXPECT, Observable {});
        EXPECT_EQ(Observable::s_counter.was_destructed, 1);
        Observable::s_counter.was_destructed = 0;
    }

    EXPECT_EQ(Observable::s_counter.was_destructed, 1);
}

// END Expected dtor


// BEGIN Expected has_value

TEST(ExpectedFixture, has_value_of_expected_with_value_is_true) {
    const Expected<Observable, Error> sut(IN_PLACE, Observable {});

    EXPECT_TRUE(sut.has_value());
}

TEST(ExpectedFixture, has_value_of_expected_with_error_is_false) {
    const Expected<Value, Observable> sut(UNEXPECT, Observable {});

    EXPECT_FALSE(sut.has_value());
}

// END Expected has_value


// BEGIN Expected operator bool

TEST(ExpectedFixture, operator_bool_of_expected_with_value_is_true) {
    const Expected<Observable, Error> sut(IN_PLACE, Observable {});

    EXPECT_TRUE(sut.operator bool());
}

TEST(ExpectedFixture, operator_bool_of_expected_with_error_is_false) {
    const Expected<Value, Observable> sut(UNEXPECT, Observable {});

    EXPECT_FALSE(sut.operator bool());
}

// END Expected operator bool


// BEGIN Expected value

TEST(Expected, value_of_lvalue_expected_with_void_type_has_correct_type) {
    const Expected<void, Error> sut(IN_PLACE);

    using ValueType = decltype(sut.value());
    const bool value_type_is_void = std::is_same<ValueType, void>::value;
    EXPECT_TRUE(value_type_is_void);
}

TEST(Expected, value_of_lvalue_expected_with_value_has_correct_value) {
    constexpr uint32_t EXPECTED_VALUE { 23 };
    const Expected<Value, Error> sut(IN_PLACE, Value { EXPECTED_VALUE });

    EXPECT_THAT(sut.value().val, Eq(EXPECTED_VALUE));
}

TEST(Expected, value_of_const_lvalue_expected_with_value_has_correct_value) {
    constexpr uint32_t EXPECTED_VALUE { 37 };
    const Expected<Value, Error> sut(IN_PLACE, Value { EXPECTED_VALUE });

    EXPECT_THAT(sut.value().val, Eq(EXPECTED_VALUE));
}

TEST(Expected, value_of_rvalue_expected_with_void_type_has_correct_type) {
    const Expected<void, Error> sut(IN_PLACE);

    using ValueType = decltype(std::move(sut).value());
    const bool value_type_is_void = std::is_same<ValueType, void>::value;
    EXPECT_TRUE(value_type_is_void);
}

TEST(Expected, value_of_rvalue_expected_with_value_has_correct_value) {
    constexpr uint32_t EXPECTED_VALUE { 66 };
    Expected<Value, Error> sut(IN_PLACE, Value { EXPECTED_VALUE });

    EXPECT_THAT(std::move(sut).value().val, Eq(EXPECTED_VALUE));
}

TEST(Expected, value_of_const_rvalue_expected_with_value_has_correct_value) {
    constexpr uint32_t EXPECTED_VALUE { 101 };
    const Expected<Value, Error> sut(IN_PLACE, Value { EXPECTED_VALUE });

    EXPECT_THAT(std::move(sut).value().val, Eq(EXPECTED_VALUE));
}

// END Expected value


// BEGIN Expected operator star

TEST(Expected, operator_star_of_lvalue_expected_with_void_type_has_correct_type) {
    const Expected<void, Error> sut(IN_PLACE);

    using ValueType = decltype(sut.operator*());
    const bool value_type_is_void = std::is_same<ValueType, void>::value;
    EXPECT_TRUE(value_type_is_void);
}

TEST(Expected, operator_star_of_lvalue_expected_with_value_has_correct_value) {
    constexpr uint32_t EXPECTED_VALUE { 23 };
    const Expected<Value, Error> sut(IN_PLACE, Value { EXPECTED_VALUE });

    EXPECT_THAT(sut.operator*().val, Eq(EXPECTED_VALUE));
}

TEST(Expected, operator_star_of_const_lvalue_expected_with_value_has_correct_value) {
    constexpr uint32_t EXPECTED_VALUE { 37 };
    const Expected<Value, Error> sut(IN_PLACE, Value { EXPECTED_VALUE });

    EXPECT_THAT(sut.operator*().val, Eq(EXPECTED_VALUE));
}

TEST(Expected, operator_star_of_rvalue_expected_with_value_has_correct_value) {
    constexpr uint32_t EXPECTED_VALUE { 66 };
    Expected<Value, Error> sut(IN_PLACE, Value { EXPECTED_VALUE });

    EXPECT_THAT(std::move(sut).operator*().val, Eq(EXPECTED_VALUE));
}

TEST(Expected, operator_star_of_const_rvalue_expected_with_value_has_correct_value) {
    constexpr uint32_t EXPECTED_VALUE { 101 };
    const Expected<Value, Error> sut(IN_PLACE, Value { EXPECTED_VALUE });

    EXPECT_THAT(std::move(sut).operator*().val, Eq(EXPECTED_VALUE));
}

// END Expected operator star


// BEGIN Expected operator arrow

TEST(Expected, operator_arrow_of_lvalue_expected_with_value_has_correct_value) {
    constexpr uint32_t EXPECTED_VALUE { 23 };
    Expected<Value, Error> sut(IN_PLACE, Value { EXPECTED_VALUE });

    EXPECT_THAT(sut->val, Eq(EXPECTED_VALUE));
}

TEST(Expected, operator_arrow_of_const_lvalue_expected_with_value_has_correct_value) {
    constexpr uint32_t EXPECTED_VALUE { 37 };
    const Expected<Value, Error> sut(IN_PLACE, Value { EXPECTED_VALUE });

    EXPECT_THAT(sut->val, Eq(EXPECTED_VALUE));
}

// END Expected operator arrow


// BEGIN Expected error

TEST(Expected, error_of_lvalue_expected_with_error_has_correct_error) {
    constexpr uint32_t EXPECTED_ERROR { 23 };
    Expected<Value, Error> sut(UNEXPECT, Error { EXPECTED_ERROR });

    EXPECT_THAT(sut.error().err, Eq(EXPECTED_ERROR));
}

TEST(Expected, error_of_const_lvalue_expected_with_error_has_correct_error) {
    constexpr uint32_t EXPECTED_ERROR { 37 };
    const Expected<Value, Error> sut(UNEXPECT, Error { EXPECTED_ERROR });

    EXPECT_THAT(sut.error().err, Eq(EXPECTED_ERROR));
}

TEST(Expected, error_of_rvalue_expected_with_error_has_correct_error) {
    constexpr uint32_t EXPECTED_ERROR { 66 };
    Expected<Value, Error> sut(UNEXPECT, Error { EXPECTED_ERROR });

    EXPECT_THAT(std::move(sut).error().err, Eq(EXPECTED_ERROR));
}

TEST(Expected, error_of_const_rvalue_expected_with_error_has_correct_error) {
    constexpr uint32_t EXPECTED_ERROR { 101 };
    const Expected<Value, Error> sut(UNEXPECT, Error { EXPECTED_ERROR });

    EXPECT_THAT(std::move(sut).error().err, Eq(EXPECTED_ERROR));
}

// END Expected error

} // namespace
