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
#include "iox2/service.hpp"

#include "test.hpp"
#include <array>

namespace {
using namespace iox2;

struct TestHeader {
    static constexpr uint64_t CAPACITY = 1024;
    std::array<uint64_t, CAPACITY> value;
};

template <uint64_t A, uint32_t B>
class CustomTestHeader {
  public:
    CustomTestHeader()
        : m_data_a { A }
        , m_data_b { B } {
    }

    auto operator==(const CustomTestHeader& rhs) const -> bool {
        return m_data_a == rhs.m_data_a && m_data_b == rhs.m_data_b;
    }

  private:
    uint64_t m_data_a;
    uint64_t m_data_b;
};

template <typename T>
class ServicePublishSubscribeTest : public ::testing::Test {
  public:
    static constexpr ServiceType TYPE = T::TYPE;
};

TYPED_TEST_SUITE(ServicePublishSubscribeTest, iox2_testing::ServiceTypes, );

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

TYPED_TEST(ServicePublishSubscribeTest, service_name_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut = node.service_builder(service_name).template publish_subscribe<uint64_t>().create().expect("");

    ASSERT_THAT(sut.name().to_string().c_str(), StrEq(service_name.to_string().c_str()));
}

//NOLINTBEGIN(readability-function-cognitive-complexity), false positive caused by ASSERT_THAT
TYPED_TEST(ServicePublishSubscribeTest, list_service_nodes_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto node_name_1 = NodeName::create("nala is hungry").expect("");
    const auto node_name_2 = NodeName::create("maybe octo-wolf can help?").expect("");
    const auto service_name = iox2_testing::generate_service_name();

    auto node_1 = NodeBuilder().name(node_name_1).create<SERVICE_TYPE>().expect("");
    auto node_2 = NodeBuilder().name(node_name_2).create<SERVICE_TYPE>().expect("");

    auto sut_1 = node_1.service_builder(service_name).template publish_subscribe<uint64_t>().create().expect("");
    auto sut_2 = node_2.service_builder(service_name).template publish_subscribe<uint64_t>().open().expect("");

    auto counter = 0;
    auto verify_node = [&](const AliveNodeView<SERVICE_TYPE>& node_view) {
        counter++;
        if (node_view.id() == node_1.id()) {
            ASSERT_THAT(node_view.details()->name().to_string().c_str(), StrEq(node_1.name().to_string().c_str()));
        } else {
            ASSERT_THAT(node_view.details()->name().to_string().c_str(), StrEq(node_2.name().to_string().c_str()));
        }
    };

    auto result = sut_1.nodes([&](auto node_state) -> CallbackProgression {
        node_state.alive(verify_node);

        node_state.dead([](const auto&) { ASSERT_TRUE(false); });
        node_state.inaccessible([](const auto&) { ASSERT_TRUE(false); });
        node_state.undefined([](const auto&) { ASSERT_TRUE(false); });

        return CallbackProgression::Continue;
    });

    ASSERT_THAT(result.has_value(), Eq(true));
    ASSERT_THAT(counter, Eq(2));
}
//NOLINTEND(readability-function-cognitive-complexity)

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
    auto sut_create = node.service_builder(service_name).template publish_subscribe<uint64_t>().create().expect("");
    auto sut = node.service_builder(service_name).template publish_subscribe<uint64_t>().open();
    ASSERT_TRUE(sut.has_value());
}

TYPED_TEST(ServicePublishSubscribeTest, opening_existing_service_with_wrong_payload_type_fails) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut_create = node.service_builder(service_name).template publish_subscribe<uint64_t>().create().expect("");
    auto sut = node.service_builder(service_name).template publish_subscribe<double>().open();
    ASSERT_TRUE(sut.has_error());
    ASSERT_THAT(sut.error(), Eq(PublishSubscribeOpenError::IncompatibleTypes));
}

TYPED_TEST(ServicePublishSubscribeTest, open_or_create_existing_service_with_wrong_payload_type_fails) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut_create = node.service_builder(service_name).template publish_subscribe<uint64_t>().create().expect("");
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

    auto sample_uninit = sut_publisher.loan_uninit().expect("");
    const uint64_t payload = 78123791;
    auto sample = sample_uninit.write_payload(payload);
    send(std::move(sample)).expect("");
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

    auto sut_publisher = service.publisher_builder().initial_max_slice_len(SLICE_MAX_LENGTH).create().expect("");
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

    auto sut_publisher = service.publisher_builder().initial_max_slice_len(SLICE_MAX_LENGTH).create().expect("");
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

