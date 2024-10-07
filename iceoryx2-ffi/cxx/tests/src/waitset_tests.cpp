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

#include "iox2/service_type.hpp"
#include "iox2/waitset.hpp"
#include "test.hpp"

namespace {
using namespace iox2;

template <typename T>
struct WaitSetTest : public ::testing::Test {
    static constexpr ServiceType TYPE = T::TYPE;

    auto create_sut() -> WaitSet<TYPE> {
        return WaitSetBuilder().create<TYPE>().expect("");
    }
};

TYPED_TEST_SUITE(WaitSetTest, iox2_testing::ServiceTypes);

TYPED_TEST(WaitSetTest, newly_created_waitset_is_empty) {
    auto sut = this->create_sut();

    ASSERT_THAT(sut.len(), Eq(0));
    ASSERT_THAT(sut.is_empty(), Eq(true));
}
} // namespace
