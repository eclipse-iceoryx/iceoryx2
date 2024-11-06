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

#include "iox/uninitialized_array.hpp"
#include "iox2/node.hpp"
#include "iox2/node_name.hpp"
#include "iox2/service.hpp"

#include "test.hpp"
#include <array>

namespace {
using namespace iox2;

struct TestHeader {
    static constexpr uint64_t CAPACITY = 1024;
    std::array<uint64_t, CAPACITY> value;
};

template <typename T>
class ServicePublishSubscribeTest : public ::testing::Test {
  public:
    static constexpr ServiceType TYPE = T::TYPE;
};

TYPED_TEST_SUITE(ServicePublishSubscribeTest, iox2_testing::ServiceTypes);

TYPED_TEST(ServicePublishSubscribeTest, created_service_does_exist) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

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

    const auto service_name = iox2_testing::generate_service_name();

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

    const auto service_name = iox2_testing::generate_service_name();

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

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut = node.service_builder(service_name).template publish_subscribe<uint64_t>().open();
    ASSERT_TRUE(sut.has_error());
    ASSERT_THAT(sut.error(), Eq(PublishSubscribeOpenError::DoesNotExist));
}

TYPED_TEST(ServicePublishSubscribeTest, opening_existing_service_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut_create = node.service_builder(service_name).template publish_subscribe<uint64_t>().create();
    auto sut = node.service_builder(service_name).template publish_subscribe<uint64_t>().open();
    ASSERT_TRUE(sut.has_value());
}

TYPED_TEST(ServicePublishSubscribeTest, opening_existing_service_with_wrong_payload_type_fails) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut_create = node.service_builder(service_name).template publish_subscribe<uint64_t>().create();
    auto sut = node.service_builder(service_name).template publish_subscribe<double>().open();
    ASSERT_TRUE(sut.has_error());
    ASSERT_THAT(sut.error(), Eq(PublishSubscribeOpenError::IncompatibleTypes));
}

TYPED_TEST(ServicePublishSubscribeTest, open_or_create_existing_service_with_wrong_payload_type_fails) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut_create = node.service_builder(service_name).template publish_subscribe<uint64_t>().create();
    auto sut = node.service_builder(service_name).template publish_subscribe<double>().open_or_create();
    ASSERT_TRUE(sut.has_error());
    ASSERT_THAT(sut.error(), Eq(PublishSubscribeOpenOrCreateError::OpenIncompatibleTypes));
}