TYPED_TEST(ServicePublishSubscribeTest, number_of_publishers_subscribers_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto service = node.service_builder(service_name).template publish_subscribe<uint64_t>().create().expect("");

    ASSERT_THAT(service.dynamic_config().number_of_publishers(), Eq(0));
    ASSERT_THAT(service.dynamic_config().number_of_subscribers(), Eq(0));

    {
        auto sut_publisher = service.publisher_builder().create().expect("");
        ASSERT_THAT(service.dynamic_config().number_of_publishers(), Eq(1));
        ASSERT_THAT(service.dynamic_config().number_of_subscribers(), Eq(0));

        auto sut_subscriber = service.subscriber_builder().create().expect("");
        ASSERT_THAT(service.dynamic_config().number_of_publishers(), Eq(1));
        ASSERT_THAT(service.dynamic_config().number_of_subscribers(), Eq(1));
    }

    ASSERT_THAT(service.dynamic_config().number_of_publishers(), Eq(0));
    ASSERT_THAT(service.dynamic_config().number_of_subscribers(), Eq(0));
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

    auto sut_publisher = service.publisher_builder().initial_max_slice_len(SLICE_MAX_LENGTH).create().expect("");
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

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto service = node.service_builder(service_name)
                       .template publish_subscribe<iox::Slice<uint8_t>>()
                       .payload_alignment(PAYLOAD_ALIGNMENT)
                       .create()
                       .expect("");

    auto sut_publisher = service.publisher_builder().initial_max_slice_len(sizeof(DummyData)).create().expect("");
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

    auto sut_publisher = service.publisher_builder().initial_max_slice_len(SLICE_MAX_LENGTH).create().expect("");
    auto sut_subscriber = service.subscriber_builder().create().expect("");

    auto sample_uninit = sut_publisher.loan_slice_uninit(SLICE_MAX_LENGTH).expect("");
    auto send_sample = sample_uninit.write_from_fn(
        [](auto index) { return DummyData { DummyData::DEFAULT_VALUE_A + index, index % 2 == 0 }; });
    send(std::move(send_sample)).expect("");

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

    auto sut_publisher = service.publisher_builder().initial_max_slice_len(SLICE_MAX_LENGTH).create().expect("");
    auto sut_subscriber = service.subscriber_builder().create().expect("");

    iox::UninitializedArray<DummyData, SLICE_MAX_LENGTH, iox::ZeroedBuffer> elements;
    for (auto& item : elements) {
        new (&item) DummyData {};
    }
    auto payload = iox::ImmutableSlice<DummyData>(elements.begin(), SLICE_MAX_LENGTH);
    auto sample_uninit = sut_publisher.loan_slice_uninit(SLICE_MAX_LENGTH).expect("");
    auto send_sample = sample_uninit.write_from_slice(payload);
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
// NOLINTEND(readability-function-cognitive-complexity)

TYPED_TEST(ServicePublishSubscribeTest, update_connections_delivers_history) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto service =
        node.service_builder(service_name).template publish_subscribe<uint64_t>().history_size(1).create().expect("");

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
    ASSERT_THAT(static_config.message_type_details().payload().type_name(), StrEq("u64"));

    auto subscriber = service.subscriber_builder().create().expect("");
    ASSERT_THAT(subscriber.buffer_size(), Eq(SUBSCRIBER_MAX_BUFFER_SIZE));

    auto subscriber_2 = service.subscriber_builder().buffer_size(1).create().expect("");
    ASSERT_THAT(subscriber_2.buffer_size(), Eq(1));
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

    auto sut = service.publisher_builder().initial_max_slice_len(DESIRED_MAX_SLICE_LEN).create().expect("");

    ASSERT_THAT(sut.initial_max_slice_len(), Eq(DESIRED_MAX_SLICE_LEN));
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
        ASSERT_THAT(recv_sample->user_header().value.at(idx), Eq((4 * idx) + 3));
    }
}

TYPED_TEST(ServicePublishSubscribeTest, loan_has_default_constructed_user_header) {
    constexpr uint64_t RAND_A = 123;
    constexpr uint32_t RAND_B = 456;
    using UserHeader = CustomTestHeader<RAND_A, RAND_B>;
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto service = node.service_builder(service_name)
                       .template publish_subscribe<uint64_t>()
                       .template user_header<UserHeader>()
                       .create()
                       .expect("");

    auto publisher = service.publisher_builder().create().expect("");
    auto sample = publisher.loan().expect("");
    ASSERT_THAT(sample.user_header(), Eq(UserHeader()));
}

TYPED_TEST(ServicePublishSubscribeTest, loan_uninit_has_default_constructed_user_header) {
    constexpr uint64_t RAND_A = 4123;
    constexpr uint32_t RAND_B = 4456;
    using UserHeader = CustomTestHeader<RAND_A, RAND_B>;
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto service = node.service_builder(service_name)
                       .template publish_subscribe<uint64_t>()
                       .template user_header<UserHeader>()
                       .create()
                       .expect("");

    auto publisher = service.publisher_builder().create().expect("");
    auto sample = publisher.loan_uninit().expect("");
    ASSERT_THAT(sample.user_header(), Eq(UserHeader()));
}

