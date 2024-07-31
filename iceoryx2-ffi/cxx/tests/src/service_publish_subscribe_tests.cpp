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

#include "iox2/node.hpp"
#include "iox2/node_name.hpp"
#include "iox2/service.hpp"

#include "test.hpp"

namespace {
using namespace iox2;

template <typename T>
class ServicePublishSubscribeTest : public ::testing::Test {
  public:
    static constexpr ServiceType TYPE = T::TYPE;
};

TYPED_TEST_SUITE(ServicePublishSubscribeTest, iox2_testing::ServiceTypes);

TYPED_TEST(ServicePublishSubscribeTest, created_service_does_exist) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto* name_value = "I am floating through the galaxy of my brain. Oh the colors!";
    const auto service_name = ServiceName::create(name_value).expect("");

    ASSERT_FALSE(
        Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::PublishSubscribe)
            .expect(""));

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");

    {
        auto sut = node.service_builder(service_name).template publish_subscribe<uint64_t>().create().expect("");

        ASSERT_TRUE(
            Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::PublishSubscribe)
                .expect(""));
    }

    ASSERT_FALSE(
        Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::Event).expect(""));
}

TYPED_TEST(ServicePublishSubscribeTest, creating_existing_service_fails) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto* name_value = "I am floating through the galaxy of my brain. Oh the colors!";
    const auto service_name = ServiceName::create(name_value).expect("");

    ASSERT_FALSE(
        Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::PublishSubscribe)
            .expect(""));

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut = node.service_builder(service_name).template publish_subscribe<uint64_t>().create().expect("");
    auto sut_2 = node.service_builder(service_name).template publish_subscribe<uint64_t>().create();

    ASSERT_TRUE(sut_2.has_error());
    ASSERT_THAT(sut_2.error(), Eq(PublishSubscribeCreateError::AlreadyExists));
}

TYPED_TEST(ServicePublishSubscribeTest, open_or_create_service_does_exist) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto* name_value = "I am floating through the galaxy of my brain. Oh the colors!";
    const auto service_name = ServiceName::create(name_value).expect("");

    ASSERT_FALSE(
        Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::PublishSubscribe)
            .expect(""));

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");

    {
        auto sut = iox::optional<PortFactoryPublishSubscribe<SERVICE_TYPE, uint64_t, void>>(
            node.service_builder(service_name).template publish_subscribe<uint64_t>().open_or_create().expect(""));

        ASSERT_TRUE(
            Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::PublishSubscribe)
                .expect(""));

        auto sut_2 = iox::optional<PortFactoryPublishSubscribe<SERVICE_TYPE, uint64_t, void>>(
            node.service_builder(service_name).template publish_subscribe<uint64_t>().open_or_create().expect(""));

        ASSERT_TRUE(
            Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::PublishSubscribe)
                .expect(""));

        sut.reset();

        ASSERT_TRUE(
            Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::PublishSubscribe)
                .expect(""));

        sut_2.reset();
    }

    ASSERT_FALSE(
        Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::PublishSubscribe)
            .expect(""));
}

TYPED_TEST(ServicePublishSubscribeTest, opening_non_existing_service_fails) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto* name_value = "I am floating through the galaxy of my brain. Oh the colors!";
    const auto service_name = ServiceName::create(name_value).expect("");

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut = node.service_builder(service_name).template publish_subscribe<uint64_t>().open();
    ASSERT_TRUE(sut.has_error());
    ASSERT_THAT(sut.error(), Eq(PublishSubscribeOpenError::DoesNotExist));
}

TYPED_TEST(ServicePublishSubscribeTest, opening_existing_service_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto* name_value = "I am floating through the galaxy of my brain. Oh the colors!";
    const auto service_name = ServiceName::create(name_value).expect("");

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut_create = node.service_builder(service_name).template publish_subscribe<uint64_t>().create();
    auto sut = node.service_builder(service_name).template publish_subscribe<uint64_t>().open();
    ASSERT_TRUE(sut.has_value());
}