TYPED_TEST(ServicePublishSubscribeTest, send_copy_receive_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

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

TYPED_TEST(ServicePublishSubscribeTest, loan_send_receive_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto service = node.service_builder(service_name).template publish_subscribe<uint64_t>().create().expect("");

    auto sut_publisher = service.publisher_builder().create().expect("");
    auto sut_subscriber = service.subscriber_builder().create().expect("");

    auto sample = sut_publisher.loan().expect("");
    const uint64_t payload = 781891729871;
    *sample = payload;
    send(std::move(sample)).expect("");
    auto recv_sample = sut_subscriber.receive().expect("");

    ASSERT_TRUE(recv_sample.has_value());
    ASSERT_THAT(**recv_sample, Eq(payload));
}

TYPED_TEST(ServicePublishSubscribeTest, loan_uninit_send_receive_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto service = node.service_builder(service_name).template publish_subscribe<uint64_t>().create().expect("");

    auto sut_publisher = service.publisher_builder().create().expect("");
    auto sut_subscriber = service.subscriber_builder().create().expect("");

    auto sample = sut_publisher.loan_uninit().expect("");
    const uint64_t payload = 78123791;
    sample.write_payload(payload);
    send(assume_init(std::move(sample))).expect("");
    auto recv_sample = sut_subscriber.receive().expect("");

    ASSERT_TRUE(recv_sample.has_value());
    ASSERT_THAT(**recv_sample, Eq(payload));
}

struct DummyData {
    static constexpr uint64_t DEFAULT_VALUE_A = 42;
    static constexpr bool DEFAULT_VALUE_Z { false };
    uint64_t a { DEFAULT_VALUE_A };
    bool z { DEFAULT_VALUE_Z };
};

// NOLINTBEGIN(readability-function-cognitive-complexity) : Cognitive complexity of 26 (+1) is OK. Test case is complex.
TYPED_TEST(ServicePublishSubscribeTest, slice_copy_send_receive_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    constexpr auto SLICE_MAX_LENGTH = 10;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto service =
        node.service_builder(service_name).template publish_subscribe<iox::Slice<DummyData>>().create().expect("");

    auto sut_publisher = service.publisher_builder().max_slice_len(SLICE_MAX_LENGTH).create().expect("");
    auto sut_subscriber = service.subscriber_builder().create().expect("");

    iox::UninitializedArray<DummyData, SLICE_MAX_LENGTH, iox::ZeroedBuffer> elements;
    for (auto& item : elements) {
        new (&item) DummyData {};
    }
    auto payload = iox::ImmutableSlice<DummyData>(elements.begin(), SLICE_MAX_LENGTH);
    sut_publisher.send_slice_copy(payload).expect("");

    auto recv_result = sut_subscriber.receive().expect("");
    ASSERT_TRUE(recv_result.has_value());
    auto recv_sample = std::move(recv_result.value());

    auto iterations = 0;
    for (const auto& item : recv_sample.payload()) {
        ASSERT_THAT(item.a, Eq(DummyData::DEFAULT_VALUE_A));
        ASSERT_THAT(item.z, Eq(DummyData::DEFAULT_VALUE_Z));
        ++iterations;
    }

    ASSERT_THAT(recv_sample.payload().number_of_elements(), Eq(SLICE_MAX_LENGTH));
    ASSERT_THAT(iterations, Eq(SLICE_MAX_LENGTH));
}
// NOLINTEND(readability-function-cognitive-complexity)

TYPED_TEST(ServicePublishSubscribeTest, loan_slice_send_receive_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    constexpr uint64_t PAYLOAD_ALIGNMENT = 8;
    constexpr auto SLICE_MAX_LENGTH = 10;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto service = node.service_builder(service_name)
                       .template publish_subscribe<iox::Slice<DummyData>>()
                       .payload_alignment(PAYLOAD_ALIGNMENT)
                       .create()
                       .expect("");

    auto sut_publisher = service.publisher_builder().max_slice_len(SLICE_MAX_LENGTH).create().expect("");
    auto sut_subscriber = service.subscriber_builder().create().expect("");

    auto send_sample = sut_publisher.loan_slice(SLICE_MAX_LENGTH).expect("");

    send(std::move(send_sample)).expect("");

    auto recv_result = sut_subscriber.receive().expect("");
    ASSERT_TRUE(recv_result.has_value());
    auto recv_sample = std::move(recv_result.value());

    auto iterations = 0;
    for (const auto& item : recv_sample.payload()) {
        ASSERT_THAT(item.a, Eq(DummyData::DEFAULT_VALUE_A));
        ASSERT_THAT(item.z, Eq(DummyData::DEFAULT_VALUE_Z));
        ++iterations;
    }

    ASSERT_THAT(recv_sample.payload().number_of_elements(), Eq(SLICE_MAX_LENGTH));
    ASSERT_THAT(iterations, Eq(SLICE_MAX_LENGTH));
}

// NOLINTBEGIN(readability-function-cognitive-complexity) : Cognitive complexity of 26 (+1) is OK. Test case is complex.
TYPED_TEST(ServicePublishSubscribeTest, loan_slice_uninit_send_receive_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    constexpr uint64_t PAYLOAD_ALIGNMENT = 8;
    constexpr auto SLICE_MAX_LENGTH = 10;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto service = node.service_builder(service_name)
                       .template publish_subscribe<iox::Slice<DummyData>>()
                       .payload_alignment(PAYLOAD_ALIGNMENT)
                       .create()
                       .expect("");

    auto sut_publisher = service.publisher_builder().max_slice_len(SLICE_MAX_LENGTH).create().expect("");
    auto sut_subscriber = service.subscriber_builder().create().expect("");

    auto send_sample = sut_publisher.loan_slice_uninit(SLICE_MAX_LENGTH).expect("");

    auto iterations = 0;
    for (auto& item : send_sample.payload_mut()) {
        new (&item) DummyData { DummyData::DEFAULT_VALUE_A + iterations, iterations % 2 == 0 };
        ++iterations;
    }

    send(assume_init(std::move(send_sample))).expect("");

    auto recv_result = sut_subscriber.receive().expect("");
    ASSERT_TRUE(recv_result.has_value());
    auto recv_sample = std::move(recv_result.value());

    iterations = 0;
    for (const auto& item : recv_sample.payload()) {
        ASSERT_THAT(item.a, Eq(DummyData::DEFAULT_VALUE_A + iterations));
        ASSERT_THAT(item.z, Eq(iterations % 2 == 0));
        ++iterations;
    }

    ASSERT_THAT(recv_sample.payload().number_of_elements(), Eq(SLICE_MAX_LENGTH));
    ASSERT_THAT(iterations, Eq(SLICE_MAX_LENGTH));
}
// NOLINTEND(readability-function-cognitive-complexity)

TYPED_TEST(ServicePublishSubscribeTest, loan_slice_uninit_with_bytes_send_receive_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    constexpr uint64_t PAYLOAD_ALIGNMENT = 8;
    constexpr auto SLICE_MAX_LENGTH = 10;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto service = node.service_builder(service_name)
                       .template publish_subscribe<iox::Slice<uint8_t>>()
                       .payload_alignment(PAYLOAD_ALIGNMENT)
                       .create()
                       .expect("");

    auto sut_publisher = service.publisher_builder().max_slice_len(sizeof(DummyData)).create().expect("");
    auto sut_subscriber = service.subscriber_builder().create().expect("");

    auto send_sample = sut_publisher.loan_slice_uninit(sizeof(DummyData)).expect("");

    new (send_sample.payload_mut().data()) DummyData {};

    send(assume_init(std::move(send_sample))).expect("");

    auto recv_result = sut_subscriber.receive().expect("");
    ASSERT_TRUE(recv_result.has_value());

    auto recv_sample = std::move(recv_result.value());
    ASSERT_THAT(recv_sample.payload().number_of_elements(), Eq(sizeof(DummyData)));
    const auto* recv_data = reinterpret_cast<const DummyData*>(recv_sample.payload().data()); // NOLINT

    ASSERT_THAT(recv_data->a, Eq(DummyData::DEFAULT_VALUE_A));
    ASSERT_THAT(recv_data->z, Eq(DummyData::DEFAULT_VALUE_Z));
}

TYPED_TEST(ServicePublishSubscribeTest, write_from_fn_send_receive_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    constexpr auto SLICE_MAX_LENGTH = 10;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto service =
        node.service_builder(service_name).template publish_subscribe<iox::Slice<DummyData>>().create().expect("");

    auto sut_publisher = service.publisher_builder().max_slice_len(SLICE_MAX_LENGTH).create().expect("");
    auto sut_subscriber = service.subscriber_builder().create().expect("");

    auto send_sample = sut_publisher.loan_slice_uninit(SLICE_MAX_LENGTH).expect("");
    send_sample.write_from_fn(
        [](auto index) { return DummyData { DummyData::DEFAULT_VALUE_A + index, index % 2 == 0 }; });
    send(assume_init(std::move(send_sample))).expect("");

    auto recv_result = sut_subscriber.receive().expect("");
    ASSERT_TRUE(recv_result.has_value());
    auto recv_sample = std::move(recv_result.value());

    auto iterations = 0;
    for (const auto& item : recv_sample.payload()) {
        ASSERT_THAT(item.a, Eq(DummyData::DEFAULT_VALUE_A + iterations));
        ASSERT_THAT(item.z, Eq(iterations % 2 == 0));
        ++iterations;
    }

    ASSERT_THAT(recv_sample.payload().number_of_elements(), Eq(SLICE_MAX_LENGTH));
    ASSERT_THAT(iterations, Eq(SLICE_MAX_LENGTH));
}

// NOLINTBEGIN(readability-function-cognitive-complexity)
TYPED_TEST(ServicePublishSubscribeTest, write_from_slice_send_receive_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    constexpr auto SLICE_MAX_LENGTH = 10;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto service =
        node.service_builder(service_name).template publish_subscribe<iox::Slice<DummyData>>().create().expect("");

    auto sut_publisher = service.publisher_builder().max_slice_len(SLICE_MAX_LENGTH).create().expect("");
    auto sut_subscriber = service.subscriber_builder().create().expect("");

    iox::UninitializedArray<DummyData, SLICE_MAX_LENGTH, iox::ZeroedBuffer> elements;
    for (auto& item : elements) {
        new (&item) DummyData {};
    }
    auto payload = iox::ImmutableSlice<DummyData>(elements.begin(), SLICE_MAX_LENGTH);
    auto send_sample = sut_publisher.loan_slice_uninit(SLICE_MAX_LENGTH).expect("");
    send_sample.write_from_slice(payload);
    send(assume_init(std::move(send_sample))).expect("");

    auto recv_result = sut_subscriber.receive().expect("");
    ASSERT_TRUE(recv_result.has_value());
    auto recv_sample = std::move(recv_result.value());

    auto iterations = 0;
    for (const auto& item : recv_sample.payload()) {
        ASSERT_THAT(item.a, Eq(DummyData::DEFAULT_VALUE_A));
        ASSERT_THAT(item.z, Eq(DummyData::DEFAULT_VALUE_Z));
        ++iterations;
    }

    ASSERT_THAT(recv_sample.payload().number_of_elements(), Eq(SLICE_MAX_LENGTH));
    ASSERT_THAT(iterations, Eq(SLICE_MAX_LENGTH));
}
// NOLINTEND(readability-function-cognitive-complexity)

TYPED_TEST(ServicePublishSubscribeTest, update_connections_delivers_history) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto service = node.service_builder(service_name).template publish_subscribe<uint64_t>().create().expect("");

    auto sut_publisher = service.publisher_builder().create().expect("");
    const uint64_t payload = 123;
    sut_publisher.send_copy(payload).expect("");

    auto sut_subscriber = service.subscriber_builder().create().expect("");
    auto sample = sut_subscriber.receive().expect("");

    ASSERT_FALSE(sample.has_value());

    ASSERT_TRUE(sut_publisher.update_connections().has_value());
    sample = sut_subscriber.receive().expect("");

    ASSERT_TRUE(sample.has_value());
    ASSERT_THAT(**sample, Eq(payload));
}