TYPED_TEST(ServicePublishSubscribeTest, loan_slice_has_default_constructed_user_header) {
    constexpr uint64_t RAND_A = 41231;
    constexpr uint32_t RAND_B = 44561;
    using UserHeader = CustomTestHeader<RAND_A, RAND_B>;
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto service = node.service_builder(service_name)
                       .template publish_subscribe<iox::Slice<uint64_t>>()
                       .template user_header<UserHeader>()
                       .create()
                       .expect("");

    auto publisher = service.publisher_builder().create().expect("");
    auto sample = publisher.loan_slice(1).expect("");
    ASSERT_THAT(sample.user_header(), Eq(UserHeader()));
}

TYPED_TEST(ServicePublishSubscribeTest, loan_slice_uninit_has_default_constructed_user_header) {
    constexpr uint64_t RAND_A = 641231;
    constexpr uint32_t RAND_B = 644561;
    using UserHeader = CustomTestHeader<RAND_A, RAND_B>;
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto service = node.service_builder(service_name)
                       .template publish_subscribe<iox::Slice<uint64_t>>()
                       .template user_header<UserHeader>()
                       .create()
                       .expect("");

    auto publisher = service.publisher_builder().create().expect("");
    auto sample = publisher.loan_slice_uninit(1).expect("");
    ASSERT_THAT(sample.user_header(), Eq(UserHeader()));
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
    {
        auto sample = subscriber->receive().expect("");
        ASSERT_THAT(sample->payload(), Eq(payload));
    }

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
    {
        auto sample = subscriber->receive().expect("");
        ASSERT_THAT(sample->payload(), Eq(payload));
    }

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

TYPED_TEST(ServicePublishSubscribeTest, publisher_reallocates_memory_when_allocation_strategy_is_set) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    using ValueType = uint8_t;
    constexpr uint64_t INITIAL_SIZE = 128;

    const auto service_name = iox2_testing::generate_service_name();
    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto service =
        node.service_builder(service_name).template publish_subscribe<iox::Slice<ValueType>>().create().expect("");

    auto publisher = service.publisher_builder()
                         .initial_max_slice_len(INITIAL_SIZE)
                         .allocation_strategy(AllocationStrategy::BestFit)
                         .create()
                         .expect("");

    {
        auto sample = publisher.loan_slice(INITIAL_SIZE);
        ASSERT_THAT(sample.has_value(), Eq(true));
    }

    {
        auto sample = publisher.loan_slice(INITIAL_SIZE * INITIAL_SIZE);
        ASSERT_THAT(sample.has_value(), Eq(true));
    }

    {
        auto sample = publisher.loan_slice(INITIAL_SIZE * INITIAL_SIZE * INITIAL_SIZE);
        ASSERT_THAT(sample.has_value(), Eq(true));
    }
}

TYPED_TEST(ServicePublishSubscribeTest, publisher_does_not_reallocate_when_allocation_strategy_is_static) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    using ValueType = uint8_t;
    constexpr uint64_t INITIAL_SIZE = 128;

    const auto service_name = iox2_testing::generate_service_name();
    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto service =
        node.service_builder(service_name).template publish_subscribe<iox::Slice<ValueType>>().create().expect("");

    auto publisher = service.publisher_builder()
                         .initial_max_slice_len(INITIAL_SIZE)
                         .allocation_strategy(AllocationStrategy::Static)
                         .create()
                         .expect("");

    auto sample_1 = publisher.loan_slice(INITIAL_SIZE);
    ASSERT_THAT(sample_1.has_value(), Eq(true));

    auto sample_2 = publisher.loan_slice(INITIAL_SIZE * INITIAL_SIZE);
    ASSERT_THAT(sample_2.has_value(), Eq(false));
    ASSERT_THAT(sample_2.error(), Eq(LoanError::ExceedsMaxLoanSize));

    auto sample_3 = publisher.loan_slice(INITIAL_SIZE * INITIAL_SIZE * INITIAL_SIZE);
    ASSERT_THAT(sample_3.has_value(), Eq(false));
    ASSERT_THAT(sample_3.error(), Eq(LoanError::ExceedsMaxLoanSize));
}

TYPED_TEST(ServicePublishSubscribeTest, create_with_attributes_sets_attributes) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    auto key = Attribute::Key("want to make your machine run faster:");
    auto value = Attribute::Value("sudo rm -rf /");
    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto service_create = node.service_builder(service_name)
                              .template publish_subscribe<uint64_t>()
                              .create_with_attributes(AttributeSpecifier().define(key, value))
                              .expect("");

    auto service_open = node.service_builder(service_name).template publish_subscribe<uint64_t>().open().expect("");


    auto attributes_create = service_create.attributes();
    auto attributes_open = service_open.attributes();

    ASSERT_THAT(attributes_create.number_of_attributes(), Eq(1));
    ASSERT_THAT(attributes_create[0].key(), Eq(key));
    ASSERT_THAT(attributes_create[0].value(), Eq(value));

    ASSERT_THAT(attributes_open.number_of_attributes(), Eq(1));
    ASSERT_THAT(attributes_open[0].key(), Eq(key));
    ASSERT_THAT(attributes_open[0].value(), Eq(value));
}

