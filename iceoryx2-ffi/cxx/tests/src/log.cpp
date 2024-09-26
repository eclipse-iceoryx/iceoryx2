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
    set_log_level(LogLevel::INFO);
    EXPECT_EQ(get_log_level(), LogLevel::INFO);

    set_log_level(LogLevel::DEBUG);
    EXPECT_EQ(get_log_level(), LogLevel::DEBUG);

    set_log_level(LogLevel::WARN);
    EXPECT_EQ(get_log_level(), LogLevel::WARN);

    set_log_level(LogLevel::ERROR);
    EXPECT_EQ(get_log_level(), LogLevel::ERROR);

    set_log_level(LogLevel::FATAL);
    EXPECT_EQ(get_log_level(), LogLevel::FATAL);
}

} // namespace
