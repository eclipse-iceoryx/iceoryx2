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

#include "iox2/legacy/error_reporting/custom/default/error_handler.hpp"
#include "iox2/legacy/error_reporting/source_location.hpp"

namespace {

using namespace ::testing;
using namespace iox2::legacy::er;

constexpr ErrorCode CODE { 73 };

class DefaultErrorHandler_test : public Test {
  public:
    void SetUp() override {
    }

    void TearDown() override {
    }

    DefaultErrorHandler sut;
};

// Can only check that it can be called, there are no observable effects.
TEST_F(DefaultErrorHandler_test, panicDoesNothing) {
    ::testing::Test::RecordProperty("TEST_ID", "0d7f7048-94d3-42b7-a25a-1a7b506fd835");
    sut.onPanic();
}

// Can only check that it can be called, there are no observable effects.
TEST_F(DefaultErrorHandler_test, reportDoesNothing) {
    ::testing::Test::RecordProperty("TEST_ID", "9e288318-c756-4666-b779-b944b89ffaf5");
    sut.onReportError(ErrorDescriptor { IOX2_CURRENT_SOURCE_LOCATION, CODE });
    sut.onReportViolation(ErrorDescriptor { IOX2_CURRENT_SOURCE_LOCATION, CODE });
}

} // namespace