TYPED_TEST(ServicePublishSubscribeTest, open_fails_when_attributes_are_incompatible) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    auto key = Attribute::Key("whats hypnotoad doing these days?");
    auto value = Attribute::Value("eating hypnoflies?");
    auto missing_key = Attribute::Key("no he is singing a song!");
    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto service_create = node.service_builder(service_name)
                              .template publish_subscribe<uint64_t>()
                              .open_or_create_with_attributes(AttributeVerifier().require(key, value))
                              .expect("");

    auto service_open_or_create =
        node.service_builder(service_name)
            .template publish_subscribe<uint64_t>()
            .open_or_create_with_attributes(AttributeVerifier().require(key, value).require_key(missing_key));

    ASSERT_THAT(service_open_or_create.has_error(), Eq(true));
    ASSERT_THAT(service_open_or_create.error(), Eq(PublishSubscribeOpenOrCreateError::OpenIncompatibleAttributes));

    auto service_open = node.service_builder(service_name)
                            .template publish_subscribe<uint64_t>()
                            .open_with_attributes(AttributeVerifier().require(key, value).require_key(missing_key));

    ASSERT_THAT(service_open.has_error(), Eq(true));
    ASSERT_THAT(service_open.error(), Eq(PublishSubscribeOpenError::IncompatibleAttributes));
}

// BEGIN tests for customizable payload and user header type name
constexpr uint8_t CAPACITY = 100;
constexpr uint8_t ALIGNMENT = 16;
struct Payload {
    static constexpr const char* IOX2_TYPE_NAME = "Payload";
    int32_t x;
    double y;
};

struct DifferentPayloadWithSameTypeName {
    static constexpr const char* IOX2_TYPE_NAME = "Payload";
    int32_t x;
    double y;
};

struct PayloadWithSameTypeNameButDifferentSize {
    static constexpr const char* IOX2_TYPE_NAME = "Payload";
    int32_t x;
    double y;
    std::array<int32_t, CAPACITY> z;
};

struct alignas(ALIGNMENT) PayloadWithSameTypeNameButDifferentAlignment {
    static constexpr const char* IOX2_TYPE_NAME = "Payload";
    int32_t x;
    double y;
};

struct CustomHeader {
    static constexpr const char* IOX2_TYPE_NAME = "CustomHeader";
    uint64_t a;
    uint8_t b;
};

struct DifferentCustomHeaderWithSameTypeName {
    static constexpr const char* IOX2_TYPE_NAME = "CustomHeader";
    uint64_t a;
    uint8_t b;
};

struct CustomHeaderWithSameTypeNameButDifferentSize {
    static constexpr const char* IOX2_TYPE_NAME = "CustomHeader";
    uint64_t a;
    uint8_t b;
    std::array<uint8_t, CAPACITY> c;
};

struct alignas(ALIGNMENT) CustomHeaderWithSameTypeNameButDifferentAlignment {
    static constexpr const char* IOX2_TYPE_NAME = "CustomHeader";
    uint64_t a;
    uint8_t b;
};

namespace other {
struct Payload {
    static constexpr const char* IOX2_TYPE_NAME = "DifferentPayload";
    int32_t x;
    double y;
};

struct CustomHeader {
    static constexpr const char* IOX2_TYPE_NAME = "DifferentCustomHeader";
    uint64_t a;
    uint8_t b;
};
} // namespace other

TYPED_TEST(ServicePublishSubscribeTest, opening_existing_service_with_set_payload_type_name_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut_create = node.service_builder(service_name).template publish_subscribe<Payload>().create().expect("");
    auto sut_open = node.service_builder(service_name).template publish_subscribe<Payload>().open();
    ASSERT_FALSE(sut_open.has_error());
}

TYPED_TEST(ServicePublishSubscribeTest,
           opening_existing_service_with_different_payload_but_same_set_payload_type_name_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut_create = node.service_builder(service_name).template publish_subscribe<Payload>().create().expect("");
    auto sut_open =
        node.service_builder(service_name).template publish_subscribe<DifferentPayloadWithSameTypeName>().open();
    ASSERT_FALSE(sut_open.has_error());
}

TYPED_TEST(ServicePublishSubscribeTest, opening_existing_service_without_payload_type_name_fails) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut_create = node.service_builder(service_name).template publish_subscribe<Payload>().create().expect("");

    struct Payload {
        int32_t x;
        double y;
    };
    auto sut_open = node.service_builder(service_name).template publish_subscribe<Payload>().open();
    ASSERT_TRUE(sut_open.has_error());
    EXPECT_EQ(sut_open.error(), PublishSubscribeOpenError::IncompatibleTypes);
}

TYPED_TEST(ServicePublishSubscribeTest,
           opening_existing_service_with_same_payload_but_different_payload_type_name_fails) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut_create = node.service_builder(service_name).template publish_subscribe<Payload>().create().expect("");

    auto sut_open = node.service_builder(service_name).template publish_subscribe<other::Payload>().open();
    ASSERT_TRUE(sut_open.has_error());
    EXPECT_EQ(sut_open.error(), PublishSubscribeOpenError::IncompatibleTypes);
}