TYPED_TEST(ServicePublishSubscribeTest, opening_existing_service_with_wrong_payload_type_fails) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto* name_value = "I am floating through the galaxy of my brain. Oh the colors!";
    const auto service_name = ServiceName::create(name_value).expect("");

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut_create = node.service_builder(service_name).template publish_subscribe<uint64_t>().create();
    auto sut = node.service_builder(service_name).template publish_subscribe<double>().open();
    ASSERT_TRUE(sut.has_error());
    ASSERT_THAT(sut.error(), Eq(PublishSubscribeOpenError::IncompatibleTypes));
}

TYPED_TEST(ServicePublishSubscribeTest, open_or_create_existing_service_with_wrong_payload_type_fails) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto* name_value = "I am floating through the galaxy of my brain. Oh the colors!";
    const auto service_name = ServiceName::create(name_value).expect("");

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut_create = node.service_builder(service_name).template publish_subscribe<uint64_t>().create();
    auto sut = node.service_builder(service_name).template publish_subscribe<double>().open_or_create();
    ASSERT_TRUE(sut.has_error());
    ASSERT_THAT(sut.error(), Eq(PublishSubscribeOpenOrCreateError::OpenIncompatibleTypes));
}

TYPED_TEST(ServicePublishSubscribeTest, send_copy_receive_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto* name_value = "I am floating through the galaxy of my brain. Oh the colors!";
    const auto service_name = ServiceName::create(name_value).expect("");

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto service = node.service_builder(service_name).template publish_subscribe<uint64_t>().create().expect("");

    auto sut_publisher = service.publisher_builder().create().expect("");
    auto sut_subscriber = service.subscriber_builder().create().expect("");

    const uint64_t payload = 123;
    sut_publisher.send_copy(payload).expect("");
    auto sample = sut_subscriber.receive().expect("");

    ASSERT_TRUE(sample.has_value());
    ASSERT_THAT(**sample, Eq(payload));
}

TYPED_TEST(ServicePublishSubscribeTest, loan_uninit_send_receive_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto* name_value = "I am floating through the galaxy of my brain. Oh the colors!";
    const auto service_name = ServiceName::create(name_value).expect("");

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto service = node.service_builder(service_name).template publish_subscribe<uint64_t>().create().expect("");

    auto sut_publisher = service.publisher_builder().create().expect("");
    auto sut_subscriber = service.subscriber_builder().create().expect("");

    auto sample = sut_publisher.loan_uninit().expect("");
    const uint64_t payload = 78123791;
    sample.write_payload(payload);
    send_sample(std::move(sample)).expect("");
    auto recv_sample = sut_subscriber.receive().expect("");

    ASSERT_TRUE(recv_sample.has_value());
    ASSERT_THAT(**recv_sample, Eq(payload));
}

TYPED_TEST(ServicePublishSubscribeTest, loan_send_receive_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto* name_value = "I am floating through the galaxy of my brain. Oh the colors!";
    const auto service_name = ServiceName::create(name_value).expect("");

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto service = node.service_builder(service_name).template publish_subscribe<uint64_t>().create().expect("");

    auto sut_publisher = service.publisher_builder().create().expect("");
    auto sut_subscriber = service.subscriber_builder().create().expect("");

    auto sample = sut_publisher.loan().expect("");
    const uint64_t payload = 781891729871;
    *sample = payload;
    send_sample(std::move(sample)).expect("");
    auto recv_sample = sut_subscriber.receive().expect("");

    ASSERT_TRUE(recv_sample.has_value());
    ASSERT_THAT(**recv_sample, Eq(payload));
}

