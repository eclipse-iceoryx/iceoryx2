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

#include "iox2/listener.hpp"
#include "iox2/node.hpp"
#include "iox2/node_name.hpp"
#include "iox2/notifier.hpp"
#include "iox2/publisher.hpp"
#include "iox2/service_name.hpp"
#include "iox2/subscriber.hpp"

#include "test.hpp"

#include <atomic>

namespace {
using namespace iox2;

auto generate_name() -> ServiceName {
    static std::atomic<uint64_t> COUNTER = 0;
    return ServiceName::create((std::string("unique_port_id_tests_") + std::to_string(COUNTER.fetch_add(1))).c_str())
        .expect("");
}

template <typename T>
struct UniquePortIdTest : public ::testing::Test {
    static constexpr ServiceType TYPE = T::TYPE;

    UniquePortIdTest()
        : node { NodeBuilder().create<TYPE>().expect("") }
        , service_name { generate_name() }
        , event { node.service_builder(service_name).event().create().expect("") }
        , pubsub { node.service_builder(service_name).template publish_subscribe<uint64_t>().create().expect("") }
        , listener_1 { event.listener_builder().create().expect("") }
        , listener_2 { event.listener_builder().create().expect("") }
        , notifier_1 { event.notifier_builder().create().expect("") }
        , notifier_2 { event.notifier_builder().create().expect("") }
        , publisher_1 { pubsub.publisher_builder().create().expect("") }
        , publisher_2 { pubsub.publisher_builder().create().expect("") }
        , subscriber_1 { pubsub.subscriber_builder().create().expect("") }
        , subscriber_2 { pubsub.subscriber_builder().create().expect("") } {
    }

    // NOLINTBEGIN(misc-non-private-member-variables-in-classes), come on, its a test
    Node<TYPE> node;
    ServiceName service_name;
    PortFactoryEvent<TYPE> event;
    PortFactoryPublishSubscribe<TYPE, uint64_t, void> pubsub;

    Listener<TYPE> listener_1;
    Listener<TYPE> listener_2;
    Notifier<TYPE> notifier_1;
    Notifier<TYPE> notifier_2;
    Publisher<TYPE, uint64_t, void> publisher_1;
    Publisher<TYPE, uint64_t, void> publisher_2;
    Subscriber<TYPE, uint64_t, void> subscriber_1;
    Subscriber<TYPE, uint64_t, void> subscriber_2;
    // NOLINTEND(misc-non-private-member-variables-in-classes)
};

TYPED_TEST_SUITE(UniquePortIdTest, iox2_testing::ServiceTypes);

TYPED_TEST(UniquePortIdTest, unique_port_id_from_same_port_is_equal) {
    ASSERT_TRUE(this->listener_1.id() == this->listener_1.id());
    ASSERT_TRUE(this->notifier_1.id() == this->notifier_1.id());
    ASSERT_TRUE(this->publisher_1.id() == this->publisher_1.id());
    ASSERT_TRUE(this->subscriber_1.id() == this->subscriber_1.id());

    ASSERT_FALSE(this->listener_1.id() < this->listener_1.id());
    ASSERT_FALSE(this->notifier_1.id() < this->notifier_1.id());
    ASSERT_FALSE(this->publisher_1.id() < this->publisher_1.id());
    ASSERT_FALSE(this->subscriber_1.id() < this->subscriber_1.id());
}

TYPED_TEST(UniquePortIdTest, unique_port_id_from_different_ports_is_not_equal) {
    ASSERT_FALSE(this->listener_1.id() == this->listener_2.id());
    ASSERT_FALSE(this->notifier_1.id() == this->notifier_2.id());
    ASSERT_FALSE(this->publisher_1.id() == this->publisher_2.id());
    ASSERT_FALSE(this->subscriber_1.id() == this->subscriber_2.id());

    ASSERT_TRUE(this->listener_1.id() < this->listener_2.id() || this->listener_2.id() < this->listener_1.id());
    ASSERT_TRUE(this->notifier_1.id() < this->notifier_2.id() || this->notifier_2.id() < this->notifier_1.id());
    ASSERT_TRUE(this->publisher_1.id() < this->publisher_2.id() || this->publisher_2.id() < this->publisher_1.id());
    ASSERT_TRUE(this->subscriber_1.id() < this->subscriber_2.id() || this->subscriber_2.id() < this->subscriber_1.id());
}

TYPED_TEST(UniquePortIdTest, unique_port_id_identifies_origin) {
    auto sample_1 = this->publisher_1.loan().expect("");
    auto sample_2 = this->publisher_2.loan().expect("");

    ASSERT_TRUE(this->publisher_1.id() == sample_1.header().publisher_id());
    ASSERT_TRUE(this->publisher_2.id() == sample_2.header().publisher_id());

    send_sample(std::move(sample_1)).expect("");

    auto recv_sample_1 = this->subscriber_1.receive().expect("").value();
    ASSERT_TRUE(this->publisher_1.id() == recv_sample_1.header().publisher_id());
    ASSERT_TRUE(this->publisher_1.id() == recv_sample_1.origin());

    send_sample(std::move(sample_2)).expect("");

    auto recv_sample_2 = this->subscriber_1.receive().expect("").value();
    ASSERT_TRUE(this->publisher_2.id() == recv_sample_2.header().publisher_id());
    ASSERT_TRUE(this->publisher_2.id() == recv_sample_2.origin());
}
} // namespace