TYPED_TEST(ServicePublishSubscribeTest, opening_existing_service_with_same_payload_type_name_but_different_size_fails) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut_create = node.service_builder(service_name).template publish_subscribe<Payload>().create().expect("");

    auto sut_open =
        node.service_builder(service_name).template publish_subscribe<PayloadWithSameTypeNameButDifferentSize>().open();
    ASSERT_TRUE(sut_open.has_error());
    EXPECT_EQ(sut_open.error(), PublishSubscribeOpenError::IncompatibleTypes);
}

TYPED_TEST(ServicePublishSubscribeTest,
           opening_existing_service_with_same_payload_type_name_but_different_alignment_fails) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut_create = node.service_builder(service_name).template publish_subscribe<Payload>().create().expect("");

    auto sut_open = node.service_builder(service_name)
                        .template publish_subscribe<PayloadWithSameTypeNameButDifferentAlignment>()
                        .open();
    ASSERT_TRUE(sut_open.has_error());
    EXPECT_EQ(sut_open.error(), PublishSubscribeOpenError::IncompatibleTypes);
}

TYPED_TEST(ServicePublishSubscribeTest, opening_existing_service_with_set_user_header_type_name_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut_create = node.service_builder(service_name)
                          .template publish_subscribe<uint8_t>()
                          .template user_header<CustomHeader>()
                          .create()
                          .expect("");
    auto sut_open = node.service_builder(service_name)
                        .template publish_subscribe<uint8_t>()
                        .template user_header<CustomHeader>()
                        .open();
    ASSERT_FALSE(sut_open.has_error());
}

TYPED_TEST(ServicePublishSubscribeTest,
           opening_existing_service_with_different_header_but_same_set_user_header_type_name_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut_create = node.service_builder(service_name)
                          .template publish_subscribe<uint8_t>()
                          .template user_header<CustomHeader>()
                          .create()
                          .expect("");
    auto sut_open = node.service_builder(service_name)
                        .template publish_subscribe<uint8_t>()
                        .template user_header<DifferentCustomHeaderWithSameTypeName>()
                        .open();
    ASSERT_FALSE(sut_open.has_error());
}

TYPED_TEST(ServicePublishSubscribeTest, opening_existing_service_without_user_header_type_name_fails) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut_create = node.service_builder(service_name)
                          .template publish_subscribe<uint8_t>()
                          .template user_header<CustomHeader>()
                          .create()
                          .expect("");

    struct CustomHeader {
        uint64_t a;
        uint8_t b;
    };
    auto sut_open = node.service_builder(service_name)
                        .template publish_subscribe<uint8_t>()
                        .template user_header<CustomHeader>()
                        .open();
    ASSERT_TRUE(sut_open.has_error());
    EXPECT_EQ(sut_open.error(), PublishSubscribeOpenError::IncompatibleTypes);
}

TYPED_TEST(ServicePublishSubscribeTest,
           opening_existing_service_with_same_header_but_different_user_header_type_name_fails) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut_create = node.service_builder(service_name)
                          .template publish_subscribe<uint8_t>()
                          .template user_header<CustomHeader>()
                          .create()
                          .expect("");

    auto sut_open = node.service_builder(service_name)
                        .template publish_subscribe<uint8_t>()
                        .template user_header<other::CustomHeader>()
                        .open();
    ASSERT_TRUE(sut_open.has_error());
    EXPECT_EQ(sut_open.error(), PublishSubscribeOpenError::IncompatibleTypes);
}

TYPED_TEST(ServicePublishSubscribeTest, opening_existing_service_with_same_header_type_but_different_size_fails) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut_create = node.service_builder(service_name)
                          .template publish_subscribe<uint8_t>()
                          .template user_header<CustomHeader>()
                          .create()
                          .expect("");

    auto sut_open = node.service_builder(service_name)
                        .template publish_subscribe<uint8_t>()
                        .template user_header<CustomHeaderWithSameTypeNameButDifferentSize>()
                        .open();
    ASSERT_TRUE(sut_open.has_error());
    EXPECT_EQ(sut_open.error(), PublishSubscribeOpenError::IncompatibleTypes);
}

TYPED_TEST(ServicePublishSubscribeTest, opening_existing_service_with_same_header_type_but_different_alignment_fails) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut_create = node.service_builder(service_name)
                          .template publish_subscribe<uint8_t>()
                          .template user_header<CustomHeader>()
                          .create()
                          .expect("");

    auto sut_open = node.service_builder(service_name)
                        .template publish_subscribe<uint8_t>()
                        .template user_header<CustomHeaderWithSameTypeNameButDifferentAlignment>()
                        .open();
    ASSERT_TRUE(sut_open.has_error());
    EXPECT_EQ(sut_open.error(), PublishSubscribeOpenError::IncompatibleTypes);
}