TYPED_TEST(ServicePublishSubscribeTest, setting_service_properties_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    constexpr uint64_t NUMBER_OF_NODES = 10;
    constexpr uint64_t NUMBER_OF_PUBLISHERS = 11;
    constexpr uint64_t NUMBER_OF_SUBSCRIBERS = 12;
    constexpr uint64_t HISTORY_SIZE = 13;
    constexpr uint64_t SUBSCRIBER_MAX_BUFFER_SIZE = 14;
    constexpr uint64_t SUBSCRIBER_MAX_BORROWED_SAMPLES = 15;
    constexpr uint64_t PAYLOAD_ALIGNMENT = 4;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto service = node.service_builder(service_name)
                       .template publish_subscribe<uint64_t>()
                       .max_nodes(NUMBER_OF_NODES)
                       .max_publishers(NUMBER_OF_PUBLISHERS)
                       .max_subscribers(NUMBER_OF_SUBSCRIBERS)
                       .history_size(HISTORY_SIZE)
                       .subscriber_max_buffer_size(SUBSCRIBER_MAX_BUFFER_SIZE)
                       .subscriber_max_borrowed_samples(SUBSCRIBER_MAX_BORROWED_SAMPLES)
                       .payload_alignment(PAYLOAD_ALIGNMENT)
                       .create()
                       .expect("");

    auto static_config = service.static_config();

    ASSERT_THAT(static_config.max_nodes(), Eq(NUMBER_OF_NODES));
    ASSERT_THAT(static_config.max_publishers(), Eq(NUMBER_OF_PUBLISHERS));
    ASSERT_THAT(static_config.max_subscribers(), Eq(NUMBER_OF_SUBSCRIBERS));
    ASSERT_THAT(static_config.history_size(), Eq(HISTORY_SIZE));
    ASSERT_THAT(static_config.subscriber_max_buffer_size(), Eq(SUBSCRIBER_MAX_BUFFER_SIZE));
    ASSERT_THAT(static_config.subscriber_max_borrowed_samples(), Eq(SUBSCRIBER_MAX_BORROWED_SAMPLES));
    ASSERT_THAT(static_config.message_type_details().payload().size(), Eq(sizeof(uint64_t)));
    ASSERT_THAT(static_config.message_type_details().payload().alignment(), Eq(alignof(uint64_t)));
    ASSERT_THAT(static_config.message_type_details().payload().type_name(), StrEq(typeid(uint64_t).name()));
}

