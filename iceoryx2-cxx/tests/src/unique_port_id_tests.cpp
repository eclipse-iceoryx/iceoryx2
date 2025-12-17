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

#include "iox2/bb/static_vector.hpp"
#include "iox2/listener.hpp"
#include "iox2/node.hpp"
#include "iox2/node_name.hpp"
#include "iox2/notifier.hpp"
#include "iox2/publisher.hpp"
#include "iox2/reader.hpp"
#include "iox2/service_name.hpp"
#include "iox2/subscriber.hpp"
#include "iox2/unique_port_id.hpp"
#include "iox2/writer.hpp"

#include "test.hpp"

#include <gtest/gtest.h>

namespace {
using namespace iox2;

template <typename T>
struct UniquePortIdTest : public ::testing::Test {
    static constexpr ServiceType TYPE = T::TYPE;

    UniquePortIdTest()
        : node { NodeBuilder().create<TYPE>().value() }
        , service_name { iox2_testing::generate_service_name() }
        , event { node.service_builder(service_name).event().create().value() }
        , pubsub { node.service_builder(service_name).template publish_subscribe<uint64_t>().create().value() }
        , blackboard { node.service_builder(service_name)
                           .template blackboard_creator<uint64_t>()
                           .template add_with_default<uint64_t>(0)
                           .create()
                           .value() }
        , listener_1 { event.listener_builder().create().value() }
        , listener_2 { event.listener_builder().create().value() }
        , notifier_1 { event.notifier_builder().create().value() }
        , notifier_2 { event.notifier_builder().create().value() }
        , publisher_1 { pubsub.publisher_builder().create().value() }
        , publisher_2 { pubsub.publisher_builder().create().value() }
        , subscriber_1 { pubsub.subscriber_builder().create().value() }
        , subscriber_2 { pubsub.subscriber_builder().create().value() }
        , reader_1 { blackboard.reader_builder().create().value() }
        , reader_2 { blackboard.reader_builder().create().value() }
        , writer_1 { blackboard.writer_builder().create().value() } {
    }

    // NOLINTBEGIN(misc-non-private-member-variables-in-classes), come on, its a test
    Node<TYPE> node;
    ServiceName service_name;
    PortFactoryEvent<TYPE> event;
    PortFactoryPublishSubscribe<TYPE, uint64_t, void> pubsub;
    PortFactoryBlackboard<TYPE, uint64_t> blackboard;