TYPED_TEST(ServicePublishSubscribeTest, PayloadTypeNameIsSetToRustPendantForFixedSizeIntegersAndBoolAndSlicesOfThem) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    const auto service_name = iox2_testing::generate_service_name();
    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    {
        auto service = node.service_builder(service_name).template publish_subscribe<uint8_t>().create().expect("");
        auto static_config = service.static_config();
        ASSERT_THAT(static_config.message_type_details().payload().type_name(), StrEq("u8"));
    }
    {
        auto service = node.service_builder(service_name).template publish_subscribe<uint16_t>().create().expect("");
        auto static_config = service.static_config();
        ASSERT_THAT(static_config.message_type_details().payload().type_name(), StrEq("u16"));
    }
    {
        auto service = node.service_builder(service_name).template publish_subscribe<uint32_t>().create().expect("");
        auto static_config = service.static_config();
        ASSERT_THAT(static_config.message_type_details().payload().type_name(), StrEq("u32"));
    }
    {
        auto service = node.service_builder(service_name).template publish_subscribe<uint64_t>().create().expect("");
        auto static_config = service.static_config();
        ASSERT_THAT(static_config.message_type_details().payload().type_name(), StrEq("u64"));
    }
    {
        auto service = node.service_builder(service_name).template publish_subscribe<int8_t>().create().expect("");
        auto static_config = service.static_config();
        ASSERT_THAT(static_config.message_type_details().payload().type_name(), StrEq("i8"));
    }
    {
        auto service = node.service_builder(service_name).template publish_subscribe<int16_t>().create().expect("");
        auto static_config = service.static_config();
        ASSERT_THAT(static_config.message_type_details().payload().type_name(), StrEq("i16"));
    }
    {
        auto service = node.service_builder(service_name).template publish_subscribe<int32_t>().create().expect("");
        auto static_config = service.static_config();
        ASSERT_THAT(static_config.message_type_details().payload().type_name(), StrEq("i32"));
    }
    {
        auto service = node.service_builder(service_name).template publish_subscribe<int64_t>().create().expect("");
        auto static_config = service.static_config();
        ASSERT_THAT(static_config.message_type_details().payload().type_name(), StrEq("i64"));
    }
    {
        auto service = node.service_builder(service_name).template publish_subscribe<float>().create().expect("");
        auto static_config = service.static_config();
        ASSERT_THAT(static_config.message_type_details().payload().type_name(), StrEq("f32"));
    }
    {
        auto service = node.service_builder(service_name).template publish_subscribe<double>().create().expect("");
        auto static_config = service.static_config();
        ASSERT_THAT(static_config.message_type_details().payload().type_name(), StrEq("f64"));
    }
    {
        auto service = node.service_builder(service_name).template publish_subscribe<bool>().create().expect("");
        auto static_config = service.static_config();
        ASSERT_THAT(static_config.message_type_details().payload().type_name(), StrEq("bool"));
    }
    {
        auto service =
            node.service_builder(service_name).template publish_subscribe<iox::Slice<uint8_t>>().create().expect("");
        auto static_config = service.static_config();
        ASSERT_THAT(static_config.message_type_details().payload().type_name(), StrEq("u8"));
    }
    {
        auto service =
            node.service_builder(service_name).template publish_subscribe<iox::Slice<uint16_t>>().create().expect("");
        auto static_config = service.static_config();
        ASSERT_THAT(static_config.message_type_details().payload().type_name(), StrEq("u16"));
    }
    {
        auto service =
            node.service_builder(service_name).template publish_subscribe<iox::Slice<uint32_t>>().create().expect("");
        auto static_config = service.static_config();
        ASSERT_THAT(static_config.message_type_details().payload().type_name(), StrEq("u32"));
    }
    {
        auto service =
            node.service_builder(service_name).template publish_subscribe<iox::Slice<uint64_t>>().create().expect("");
        auto static_config = service.static_config();
        ASSERT_THAT(static_config.message_type_details().payload().type_name(), StrEq("u64"));
    }
    {
        auto service =
            node.service_builder(service_name).template publish_subscribe<iox::Slice<int8_t>>().create().expect("");
        auto static_config = service.static_config();
        ASSERT_THAT(static_config.message_type_details().payload().type_name(), StrEq("i8"));
    }
    {
        auto service =
            node.service_builder(service_name).template publish_subscribe<iox::Slice<int16_t>>().create().expect("");
        auto static_config = service.static_config();
        ASSERT_THAT(static_config.message_type_details().payload().type_name(), StrEq("i16"));
    }
    {
        auto service =
            node.service_builder(service_name).template publish_subscribe<iox::Slice<int32_t>>().create().expect("");
        auto static_config = service.static_config();
        ASSERT_THAT(static_config.message_type_details().payload().type_name(), StrEq("i32"));
    }
    {
        auto service =
            node.service_builder(service_name).template publish_subscribe<iox::Slice<int64_t>>().create().expect("");
        auto static_config = service.static_config();
        ASSERT_THAT(static_config.message_type_details().payload().type_name(), StrEq("i64"));
    }
    {
        auto service =
            node.service_builder(service_name).template publish_subscribe<iox::Slice<float>>().create().expect("");
        auto static_config = service.static_config();
        ASSERT_THAT(static_config.message_type_details().payload().type_name(), StrEq("f32"));
    }
    {
        auto service =
            node.service_builder(service_name).template publish_subscribe<iox::Slice<double>>().create().expect("");
        auto static_config = service.static_config();
        ASSERT_THAT(static_config.message_type_details().payload().type_name(), StrEq("f64"));
    }
    {
        auto service =
            node.service_builder(service_name).template publish_subscribe<iox::Slice<bool>>().create().expect("");
        auto static_config = service.static_config();
        ASSERT_THAT(static_config.message_type_details().payload().type_name(), StrEq("bool"));
    }
}

