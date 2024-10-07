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

#include <vector>

#include "iox2/node.hpp"
#include "iox2/service_name.hpp"
#include "iox2/service_type.hpp"
#include "iox2/waitset.hpp"
#include "test.hpp"


namespace {
using namespace iox2;

auto generate_name() -> ServiceName {
    static std::atomic<uint64_t> COUNTER = 0;
    return ServiceName::create((std::string("waitset_tests_") + std::to_string(COUNTER.fetch_add(1))).c_str())
        .expect("");
}

template <typename T>
struct WaitSetTest : public ::testing::Test {
    static constexpr ServiceType TYPE = T::TYPE;

    WaitSetTest()
        : node { NodeBuilder().create<TYPE>().expect("") }
        , event { node.service_builder(generate_name()).event().create().expect("") } {
    }

    auto create_sut() -> WaitSet<TYPE> {
        return WaitSetBuilder().create<TYPE>().expect("");
    }

    auto create_listener() -> Listener<TYPE> {
        return event.listener_builder().create().expect("");
    }

    // NOLINTBEGIN(misc-non-private-member-variables-in-classes), come on, its a test
    Node<TYPE> node;
    PortFactoryEvent<TYPE> event;
    // NOLINTEND(misc-non-private-member-variables-in-classes)
};

TYPED_TEST_SUITE(WaitSetTest, iox2_testing::ServiceTypes);

TYPED_TEST(WaitSetTest, newly_created_waitset_is_empty) {
    auto sut = this->create_sut();

    ASSERT_THAT(sut.len(), Eq(0));
    ASSERT_THAT(sut.is_empty(), Eq(true));
}

TYPED_TEST(WaitSetTest, attaching_different_elements_works) {
    constexpr uint64_t NUMBER_OF_DEADLINES = 0;
    constexpr uint64_t NUMBER_OF_NOTIFICATIONS = 1;
    constexpr uint64_t NUMBER_OF_INTERVALS = 7;
    auto sut = this->create_sut();

    std::vector<Listener<TestFixture::TYPE>> listeners;
    std::vector<Guard<TestFixture::TYPE>> guards;

    for (uint64_t idx = 0; idx < NUMBER_OF_INTERVALS; ++idx) {
        guards.emplace_back(sut.attach_interval(iox::units::Duration::fromSeconds(idx + 1)).expect(""));
        ASSERT_THAT(sut.len(), Eq(idx + 1));
        ASSERT_THAT(sut.is_empty(), Eq(false));
    }

    for (uint64_t idx = 0; idx < NUMBER_OF_NOTIFICATIONS; ++idx) {
        auto listener = this->create_listener();
        guards.emplace_back(sut.attach_notification(listener).expect(""));
        listeners.emplace_back(std::move(listener));
        ASSERT_THAT(sut.len(), Eq(NUMBER_OF_INTERVALS + idx + 1));
        ASSERT_THAT(sut.is_empty(), Eq(false));
    }

    for (uint64_t idx = 0; idx < NUMBER_OF_DEADLINES; ++idx) {
        auto listener = this->create_listener();
        guards.emplace_back(sut.attach_deadline(listener, iox::units::Duration::fromSeconds(idx + 1)).expect(""));
        listeners.emplace_back(std::move(listener));
        ASSERT_THAT(sut.len(), Eq(NUMBER_OF_INTERVALS + NUMBER_OF_NOTIFICATIONS + idx + 1));
        ASSERT_THAT(sut.is_empty(), Eq(false));
    }

    guards.clear();
    listeners.clear();
    ASSERT_THAT(sut.len(), Eq(0));
    ASSERT_THAT(sut.is_empty(), Eq(true));
}
} // namespace
