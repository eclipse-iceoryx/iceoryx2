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

#include "iox2/bb/expected.hpp"

#include <gmock/gmock.h>
#include <gtest/gtest.h>

namespace {
using namespace ::testing;
using namespace iox2::bb;

struct Value { };

struct Error {
    int32_t err { 0 };
};

TEST(ExpectedErrFreeFunction, expected_can_be_constructed_with_err_free_function_from_lvalue) {
    constexpr uint32_t EXPECTED_ERROR { 23 };
    auto error = Error { EXPECTED_ERROR };

    const Expected<Value, Error> sut = err(Error { error });

    ASSERT_FALSE(sut.has_value());
    ASSERT_THAT(sut.error().err, Eq(EXPECTED_ERROR));
}

TEST(ExpectedErrFreeFunction, expected_can_be_constructed_with_err_free_function_from_rvalue) {
    constexpr uint32_t EXPECTED_ERROR { 32 };

    const Expected<Value, Error> sut = err(Error { EXPECTED_ERROR });

    ASSERT_FALSE(sut.has_value());
    ASSERT_THAT(sut.error().err, Eq(EXPECTED_ERROR));
}

} // namespace