TYPED_TEST(ServicePublishSubscribeTest, safe_overflow_can_be_set) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");

    for (auto has_safe_overflow : { true, false }) {
        auto service = node.service_builder(service_name)
                           .template publish_subscribe<uint64_t>()
                           .enable_safe_overflow(has_safe_overflow)
                           .create()
                           .expect("");

        auto static_config = service.static_config();

        ASSERT_THAT(static_config.has_safe_overflow(), Eq(has_safe_overflow));
    }
}

TYPED_TEST(ServicePublishSubscribeTest, open_fails_with_incompatible_publisher_requirement) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    constexpr uint64_t NUMBER_OF_PUBLISHERS = 11;

    const auto service_name = iox2_testing::generate_service_name();

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

    const auto service_name = iox2_testing::generate_service_name();

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

    const auto service_name = iox2_testing::generate_service_name();

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

TYPED_TEST(ServicePublishSubscribeTest, publisher_applies_max_slice_len) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    constexpr uint64_t DESIRED_MAX_SLICE_LEN = 10;
    using ValueType = uint8_t;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto service =
        node.service_builder(service_name).template publish_subscribe<iox::Slice<ValueType>>().create().expect("");

    auto sut = service.publisher_builder().max_slice_len(DESIRED_MAX_SLICE_LEN).create().expect("");

    ASSERT_THAT(sut.max_slice_len(), Eq(DESIRED_MAX_SLICE_LEN));
}

