// Copyright (c) 2020 - 2023 by Apex.AI Inc. All rights reserved.
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

#include "iox2/bb/detail/source_location.hpp"

#include <gmock/gmock.h>
#include <gtest/gtest.h>

namespace {
using namespace ::testing;
using namespace iox2::bb::detail;

auto constexpr sut(SourceLocation value = SourceLocation::current()) -> SourceLocation {
    return value;
}

auto constexpr hypnotoad() -> SourceLocation {
    return SourceLocation::current();
}

TEST(SourceLocation, file_has_correct_value) {
    ASSERT_THAT(sut().file_name(), StrEq(__FILE__));
}

TEST(SourceLocation, line_has_correct_value) {
    ASSERT_THAT(sut().line(), Eq(__LINE__));
}

TEST(SourceLocation, function_has_correct_value) {
    // NOTE: __FUCTION__ does not match `__builtin_FUNCTION()` on Windows;
    //       -> we need to use a dedicated function with a known name
    ASSERT_THAT(hypnotoad().function_name(), StrEq("hypnotoad"));
}

} // namespace