TYPED_TEST(ServicePublishSubscribeTest, setting_service_properties_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    constexpr uint64_t NUMBER_OF_PUBLISHERS = 11;
    constexpr uint64_t NUMBER_OF_SUBSCRIBERS = 12;

    const auto* name_value = "I am floating through the galaxy of my brain. Oh the colors!";
    const auto service_name = ServiceName::create(name_value).expect("");

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto service = node.service_builder(service_name)
                       .template publish_subscribe<uint64_t>()
                       .max_publishers(NUMBER_OF_PUBLISHERS)
                       .max_subscribers(NUMBER_OF_SUBSCRIBERS)
                       .create()
                       .expect("");

    auto static_config = service.static_config();

    ASSERT_THAT(static_config.max_publishers(), Eq(NUMBER_OF_PUBLISHERS));
    ASSERT_THAT(static_config.max_subscribers(), Eq(NUMBER_OF_SUBSCRIBERS));
    ASSERT_THAT(static_config.message_type_details().payload().size(), Eq(sizeof(uint64_t)));
    ASSERT_THAT(static_config.message_type_details().payload().alignment(), Eq(alignof(uint64_t)));
    ASSERT_THAT(static_config.message_type_details().payload().type_name(), StrEq(typeid(uint64_t).name()));
}

TYPED_TEST(ServicePublishSubscribeTest, open_fails_with_incompatible_publisher_requirement) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    constexpr uint64_t NUMBER_OF_PUBLISHERS = 11;

    const auto* name_value = "I am floating through the galaxy of my brain. Oh the colors!";
    const auto service_name = ServiceName::create(name_value).expect("");

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto service = node.service_builder(service_name)
                       .template publish_subscribe<uint64_t>()
                       .max_publishers(NUMBER_OF_PUBLISHERS)
                       .create()
                       .expect("");

    auto service_fail = node.service_builder(service_name)
                            .template publish_subscribe<uint64_t>()
                            .max_publishers(NUMBER_OF_PUBLISHERS + 1)
                            .open();

    ASSERT_TRUE(service_fail.has_error());
    ASSERT_THAT(service_fail.error(), Eq(PublishSubscribeOpenError::DoesNotSupportRequestedAmountOfPublishers));
}

TYPED_TEST(ServicePublishSubscribeTest, open_fails_with_incompatible_subscriber_requirement) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    constexpr uint64_t NUMBER_OF_SUBSCRIBERS = 12;

    const auto* name_value = "I am floating through the galaxy of my brain. Oh the colors!";
    const auto service_name = ServiceName::create(name_value).expect("");

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto service = node.service_builder(service_name)
                       .template publish_subscribe<uint64_t>()
                       .max_subscribers(NUMBER_OF_SUBSCRIBERS)
                       .create()
                       .expect("");

    auto service_fail = node.service_builder(service_name)
                            .template publish_subscribe<uint64_t>()
                            .max_subscribers(NUMBER_OF_SUBSCRIBERS + 1)
                            .open();

    ASSERT_TRUE(service_fail.has_error());
    ASSERT_THAT(service_fail.error(), Eq(PublishSubscribeOpenError::DoesNotSupportRequestedAmountOfSubscribers));
}

TYPED_TEST(ServicePublishSubscribeTest, publisher_applies_unable_to_deliver_strategy) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto* name_value = "I am floating through the galaxy of my brain. Oh the colors!";
    const auto service_name = ServiceName::create(name_value).expect("");

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto service = node.service_builder(service_name).template publish_subscribe<uint64_t>().create().expect("");

    auto sut_pub_1 =
        service.publisher_builder().unable_to_deliver_strategy(UnableToDeliverStrategy::Block).create().expect("");
    auto sut_pub_2 = service.publisher_builder()
                         .unable_to_deliver_strategy(UnableToDeliverStrategy::DiscardSample)
                         .create()
                         .expect("");

    ASSERT_THAT(sut_pub_1.unable_to_deliver_strategy(), Eq(UnableToDeliverStrategy::Block));
    ASSERT_THAT(sut_pub_2.unable_to_deliver_strategy(), Eq(UnableToDeliverStrategy::DiscardSample));
}
} // namespace