    Listener<TYPE> listener_1;
    Listener<TYPE> listener_2;
    Notifier<TYPE> notifier_1;
    Notifier<TYPE> notifier_2;
    Publisher<TYPE, uint64_t, void> publisher_1;
    Publisher<TYPE, uint64_t, void> publisher_2;
    Subscriber<TYPE, uint64_t, void> subscriber_1;
    Subscriber<TYPE, uint64_t, void> subscriber_2;
    Reader<TYPE, uint64_t> reader_1;
    Reader<TYPE, uint64_t> reader_2;
    Writer<TYPE, uint64_t> writer_1;
    //  NOLINTEND(misc-non-private-member-variables-in-classes)
};

TYPED_TEST_SUITE(UniquePortIdTest, iox2_testing::ServiceTypes, );

TYPED_TEST(UniquePortIdTest, unique_port_id_value) {
    auto null_id =
        iox2::bb::StaticVector<uint8_t, iox2::UNIQUE_PORT_ID_LENGTH>::from_value<iox2::UNIQUE_PORT_ID_LENGTH>({});

    auto unique_publisher_id = this->publisher_1.id();
    ASSERT_TRUE(unique_publisher_id.bytes().has_value());
    ASSERT_NE(unique_publisher_id.bytes().value(), null_id);

    auto unique_subscriber_id = this->subscriber_1.id();
    ASSERT_TRUE(unique_subscriber_id.bytes().has_value());
    ASSERT_NE(unique_subscriber_id.bytes().value(), null_id);

    auto unique_notifier_id = this->notifier_1.id();
    ASSERT_TRUE(unique_notifier_id.bytes().has_value());
    ASSERT_NE(unique_notifier_id.bytes().value(), null_id);

    auto unique_listener_id = this->listener_1.id();
    ASSERT_TRUE(unique_listener_id.bytes().has_value());
    ASSERT_NE(unique_listener_id.bytes().value(), null_id);

    auto unique_reader_id = this->reader_1.id();
    ASSERT_TRUE(unique_reader_id.bytes().has_value());
    ASSERT_NE(unique_reader_id.bytes().value(), null_id);

    auto unique_writer_id = this->writer_1.id();
    ASSERT_TRUE(unique_writer_id.bytes().has_value());
    ASSERT_NE(unique_writer_id.bytes().value(), null_id);
}

TYPED_TEST(UniquePortIdTest, unique_port_id_from_same_port_is_equal) {
    ASSERT_TRUE(this->listener_1.id() == this->listener_1.id());
    ASSERT_TRUE(this->notifier_1.id() == this->notifier_1.id());
    ASSERT_TRUE(this->publisher_1.id() == this->publisher_1.id());
    ASSERT_TRUE(this->subscriber_1.id() == this->subscriber_1.id());
    ASSERT_TRUE(this->reader_1.id() == this->reader_1.id());
    ASSERT_TRUE(this->writer_1.id() == this->writer_1.id());

    ASSERT_FALSE(this->listener_1.id() < this->listener_1.id());
    ASSERT_FALSE(this->notifier_1.id() < this->notifier_1.id());
    ASSERT_FALSE(this->publisher_1.id() < this->publisher_1.id());
    ASSERT_FALSE(this->subscriber_1.id() < this->subscriber_1.id());
    ASSERT_FALSE(this->reader_1.id() < this->reader_1.id());
    ASSERT_FALSE(this->writer_1.id() < this->writer_1.id());
}

TYPED_TEST(UniquePortIdTest, unique_port_id_from_different_ports_is_not_equal) {
    ASSERT_FALSE(this->listener_1.id() == this->listener_2.id());
    ASSERT_FALSE(this->notifier_1.id() == this->notifier_2.id());
    ASSERT_FALSE(this->publisher_1.id() == this->publisher_2.id());
    ASSERT_FALSE(this->subscriber_1.id() == this->subscriber_2.id());
    ASSERT_FALSE(this->reader_1.id() == this->reader_2.id());

    ASSERT_TRUE(this->listener_1.id() < this->listener_2.id() || this->listener_2.id() < this->listener_1.id());
    ASSERT_TRUE(this->notifier_1.id() < this->notifier_2.id() || this->notifier_2.id() < this->notifier_1.id());
    ASSERT_TRUE(this->publisher_1.id() < this->publisher_2.id() || this->publisher_2.id() < this->publisher_1.id());
    ASSERT_TRUE(this->subscriber_1.id() < this->subscriber_2.id() || this->subscriber_2.id() < this->subscriber_1.id());
    ASSERT_TRUE(this->reader_1.id() < this->reader_2.id() || this->reader_2.id() < this->reader_1.id());
}

TYPED_TEST(UniquePortIdTest, unique_port_id_identifies_origin) {
    auto sample_1 = this->publisher_1.loan().value();
    auto sample_2 = this->publisher_2.loan().value();

    ASSERT_TRUE(this->publisher_1.id() == sample_1.header().publisher_id());
    ASSERT_TRUE(this->publisher_2.id() == sample_2.header().publisher_id());

    send(std::move(sample_1)).value();

    auto recv_sample_1 = this->subscriber_1.receive().value().value();
    ASSERT_TRUE(this->publisher_1.id() == recv_sample_1.header().publisher_id());
    ASSERT_TRUE(this->publisher_1.id() == recv_sample_1.origin());

    send(std::move(sample_2)).value();

    auto recv_sample_2 = this->subscriber_1.receive().value().value();
    ASSERT_TRUE(this->publisher_2.id() == recv_sample_2.header().publisher_id());
    ASSERT_TRUE(this->publisher_2.id() == recv_sample_2.origin());
}
} // namespace