TYPED_TEST(ServicePublishSubscribeTest, PayloadTypeNameIsSetToInnerTypeNameIfProvided) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    const auto service_name = iox2_testing::generate_service_name();
    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto service =
        node.service_builder(service_name).template publish_subscribe<iox::Slice<Payload>>().create().expect("");

    auto static_config = service.static_config();
    ASSERT_THAT(static_config.message_type_details().payload().type_name(), StrEq("Payload"));
}
// END tests for customizable payload and user header type name

TYPED_TEST(ServicePublishSubscribeTest, service_id_is_unique_per_service) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    const auto service_name_1 = iox2_testing::generate_service_name();
    const auto service_name_2 = iox2_testing::generate_service_name();
    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");

    auto service_1_create =
        node.service_builder(service_name_1).template publish_subscribe<uint64_t>().create().expect("");
    auto service_1_open = node.service_builder(service_name_1).template publish_subscribe<uint64_t>().open().expect("");
    auto service_2 = node.service_builder(service_name_2).template publish_subscribe<uint64_t>().create().expect("");

    ASSERT_THAT(service_1_create.service_id().c_str(), StrEq(service_1_open.service_id().c_str()));
    ASSERT_THAT(service_1_create.service_id().c_str(), Not(StrEq(service_2.service_id().c_str())));
}

TYPED_TEST(ServicePublishSubscribeTest, listing_all_subscribers_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    constexpr uint64_t NUMBER_OF_SUBSCRIBERS = 16;

    const auto service_name = iox2_testing::generate_service_name();
    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut = node.service_builder(service_name)
                   .template publish_subscribe<uint64_t>()
                   .max_subscribers(NUMBER_OF_SUBSCRIBERS)
                   .create()
                   .expect("");

    std::vector<iox2::Subscriber<SERVICE_TYPE, uint64_t, void>> subscribers;
    subscribers.reserve(NUMBER_OF_SUBSCRIBERS);
    for (uint64_t idx = 0; idx < NUMBER_OF_SUBSCRIBERS; ++idx) {
        subscribers.push_back(sut.subscriber_builder().create().expect(""));
    }

    std::vector<UniqueSubscriberId> subscriber_ids;
    subscriber_ids.reserve(NUMBER_OF_SUBSCRIBERS);
    sut.dynamic_config().list_subscribers([&](auto subscriber_details_view) {
        subscriber_ids.push_back(subscriber_details_view.subscriber_id());
        return CallbackProgression::Continue;
    });

    ASSERT_THAT(subscriber_ids.size(), Eq(NUMBER_OF_SUBSCRIBERS));
    for (auto& subscriber : subscribers) {
        auto iter = std::find(subscriber_ids.begin(), subscriber_ids.end(), subscriber.id());
        ASSERT_THAT(iter, Ne(subscriber_ids.end()));
    }
}

TYPED_TEST(ServicePublishSubscribeTest, listing_all_subscribers_stops_on_request) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    constexpr uint64_t NUMBER_OF_SUBSCRIBERS = 13;

    const auto service_name = iox2_testing::generate_service_name();
    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut = node.service_builder(service_name)
                   .template publish_subscribe<uint64_t>()
                   .max_subscribers(NUMBER_OF_SUBSCRIBERS)
                   .create()
                   .expect("");

    std::vector<iox2::Subscriber<SERVICE_TYPE, uint64_t, void>> subscribers;
    subscribers.reserve(NUMBER_OF_SUBSCRIBERS);
    for (uint64_t idx = 0; idx < NUMBER_OF_SUBSCRIBERS; ++idx) {
        subscribers.push_back(sut.subscriber_builder().create().expect(""));
    }

    auto counter = 0;
    sut.dynamic_config().list_subscribers([&](auto) {
        counter++;
        return CallbackProgression::Stop;
    });

    ASSERT_THAT(counter, Eq(1));
}

TYPED_TEST(ServicePublishSubscribeTest, subscriber_details_are_correct) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();
    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut = node.service_builder(service_name).template publish_subscribe<uint64_t>().create().expect("");

    iox2::Subscriber<SERVICE_TYPE, uint64_t, void> subscriber = sut.subscriber_builder().create().expect("");

    auto counter = 0;
    sut.dynamic_config().list_subscribers([&](auto subscriber_details_view) {
        counter++;
        EXPECT_TRUE(subscriber_details_view.subscriber_id() == subscriber.id());
        EXPECT_TRUE(subscriber_details_view.node_id() == node.id());
        EXPECT_TRUE(subscriber_details_view.buffer_size() == subscriber.buffer_size());
        return CallbackProgression::Stop;
    });

    ASSERT_THAT(counter, Eq(1));
}

