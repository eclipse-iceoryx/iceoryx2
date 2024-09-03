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

#include "iox2/config.hpp"

#include "test.hpp"

namespace {
using namespace iox2;

TEST(Config, global_prefix_works) {
    const auto test_value = iox::FileName::create("oh_my_dot").expect("");
    auto config = Config();

    config.global().set_prefix(test_value);
    ASSERT_THAT(config.global().prefix(), StrEq(test_value.as_string().c_str()));
}
} // namespace