TYPED_TEST(ServicePublishSubscribeTest, send_receive_with_user_header_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto service_pub = node.service_builder(service_name)
                           .template publish_subscribe<uint64_t>()
                           .template user_header<TestHeader>()
                           .create()
                           .expect("");
    auto service_sub = node.service_builder(service_name)
                           .template publish_subscribe<uint64_t>()
                           .template user_header<TestHeader>()
                           .open()
                           .expect("");

    auto sut_publisher = service_pub.publisher_builder().create().expect("");
    auto sut_subscriber = service_sub.subscriber_builder().create().expect("");

    auto sample = sut_publisher.loan().expect("");
    const uint64_t payload = 781891729871;
    *sample = payload;
    for (uint64_t idx = 0; idx < TestHeader::CAPACITY; ++idx) {
        sample.user_header_mut().value.at(idx) = 4 * idx + 3;
    }
    send(std::move(sample)).expect("");
    auto recv_sample = sut_subscriber.receive().expect("");

    ASSERT_TRUE(recv_sample.has_value());
    ASSERT_THAT(**recv_sample, Eq(payload));

    for (uint64_t idx = 0; idx < TestHeader::CAPACITY; ++idx) {
        ASSERT_THAT(recv_sample->user_header().value.at(idx), Eq(4 * idx + 3));
    }
}

TYPED_TEST(ServicePublishSubscribeTest, has_sample_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto service = node.service_builder(service_name).template publish_subscribe<uint64_t>().create().expect("");

    auto sut_publisher = service.publisher_builder().create().expect("");
    auto sut_subscriber = service.subscriber_builder().create().expect("");

    ASSERT_FALSE(*sut_subscriber.has_samples());

    const uint64_t payload = 123;
    sut_publisher.send_copy(payload).expect("");
    ASSERT_TRUE(*sut_subscriber.has_samples());
    auto sample = sut_subscriber.receive().expect("");
    ASSERT_FALSE(*sut_subscriber.has_samples());
}