TYPED_TEST(ServicePublishSubscribeTest, listing_all_publishers_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    constexpr uint64_t NUMBER_OF_PUBLISHERS = 16;

    const auto service_name = iox2_testing::generate_service_name();
    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut = node.service_builder(service_name)
                   .template publish_subscribe<uint64_t>()
                   .max_publishers(NUMBER_OF_PUBLISHERS)
                   .create()
                   .expect("");

    std::vector<iox2::Publisher<SERVICE_TYPE, uint64_t, void>> publishers;
    publishers.reserve(NUMBER_OF_PUBLISHERS);
    for (uint64_t idx = 0; idx < NUMBER_OF_PUBLISHERS; ++idx) {
        publishers.push_back(sut.publisher_builder().create().expect(""));
    }

    std::vector<UniquePublisherId> publisher_ids;
    publisher_ids.reserve(NUMBER_OF_PUBLISHERS);
    sut.dynamic_config().list_publishers([&](auto publisher_details_view) {
        publisher_ids.push_back(publisher_details_view.publisher_id());
        return CallbackProgression::Continue;
    });

    ASSERT_THAT(publisher_ids.size(), Eq(NUMBER_OF_PUBLISHERS));
    for (auto& publisher : publishers) {
        auto iter = std::find(publisher_ids.begin(), publisher_ids.end(), publisher.id());
        ASSERT_THAT(iter, Ne(publisher_ids.end()));
    }
}

TYPED_TEST(ServicePublishSubscribeTest, listing_all_publishers_stops_on_request) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    constexpr uint64_t NUMBER_OF_PUBLISHERS = 13;

    const auto service_name = iox2_testing::generate_service_name();
    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut = node.service_builder(service_name)
                   .template publish_subscribe<uint64_t>()
                   .max_publishers(NUMBER_OF_PUBLISHERS)
                   .create()
                   .expect("");

    std::vector<iox2::Publisher<SERVICE_TYPE, uint64_t, void>> publishers;
    publishers.reserve(NUMBER_OF_PUBLISHERS);
    for (uint64_t idx = 0; idx < NUMBER_OF_PUBLISHERS; ++idx) {
        publishers.push_back(sut.publisher_builder().create().expect(""));
    }

    auto counter = 0;
    sut.dynamic_config().list_publishers([&](auto) {
        counter++;
        return CallbackProgression::Stop;
    });

    ASSERT_THAT(counter, Eq(1));
}

TYPED_TEST(ServicePublishSubscribeTest, publisher_details_are_correct) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    constexpr uint64_t INITIAL_MAX_SLICE_LEN = 5;

    const auto service_name = iox2_testing::generate_service_name();
    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut =
        node.service_builder(service_name).template publish_subscribe<iox::Slice<uint64_t>>().create().expect("");

    iox2::Publisher<SERVICE_TYPE, iox::Slice<uint64_t>, void> publisher =
        sut.publisher_builder().initial_max_slice_len(INITIAL_MAX_SLICE_LEN).create().expect("");

    auto counter = 0;
    sut.dynamic_config().list_publishers([&](auto publisher_details_view) {
        counter++;
        EXPECT_TRUE(publisher_details_view.publisher_id() == publisher.id());
        EXPECT_TRUE(publisher_details_view.node_id() == node.id());
        EXPECT_TRUE(publisher_details_view.max_slice_len() == INITIAL_MAX_SLICE_LEN);
        return CallbackProgression::Stop;
    });

    ASSERT_THAT(counter, Eq(1));
}

TYPED_TEST(ServicePublishSubscribeTest, only_max_publishers_can_be_created) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto service =
        node.service_builder(service_name).template publish_subscribe<uint64_t>().max_publishers(1).create().expect("");
    auto publisher =
        iox::optional<Publisher<SERVICE_TYPE, uint64_t, void>>(service.publisher_builder().create().expect(""));

    auto failing_sut = service.publisher_builder().create();
    ASSERT_TRUE(failing_sut.has_error());

    publisher.reset();

    auto sut = service.publisher_builder().create();
    ASSERT_FALSE(sut.has_error());
}

TYPED_TEST(ServicePublishSubscribeTest, only_max_subscribers_can_be_created) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto service = node.service_builder(service_name)
                       .template publish_subscribe<uint64_t>()
                       .max_subscribers(1)
                       .create()
                       .expect("");
    auto subscriber =
        iox::optional<Subscriber<SERVICE_TYPE, uint64_t, void>>(service.subscriber_builder().create().expect(""));

    auto failing_sut = service.subscriber_builder().create();
    ASSERT_TRUE(failing_sut.has_error());

    subscriber.reset();

    auto sut = service.subscriber_builder().create();
    ASSERT_FALSE(sut.has_error());
}
} // namespace
