// Copyright (c) 2024 Contributors to the Eclipse Foundation
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

#include "iox2/log.hpp"
#include "iox2/log_level.hpp"

#include "test.hpp"

namespace {
using namespace iox2;

TEST(LogLevel, can_set_and_get_log_level) {
    set_log_level(LogLevel::Trace);
    EXPECT_EQ(get_log_level(), LogLevel::Trace);

    set_log_level(LogLevel::Debug);
    EXPECT_EQ(get_log_level(), LogLevel::Debug);

    set_log_level(LogLevel::Info);
    EXPECT_EQ(get_log_level(), LogLevel::Info);

    set_log_level(LogLevel::Warn);
    EXPECT_EQ(get_log_level(), LogLevel::Warn);

    set_log_level(LogLevel::Error);
    EXPECT_EQ(get_log_level(), LogLevel::Error);

    set_log_level(LogLevel::Fatal);
    EXPECT_EQ(get_log_level(), LogLevel::Fatal);
}

} // namespace