TYPED_TEST(ServicePublishSubscribeTest, service_can_be_opened_when_there_is_a_publisher) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    const uint64_t payload = 9871273;
    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut = iox::optional<PortFactoryPublishSubscribe<SERVICE_TYPE, uint64_t, void>>(
        node.service_builder(service_name).template publish_subscribe<uint64_t>().create().expect(""));
    auto subscriber =
        iox::optional<Subscriber<SERVICE_TYPE, uint64_t, void>>(sut->subscriber_builder().create().expect(""));
    auto publisher =
        iox::optional<Publisher<SERVICE_TYPE, uint64_t, void>>(sut->publisher_builder().create().expect(""));

    sut.reset();
    {
        auto temp_sut = node.service_builder(service_name).template publish_subscribe<uint64_t>().open();
        ASSERT_THAT(temp_sut.has_value(), Eq(true));
    }
    {
        auto temp_sut = node.service_builder(service_name).template publish_subscribe<uint64_t>().create();
        ASSERT_THAT(temp_sut.error(), Eq(PublishSubscribeCreateError::AlreadyExists));
    }
    subscriber.reset();

    sut = iox::optional<PortFactoryPublishSubscribe<SERVICE_TYPE, uint64_t, void>>(
        node.service_builder(service_name).template publish_subscribe<uint64_t>().open().expect(""));
    subscriber = iox::optional<Subscriber<SERVICE_TYPE, uint64_t, void>>(sut->subscriber_builder().create().expect(""));
    publisher->send_copy(payload).expect("");
    auto sample = subscriber->receive().expect("");
    ASSERT_THAT(sample->payload(), Eq(payload));

    subscriber.reset();
    sut.reset();
    publisher.reset();

    {
        auto temp_sut = node.service_builder(service_name).template publish_subscribe<uint64_t>().open();
        ASSERT_THAT(temp_sut.error(), Eq(PublishSubscribeOpenError::DoesNotExist));
    }
    {
        auto temp_sut = node.service_builder(service_name).template publish_subscribe<uint64_t>().create();
        ASSERT_THAT(temp_sut.has_value(), Eq(true));
    }
}

TYPED_TEST(ServicePublishSubscribeTest, service_can_be_opened_when_there_is_a_subscriber) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    const uint64_t payload = 57812;
    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut = iox::optional<PortFactoryPublishSubscribe<SERVICE_TYPE, uint64_t, void>>(
        node.service_builder(service_name).template publish_subscribe<uint64_t>().create().expect(""));
    auto subscriber =
        iox::optional<Subscriber<SERVICE_TYPE, uint64_t, void>>(sut->subscriber_builder().create().expect(""));
    auto publisher =
        iox::optional<Publisher<SERVICE_TYPE, uint64_t, void>>(sut->publisher_builder().create().expect(""));

    sut.reset();
    {
        auto temp_sut = node.service_builder(service_name).template publish_subscribe<uint64_t>().open();
        ASSERT_THAT(temp_sut.has_value(), Eq(true));
    }
    {
        auto temp_sut = node.service_builder(service_name).template publish_subscribe<uint64_t>().create();
        ASSERT_THAT(temp_sut.error(), Eq(PublishSubscribeCreateError::AlreadyExists));
    }
    publisher.reset();

    sut = iox::optional<PortFactoryPublishSubscribe<SERVICE_TYPE, uint64_t, void>>(
        node.service_builder(service_name).template publish_subscribe<uint64_t>().open().expect(""));
    publisher = iox::optional<Publisher<SERVICE_TYPE, uint64_t, void>>(sut->publisher_builder().create().expect(""));
    publisher->send_copy(payload).expect("");
    auto sample = subscriber->receive().expect("");
    ASSERT_THAT(sample->payload(), Eq(payload));

    publisher.reset();
    sut.reset();
    subscriber.reset();

    {
        auto temp_sut = node.service_builder(service_name).template publish_subscribe<uint64_t>().open();
        ASSERT_THAT(temp_sut.error(), Eq(PublishSubscribeOpenError::DoesNotExist));
    }
    {
        auto temp_sut = node.service_builder(service_name).template publish_subscribe<uint64_t>().create();
        ASSERT_THAT(temp_sut.has_value(), Eq(true));
    }
}

} // namespace
