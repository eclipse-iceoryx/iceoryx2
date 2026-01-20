// Copyright (c) 2019 by Robert Bosch GmbH. All rights reserved.
// Copyright (c) 2021 - 2022 by Apex AI Inc. All rights reserved.
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

#include "iox2/bb/into.hpp"

#include <gmock/gmock.h>
#include <gtest/gtest.h>

namespace {
enum class A : uint8_t {
    A1 = 13,
    A2
};

enum class B : uint8_t {
    B1 = 42,
    B2
};
} // namespace

namespace iox2 {
namespace bb {
template <>
constexpr auto from<A, B>(A value) noexcept -> B {
    switch (value) {
    case A::A1:
        return B::B1;
    case A::A2:
        return B::B2;
    }
}
} // namespace bb
} // namespace iox2

namespace {
using namespace ::testing;
using namespace iox2::bb;

TEST(Into, from_works_as_constexpr) {
    ::testing::Test::RecordProperty("TEST_ID", "5b7cac32-c0ef-4f29-8314-59ed8850d1f5");
    constexpr A FROM_VALUE { A::A1 };
    constexpr B TO_VALUE { B::B1 };
    constexpr B SUT = from<A, B>(FROM_VALUE);
    EXPECT_EQ(SUT, TO_VALUE);
}

TEST(Into, into_works_when_from_is_specialized) {
    ::testing::Test::RecordProperty("TEST_ID", "1d4331e5-f603-4e50-bdb2-75df57b0b517");
    constexpr A FROM_VALUE { A::A2 };
    constexpr B TO_VALUE { B::B2 };
    constexpr B SUT = into<B>(FROM_VALUE);
    EXPECT_EQ(SUT, TO_VALUE);
}
} // namespace
