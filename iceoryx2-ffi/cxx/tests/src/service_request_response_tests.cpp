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

#include "iox2/node.hpp"
#include "iox2/service.hpp"

#include "test.hpp"

namespace {
using namespace iox2;

template <typename T>
class ServiceRequestResponseTest : public ::testing::Test {
  public:
    static constexpr ServiceType TYPE = T::TYPE;
};

TYPED_TEST_SUITE(ServiceRequestResponseTest, iox2_testing::ServiceTypes, );

TYPED_TEST(ServiceRequestResponseTest, created_service_does_exist) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    ASSERT_FALSE(
        Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::RequestResponse)
            .expect(""));

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");

    {
        auto sut =
            node.service_builder(service_name).template request_response<uint64_t, uint64_t>().create().expect("");

        ASSERT_TRUE(
            Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::RequestResponse)
                .expect(""));
    }

    ASSERT_FALSE(
        Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::Event).expect(""));
    ASSERT_FALSE(
        Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::PublishSubscribe)
            .expect(""));
}

TYPED_TEST(ServiceRequestResponseTest, service_name_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut = node.service_builder(service_name).template request_response<uint64_t, uint64_t>().create().expect("");

    ASSERT_THAT(sut.name().to_string().c_str(), StrEq(service_name.to_string().c_str()));
}

TYPED_TEST(ServiceRequestResponseTest, list_service_nodes_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto node_name_1 = NodeName::create("is there any of the herring left?").expect("");
    const auto node_name_2 = NodeName::create("nala and octo-wolf asked in unison").expect("");
    const auto service_name = iox2_testing::generate_service_name();

    auto node_1 = NodeBuilder().name(node_name_1).create<SERVICE_TYPE>().expect("");
    auto node_2 = NodeBuilder().name(node_name_2).create<SERVICE_TYPE>().expect("");

    auto sut_1 =
        node_1.service_builder(service_name).template request_response<uint64_t, uint64_t>().create().expect("");
    auto sut_2 = node_2.service_builder(service_name).template request_response<uint64_t, uint64_t>().open().expect("");

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

TYPED_TEST(ServiceRequestResponseTest, creating_existing_service_fails) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    ASSERT_FALSE(
        Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::RequestResponse)
            .expect(""));

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut = node.service_builder(service_name).template request_response<uint64_t, uint64_t>().create().expect("");
    auto sut_2 = node.service_builder(service_name).template request_response<uint64_t, uint64_t>().create();

    ASSERT_TRUE(sut_2.has_error());
    ASSERT_THAT(sut_2.error(), Eq(RequestResponseCreateError::AlreadyExists));
}

TYPED_TEST(ServiceRequestResponseTest, open_or_create_service_does_exist) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    ASSERT_FALSE(
        Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::RequestResponse)
            .expect(""));

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");

    {
        auto sut = iox::optional<PortFactoryRequestResponse<SERVICE_TYPE, uint64_t, void, uint64_t, void>>(
            node.service_builder(service_name)
                .template request_response<uint64_t, uint64_t>()
                .open_or_create()
                .expect(""));

        ASSERT_TRUE(
            Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::RequestResponse)
                .expect(""));

        auto sut_2 = iox::optional<PortFactoryRequestResponse<SERVICE_TYPE, uint64_t, void, uint64_t, void>>(
            node.service_builder(service_name)
                .template request_response<uint64_t, uint64_t>()
                .open_or_create()
                .expect(""));

        ASSERT_TRUE(
            Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::RequestResponse)
                .expect(""));

        sut.reset();

        ASSERT_TRUE(
            Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::RequestResponse)
                .expect(""));

        sut_2.reset();
    }

    ASSERT_FALSE(
        Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::RequestResponse)
            .expect(""));
}

TYPED_TEST(ServiceRequestResponseTest, opening_non_existing_service_fails) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut = node.service_builder(service_name).template request_response<uint64_t, uint64_t>().open();
    ASSERT_TRUE(sut.has_error());
    ASSERT_THAT(sut.error(), Eq(RequestResponseOpenError::DoesNotExist));
}

TYPED_TEST(ServiceRequestResponseTest, opening_existing_service_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut_create =
        node.service_builder(service_name).template request_response<uint64_t, uint64_t>().create().expect("");
    auto sut = node.service_builder(service_name).template request_response<uint64_t, uint64_t>().open();
    ASSERT_TRUE(sut.has_value());
}

TYPED_TEST(ServiceRequestResponseTest, opening_existing_service_with_wrong_payload_type_fails) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut_create =
        node.service_builder(service_name).template request_response<uint64_t, uint64_t>().create().expect("");

    auto sut1 = node.service_builder(service_name).template request_response<double, uint64_t>().open();
    ASSERT_TRUE(sut1.has_error());
    ASSERT_THAT(sut1.error(), Eq(RequestResponseOpenError::IncompatibleRequestType));

    auto sut2 = node.service_builder(service_name).template request_response<uint64_t, double>().open();
    ASSERT_TRUE(sut2.has_error());
    ASSERT_THAT(sut2.error(), Eq(RequestResponseOpenError::IncompatibleResponseType));
}

TYPED_TEST(ServiceRequestResponseTest, open_or_create_existing_service_with_wrong_payload_type_fails) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut_create =
        node.service_builder(service_name).template request_response<uint64_t, uint64_t>().create().expect("");

    auto sut1 = node.service_builder(service_name).template request_response<double, uint64_t>().open_or_create();
    ASSERT_TRUE(sut1.has_error());
    ASSERT_THAT(sut1.error(), Eq(RequestResponseOpenOrCreateError::OpenIncompatibleRequestType));

    auto sut2 = node.service_builder(service_name).template request_response<uint64_t, double>().open_or_create();
    ASSERT_TRUE(sut2.has_error());
    ASSERT_THAT(sut2.error(), Eq(RequestResponseOpenOrCreateError::OpenIncompatibleResponseType));
}

TYPED_TEST(ServiceRequestResponseTest, send_copy_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto service =
        node.service_builder(service_name).template request_response<uint64_t, uint64_t>().create().expect("");

    auto sut_client = service.client_builder().create().expect("");

    const uint64_t payload = 123;
    auto pending_response = sut_client.send_copy(payload);
    ASSERT_FALSE(pending_response.has_error());
}

TYPED_TEST(ServiceRequestResponseTest, loan_uninit_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto service =
        node.service_builder(service_name).template request_response<uint64_t, uint64_t>().create().expect("");

    auto sut_client = service.client_builder().create().expect("");

    auto request = sut_client.loan_uninit();
    ASSERT_FALSE(request.has_error());
}

TYPED_TEST(ServiceRequestResponseTest, loan_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    struct Payload {
        uint64_t p { 3 };
    };

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto service =
        node.service_builder(service_name).template request_response<Payload, uint64_t>().create().expect("");

    auto sut_client = service.client_builder().create().expect("");

    auto request = sut_client.loan();
    ASSERT_FALSE(request.has_error());
    EXPECT_THAT(request.value().payload().p, Eq(3));
}

struct DummyData {
    static constexpr uint64_t DEFAULT_VALUE_A = 42;
    static constexpr bool DEFAULT_VALUE_Z { false };
    uint64_t a { DEFAULT_VALUE_A };
    bool z { DEFAULT_VALUE_Z };
};

TYPED_TEST(ServiceRequestResponseTest, send_slice_copy_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    constexpr auto SLICE_MAX_LENGTH = 10;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto service = node.service_builder(service_name)
                       .template request_response<iox::Slice<DummyData>, uint64_t>()
                       .create()
                       .expect("");

    auto sut_client = service.client_builder().initial_max_slice_len(SLICE_MAX_LENGTH).create().expect("");

    iox::UninitializedArray<DummyData, SLICE_MAX_LENGTH, iox::ZeroedBuffer> elements;
    for (auto& item : elements) {
        new (&item) DummyData {};
    }
    auto payload = iox::ImmutableSlice<DummyData>(elements.begin(), SLICE_MAX_LENGTH);
    auto pending_response = sut_client.send_slice_copy(payload);
    ASSERT_FALSE(pending_response.has_error());
}

TYPED_TEST(ServiceRequestResponseTest, loan_slice_uninit_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    constexpr auto SLICE_MAX_LENGTH = 10;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto service = node.service_builder(service_name)
                       .template request_response<iox::Slice<DummyData>, uint64_t>()
                       .create()
                       .expect("");

    auto sut_client = service.client_builder().initial_max_slice_len(SLICE_MAX_LENGTH).create().expect("");

    auto request = sut_client.loan_slice_uninit(SLICE_MAX_LENGTH);
    ASSERT_FALSE(request.has_error());
}

TYPED_TEST(ServiceRequestResponseTest, loan_slice_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    constexpr auto SLICE_MAX_LENGTH = 10;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto service = node.service_builder(service_name)
                       .template request_response<iox::Slice<DummyData>, uint64_t>()
                       .create()
                       .expect("");

    auto sut_client = service.client_builder().initial_max_slice_len(SLICE_MAX_LENGTH).create().expect("");

    auto request = sut_client.loan_slice(SLICE_MAX_LENGTH);
    ASSERT_FALSE(request.has_error());
}

TYPED_TEST(ServiceRequestResponseTest, setting_service_properties_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    constexpr uint64_t NUMBER_OF_NODES = 10;
    constexpr uint64_t NUMBER_OF_CLIENTS = 11;
    constexpr uint64_t NUMBER_OF_SERVERS = 12;
    constexpr uint64_t ACTIVE_REQUESTS_PER_CLIENT = 3;
    constexpr uint64_t MAX_RESPONSE_BUFFER_SIZE = 4;
    constexpr uint64_t MAX_BORROWED_RESPONSES = 5;
    constexpr uint64_t MAX_LOANED_REQUESTS = 3;
    constexpr uint64_t REQUEST_PAYLOAD_ALIGNMENT = 4;
    constexpr uint64_t RESPONSE_PAYLOAD_ALIGNMENT = 8;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto service = node.service_builder(service_name)
                       .template request_response<uint64_t, uint64_t>()
                       .max_nodes(NUMBER_OF_NODES)
                       .max_clients(NUMBER_OF_CLIENTS)
                       .max_servers(NUMBER_OF_SERVERS)
                       .request_payload_alignment(REQUEST_PAYLOAD_ALIGNMENT)
                       .response_payload_alignment(RESPONSE_PAYLOAD_ALIGNMENT)
                       .enable_safe_overflow_for_requests(false)
                       .enable_safe_overflow_for_responses(false)
                       .max_active_requests_per_client(ACTIVE_REQUESTS_PER_CLIENT)
                       .max_response_buffer_size(MAX_RESPONSE_BUFFER_SIZE)
                       .max_borrowed_responses_per_pending_response(MAX_BORROWED_RESPONSES)
                       .max_loaned_requests(MAX_LOANED_REQUESTS)
                       .enable_fire_and_forget_requests(false)
                       .create()
                       .expect("");

    auto static_config = service.static_config();

    ASSERT_THAT(static_config.max_nodes(), Eq(NUMBER_OF_NODES));
    ASSERT_THAT(static_config.max_clients(), Eq(NUMBER_OF_CLIENTS));
    ASSERT_THAT(static_config.max_servers(), Eq(NUMBER_OF_SERVERS));
    ASSERT_THAT(static_config.request_message_type_details().payload().size(), Eq(sizeof(uint64_t)));
    ASSERT_THAT(static_config.request_message_type_details().payload().alignment(), Eq(alignof(uint64_t)));
    ASSERT_THAT(static_config.request_message_type_details().payload().type_name(), StrEq("u64"));
    ASSERT_THAT(static_config.response_message_type_details().payload().size(), Eq(sizeof(uint64_t)));
    ASSERT_THAT(static_config.response_message_type_details().payload().alignment(), Eq(alignof(uint64_t)));
    ASSERT_THAT(static_config.response_message_type_details().payload().type_name(), StrEq("u64"));
    ASSERT_THAT(static_config.has_safe_overflow_for_requests(), Eq(false));
    ASSERT_THAT(static_config.has_safe_overflow_for_responses(), Eq(false));
    ASSERT_THAT(static_config.max_active_requests_per_client(), Eq(ACTIVE_REQUESTS_PER_CLIENT));
    ASSERT_THAT(static_config.max_response_buffer_size(), Eq(MAX_RESPONSE_BUFFER_SIZE));
    ASSERT_THAT(static_config.max_borrowed_responses_per_pending_responses(), Eq(MAX_BORROWED_RESPONSES));
    ASSERT_THAT(static_config.max_loaned_requests(), Eq(MAX_LOANED_REQUESTS));
    ASSERT_THAT(static_config.does_support_fire_and_forget_requests(), Eq(false));
}

TYPED_TEST(ServiceRequestResponseTest, open_fails_with_incompatible_client_requirement) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    constexpr uint64_t NUMBER_OF_CLIENTS = 11;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto service = node.service_builder(service_name)
                       .template request_response<uint64_t, uint64_t>()
                       .max_clients(NUMBER_OF_CLIENTS)
                       .create()
                       .expect("");

    auto service_fail = node.service_builder(service_name)
                            .template request_response<uint64_t, uint64_t>()
                            .max_clients(NUMBER_OF_CLIENTS + 1)
                            .open();

    ASSERT_TRUE(service_fail.has_error());
    ASSERT_THAT(service_fail.error(), Eq(RequestResponseOpenError::DoesNotSupportRequestedAmountOfClients));
}

TYPED_TEST(ServiceRequestResponseTest, open_fails_with_incompatible_server_requirement) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    constexpr uint64_t NUMBER_OF_SERVERS = 12;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto service = node.service_builder(service_name)
                       .template request_response<uint64_t, uint64_t>()
                       .max_servers(NUMBER_OF_SERVERS)
                       .create()
                       .expect("");

    auto service_fail = node.service_builder(service_name)
                            .template request_response<uint64_t, uint64_t>()
                            .max_servers(NUMBER_OF_SERVERS + 1)
                            .open();

    ASSERT_TRUE(service_fail.has_error());
    ASSERT_THAT(service_fail.error(), Eq(RequestResponseOpenError::DoesNotSupportRequestedAmountOfServers));
}

TYPED_TEST(ServiceRequestResponseTest, client_applies_unable_to_deliver_strategy) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto service =
        node.service_builder(service_name).template request_response<uint64_t, uint64_t>().create().expect("");

    auto sut_client_1 =
        service.client_builder().unable_to_deliver_strategy(UnableToDeliverStrategy::Block).create().expect("");
    auto sut_client_2 =
        service.client_builder().unable_to_deliver_strategy(UnableToDeliverStrategy::DiscardSample).create().expect("");

    ASSERT_THAT(sut_client_1.unable_to_deliver_strategy(), Eq(UnableToDeliverStrategy::Block));
    ASSERT_THAT(sut_client_2.unable_to_deliver_strategy(), Eq(UnableToDeliverStrategy::DiscardSample));
}

TYPED_TEST(ServiceRequestResponseTest, client_applies_initial_max_slice_length) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();
    constexpr uint64_t INITIAL_MAX_SLICE_LEN = 1990;

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto service = node.service_builder(service_name)
                       .template request_response<iox::Slice<uint64_t>, uint64_t>()
                       .create()
                       .expect("");

    auto sut_client = service.client_builder().initial_max_slice_len(INITIAL_MAX_SLICE_LEN).create().expect("");

    ASSERT_THAT(sut_client.initial_max_slice_len(), Eq(INITIAL_MAX_SLICE_LEN));
}

TYPED_TEST(ServiceRequestResponseTest, number_of_clients_servers_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto service =
        node.service_builder(service_name).template request_response<uint64_t, uint64_t>().create().expect("");

    ASSERT_THAT(service.dynamic_config().number_of_clients(), Eq(0));
    ASSERT_THAT(service.dynamic_config().number_of_servers(), Eq(0));

    {
        auto sut_client = service.client_builder().create().expect("");
        ASSERT_THAT(service.dynamic_config().number_of_clients(), Eq(1));
        ASSERT_THAT(service.dynamic_config().number_of_servers(), Eq(0));

        auto sut_server = service.server_builder().create().expect("");
        ASSERT_THAT(service.dynamic_config().number_of_clients(), Eq(1));
        ASSERT_THAT(service.dynamic_config().number_of_servers(), Eq(1));
    }

    ASSERT_THAT(service.dynamic_config().number_of_clients(), Eq(0));
    ASSERT_THAT(service.dynamic_config().number_of_servers(), Eq(0));
}

TYPED_TEST(ServiceRequestResponseTest, create_with_attributes_sets_attributes) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    auto key = Attribute::Key("nice key");
    auto value = Attribute::Value("with a shiny value");
    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto service_create = node.service_builder(service_name)
                              .template request_response<uint64_t, uint64_t>()
                              .create_with_attributes(AttributeSpecifier().define(key, value))
                              .expect("");

    auto service_open =
        node.service_builder(service_name).template request_response<uint64_t, uint64_t>().open().expect("");


    auto attributes_create = service_create.attributes();
    auto attributes_open = service_open.attributes();

    ASSERT_THAT(attributes_create.number_of_attributes(), Eq(1));
    ASSERT_THAT(attributes_create[0].key(), Eq(key));
    ASSERT_THAT(attributes_create[0].value(), Eq(value));

    ASSERT_THAT(attributes_open.number_of_attributes(), Eq(1));
    ASSERT_THAT(attributes_open[0].key(), Eq(key));
    ASSERT_THAT(attributes_open[0].value(), Eq(value));
}

TYPED_TEST(ServiceRequestResponseTest, open_fails_when_attributes_are_incompatible) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    auto key = Attribute::Key("which song does hypnotoad sing?");
    auto value = Attribute::Value("is it 'all my hypnoflies'?");
    auto missing_key = Attribute::Key("no it's 'nala-la-la-la'!");
    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto service_create = node.service_builder(service_name)
                              .template request_response<uint64_t, uint64_t>()
                              .open_or_create_with_attributes(AttributeVerifier().require(key, value))
                              .expect("");

    auto service_open_or_create =
        node.service_builder(service_name)
            .template request_response<uint64_t, uint64_t>()
            .open_or_create_with_attributes(AttributeVerifier().require(key, value).require_key(missing_key));

    ASSERT_THAT(service_open_or_create.has_error(), Eq(true));
    ASSERT_THAT(service_open_or_create.error(), Eq(RequestResponseOpenOrCreateError::OpenIncompatibleAttributes));

    auto service_open = node.service_builder(service_name)
                            .template request_response<uint64_t, uint64_t>()
                            .open_with_attributes(AttributeVerifier().require(key, value).require_key(missing_key));

    ASSERT_THAT(service_open.has_error(), Eq(true));
    ASSERT_THAT(service_open.error(), Eq(RequestResponseOpenError::IncompatibleAttributes));
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

TYPED_TEST(ServiceRequestResponseTest, opening_existing_service_with_set_payload_type_name_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut_create =
        node.service_builder(service_name).template request_response<Payload, Payload>().create().expect("");
    auto sut_open = node.service_builder(service_name).template request_response<Payload, Payload>().open();
    ASSERT_FALSE(sut_open.has_error());
}

TYPED_TEST(ServiceRequestResponseTest,
           opening_existing_service_with_different_payload_but_same_set_payload_type_name_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut_create =
        node.service_builder(service_name).template request_response<Payload, Payload>().create().expect("");
    auto sut_open = node.service_builder(service_name)
                        .template request_response<DifferentPayloadWithSameTypeName, DifferentPayloadWithSameTypeName>()
                        .open();
    ASSERT_FALSE(sut_open.has_error());
}

TYPED_TEST(ServiceRequestResponseTest, opening_existing_service_without_payload_type_name_fails) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    const auto service_name_req = iox2_testing::generate_service_name();
    const auto service_name_res = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut_create_req =
        node.service_builder(service_name_req).template request_response<Payload, uint64_t>().create().expect("");
    auto sut_create_res =
        node.service_builder(service_name_res).template request_response<uint64_t, Payload>().create().expect("");

    struct Payload {
        int32_t x;
        double y;
    };

    auto sut_open_req = node.service_builder(service_name_req).template request_response<Payload, uint64_t>().open();
    ASSERT_TRUE(sut_open_req.has_error());
    EXPECT_EQ(sut_open_req.error(), RequestResponseOpenError::IncompatibleRequestType);
    auto sut_open_res = node.service_builder(service_name_res).template request_response<uint64_t, Payload>().open();
    ASSERT_TRUE(sut_open_res.has_error());
    EXPECT_EQ(sut_open_res.error(), RequestResponseOpenError::IncompatibleResponseType);
}

TYPED_TEST(ServiceRequestResponseTest,
           opening_existing_service_with_same_payload_but_different_payload_type_name_fails) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut_create =
        node.service_builder(service_name).template request_response<Payload, Payload>().create().expect("");

    auto sut_open_req = node.service_builder(service_name).template request_response<other::Payload, Payload>().open();
    ASSERT_TRUE(sut_open_req.has_error());
    EXPECT_EQ(sut_open_req.error(), RequestResponseOpenError::IncompatibleRequestType);
    auto sut_open_res = node.service_builder(service_name).template request_response<Payload, other::Payload>().open();
    ASSERT_TRUE(sut_open_res.has_error());
    EXPECT_EQ(sut_open_res.error(), RequestResponseOpenError::IncompatibleResponseType);
}

TYPED_TEST(ServiceRequestResponseTest, opening_existing_service_with_same_payload_type_name_but_different_size_fails) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut_create =
        node.service_builder(service_name).template request_response<Payload, Payload>().create().expect("");

    auto sut_open_req = node.service_builder(service_name)
                            .template request_response<PayloadWithSameTypeNameButDifferentSize, Payload>()
                            .open();
    ASSERT_TRUE(sut_open_req.has_error());
    EXPECT_EQ(sut_open_req.error(), RequestResponseOpenError::IncompatibleRequestType);
    auto sut_open_res = node.service_builder(service_name)
                            .template request_response<Payload, PayloadWithSameTypeNameButDifferentSize>()
                            .open();
    ASSERT_TRUE(sut_open_res.has_error());
    EXPECT_EQ(sut_open_res.error(), RequestResponseOpenError::IncompatibleResponseType);
}

TYPED_TEST(ServiceRequestResponseTest,
           opening_existing_service_with_same_payload_type_name_but_different_alignment_fails) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut_create =
        node.service_builder(service_name).template request_response<Payload, Payload>().create().expect("");

    auto sut_open_req = node.service_builder(service_name)
                            .template request_response<PayloadWithSameTypeNameButDifferentAlignment, Payload>()
                            .open();
    ASSERT_TRUE(sut_open_req.has_error());
    EXPECT_EQ(sut_open_req.error(), RequestResponseOpenError::IncompatibleRequestType);
    auto sut_open_res = node.service_builder(service_name)
                            .template request_response<Payload, PayloadWithSameTypeNameButDifferentAlignment>()
                            .open();
    ASSERT_TRUE(sut_open_res.has_error());
    EXPECT_EQ(sut_open_res.error(), RequestResponseOpenError::IncompatibleResponseType);
}

TYPED_TEST(ServiceRequestResponseTest, opening_existing_service_with_set_user_header_type_name_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut_create = node.service_builder(service_name)
                          .template request_response<uint8_t, uint8_t>()
                          .template request_user_header<CustomHeader>()
                          .template response_user_header<CustomHeader>()
                          .create()
                          .expect("");
    auto sut_open = node.service_builder(service_name)
                        .template request_response<uint8_t, uint8_t>()
                        .template request_user_header<CustomHeader>()
                        .template response_user_header<CustomHeader>()
                        .open();
    ASSERT_FALSE(sut_open.has_error());
}

TYPED_TEST(ServiceRequestResponseTest,
           opening_existing_service_with_different_header_but_same_set_user_header_type_name_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut_create = node.service_builder(service_name)
                          .template request_response<uint8_t, uint8_t>()
                          .template request_user_header<CustomHeader>()
                          .template response_user_header<CustomHeader>()
                          .create()
                          .expect("");
    auto sut_open = node.service_builder(service_name)
                        .template request_response<uint8_t, uint8_t>()
                        .template request_user_header<DifferentCustomHeaderWithSameTypeName>()
                        .template response_user_header<DifferentCustomHeaderWithSameTypeName>()
                        .open();
    ASSERT_FALSE(sut_open.has_error());
}

TYPED_TEST(ServiceRequestResponseTest, opening_existing_service_without_user_header_type_name_fails) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    const auto service_name_req = iox2_testing::generate_service_name();
    const auto service_name_res = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut_create_req = node.service_builder(service_name_req)
                              .template request_response<uint8_t, uint8_t>()
                              .template request_user_header<CustomHeader>()
                              .create()
                              .expect("");
    auto sut_create_res = node.service_builder(service_name_res)
                              .template request_response<uint8_t, uint8_t>()
                              .template response_user_header<CustomHeader>()
                              .create()
                              .expect("");

    struct CustomHeader {
        uint64_t a;
        uint8_t b;
    };
    auto sut_open_req = node.service_builder(service_name_req)
                            .template request_response<uint8_t, uint8_t>()
                            .template request_user_header<CustomHeader>()
                            .open();
    ASSERT_TRUE(sut_open_req.has_error());
    EXPECT_EQ(sut_open_req.error(), RequestResponseOpenError::IncompatibleRequestType);
    auto sut_open_res = node.service_builder(service_name_res)
                            .template request_response<uint8_t, uint8_t>()
                            .template response_user_header<CustomHeader>()
                            .open();
    ASSERT_TRUE(sut_open_res.has_error());
    EXPECT_EQ(sut_open_res.error(), RequestResponseOpenError::IncompatibleResponseType);
}

TYPED_TEST(ServiceRequestResponseTest,
           opening_existing_service_with_same_header_but_different_user_header_type_name_fails) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut_create = node.service_builder(service_name)
                          .template request_response<uint8_t, uint8_t>()
                          .template request_user_header<CustomHeader>()
                          .template response_user_header<CustomHeader>()
                          .create()
                          .expect("");

    auto sut_open_req = node.service_builder(service_name)
                            .template request_response<uint8_t, uint8_t>()
                            .template request_user_header<other::CustomHeader>()
                            .template response_user_header<CustomHeader>()
                            .open();
    ASSERT_TRUE(sut_open_req.has_error());
    EXPECT_EQ(sut_open_req.error(), RequestResponseOpenError::IncompatibleRequestType);

    auto sut_open_res = node.service_builder(service_name)
                            .template request_response<uint8_t, uint8_t>()
                            .template request_user_header<CustomHeader>()
                            .template response_user_header<other::CustomHeader>()
                            .open();
    ASSERT_TRUE(sut_open_res.has_error());
    EXPECT_EQ(sut_open_res.error(), RequestResponseOpenError::IncompatibleResponseType);
}

TYPED_TEST(ServiceRequestResponseTest, opening_existing_service_with_same_header_type_but_different_size_fails) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut_create = node.service_builder(service_name)
                          .template request_response<uint8_t, uint8_t>()
                          .template request_user_header<CustomHeader>()
                          .template response_user_header<CustomHeader>()
                          .create()
                          .expect("");

    auto sut_open_req = node.service_builder(service_name)
                            .template request_response<uint8_t, uint8_t>()
                            .template request_user_header<CustomHeaderWithSameTypeNameButDifferentSize>()
                            .template response_user_header<CustomHeader>()
                            .open();
    ASSERT_TRUE(sut_open_req.has_error());
    EXPECT_EQ(sut_open_req.error(), RequestResponseOpenError::IncompatibleRequestType);

    auto sut_open_res = node.service_builder(service_name)
                            .template request_response<uint8_t, uint8_t>()
                            .template request_user_header<CustomHeader>()
                            .template response_user_header<CustomHeaderWithSameTypeNameButDifferentSize>()
                            .open();
    ASSERT_TRUE(sut_open_res.has_error());
    EXPECT_EQ(sut_open_res.error(), RequestResponseOpenError::IncompatibleResponseType);
}

TYPED_TEST(ServiceRequestResponseTest, opening_existing_service_with_same_header_type_but_different_alignment_fails) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut_create = node.service_builder(service_name)
                          .template request_response<uint8_t, uint8_t>()
                          .template request_user_header<CustomHeader>()
                          .template response_user_header<CustomHeader>()
                          .create()
                          .expect("");

    auto sut_open_req = node.service_builder(service_name)
                            .template request_response<uint8_t, uint8_t>()
                            .template request_user_header<CustomHeaderWithSameTypeNameButDifferentAlignment>()
                            .template response_user_header<CustomHeader>()
                            .open();
    ASSERT_TRUE(sut_open_req.has_error());
    EXPECT_EQ(sut_open_req.error(), RequestResponseOpenError::IncompatibleRequestType);

    auto sut_open_res = node.service_builder(service_name)
                            .template request_response<uint8_t, uint8_t>()
                            .template request_user_header<CustomHeader>()
                            .template response_user_header<CustomHeaderWithSameTypeNameButDifferentAlignment>()
                            .open();
    ASSERT_TRUE(sut_open_res.has_error());
    EXPECT_EQ(sut_open_res.error(), RequestResponseOpenError::IncompatibleResponseType);
}

TYPED_TEST(ServiceRequestResponseTest, PayloadTypeNameIsSetToRustPendantForFixedSizeIntegersAndBoolAndSlicesOfThem) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    const auto service_name = iox2_testing::generate_service_name();
    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    {
        auto service =
            node.service_builder(service_name).template request_response<uint8_t, uint8_t>().create().expect("");
        auto static_config = service.static_config();
        ASSERT_THAT(static_config.request_message_type_details().payload().type_name(), StrEq("u8"));
        ASSERT_THAT(static_config.response_message_type_details().payload().type_name(), StrEq("u8"));
    }
    {
        auto service =
            node.service_builder(service_name).template request_response<uint16_t, uint16_t>().create().expect("");
        auto static_config = service.static_config();
        ASSERT_THAT(static_config.request_message_type_details().payload().type_name(), StrEq("u16"));
        ASSERT_THAT(static_config.response_message_type_details().payload().type_name(), StrEq("u16"));
    }
    {
        auto service =
            node.service_builder(service_name).template request_response<uint32_t, uint32_t>().create().expect("");
        auto static_config = service.static_config();
        ASSERT_THAT(static_config.request_message_type_details().payload().type_name(), StrEq("u32"));
        ASSERT_THAT(static_config.response_message_type_details().payload().type_name(), StrEq("u32"));
    }
    {
        auto service =
            node.service_builder(service_name).template request_response<uint64_t, uint64_t>().create().expect("");
        auto static_config = service.static_config();
        ASSERT_THAT(static_config.request_message_type_details().payload().type_name(), StrEq("u64"));
        ASSERT_THAT(static_config.response_message_type_details().payload().type_name(), StrEq("u64"));
    }
    {
        auto service =
            node.service_builder(service_name).template request_response<int8_t, int8_t>().create().expect("");
        auto static_config = service.static_config();
        ASSERT_THAT(static_config.request_message_type_details().payload().type_name(), StrEq("i8"));
        ASSERT_THAT(static_config.response_message_type_details().payload().type_name(), StrEq("i8"));
    }
    {
        auto service =
            node.service_builder(service_name).template request_response<int16_t, int16_t>().create().expect("");
        auto static_config = service.static_config();
        ASSERT_THAT(static_config.request_message_type_details().payload().type_name(), StrEq("i16"));
        ASSERT_THAT(static_config.response_message_type_details().payload().type_name(), StrEq("i16"));
    }
    {
        auto service =
            node.service_builder(service_name).template request_response<int32_t, int32_t>().create().expect("");
        auto static_config = service.static_config();
        ASSERT_THAT(static_config.request_message_type_details().payload().type_name(), StrEq("i32"));
        ASSERT_THAT(static_config.response_message_type_details().payload().type_name(), StrEq("i32"));
    }
    {
        auto service =
            node.service_builder(service_name).template request_response<int64_t, int64_t>().create().expect("");
        auto static_config = service.static_config();
        ASSERT_THAT(static_config.request_message_type_details().payload().type_name(), StrEq("i64"));
        ASSERT_THAT(static_config.response_message_type_details().payload().type_name(), StrEq("i64"));
    }
    {
        auto service = node.service_builder(service_name).template request_response<float, float>().create().expect("");
        auto static_config = service.static_config();
        ASSERT_THAT(static_config.request_message_type_details().payload().type_name(), StrEq("f32"));
        ASSERT_THAT(static_config.response_message_type_details().payload().type_name(), StrEq("f32"));
    }
    {
        auto service =
            node.service_builder(service_name).template request_response<double, double>().create().expect("");
        auto static_config = service.static_config();
        ASSERT_THAT(static_config.request_message_type_details().payload().type_name(), StrEq("f64"));
        ASSERT_THAT(static_config.response_message_type_details().payload().type_name(), StrEq("f64"));
    }
    {
        auto service = node.service_builder(service_name).template request_response<bool, bool>().create().expect("");
        auto static_config = service.static_config();
        ASSERT_THAT(static_config.request_message_type_details().payload().type_name(), StrEq("bool"));
        ASSERT_THAT(static_config.response_message_type_details().payload().type_name(), StrEq("bool"));
    }
    {
        auto service = node.service_builder(service_name)
                           .template request_response<iox::Slice<uint8_t>, iox::Slice<uint8_t>>()
                           .create()
                           .expect("");
        auto static_config = service.static_config();
        ASSERT_THAT(static_config.request_message_type_details().payload().type_name(), StrEq("u8"));
        ASSERT_THAT(static_config.response_message_type_details().payload().type_name(), StrEq("u8"));
    }
    {
        auto service = node.service_builder(service_name)
                           .template request_response<iox::Slice<uint16_t>, iox::Slice<uint16_t>>()
                           .create()
                           .expect("");
        auto static_config = service.static_config();
        ASSERT_THAT(static_config.request_message_type_details().payload().type_name(), StrEq("u16"));
        ASSERT_THAT(static_config.response_message_type_details().payload().type_name(), StrEq("u16"));
    }
    {
        auto service = node.service_builder(service_name)
                           .template request_response<iox::Slice<uint32_t>, iox::Slice<uint32_t>>()
                           .create()
                           .expect("");
        auto static_config = service.static_config();
        ASSERT_THAT(static_config.request_message_type_details().payload().type_name(), StrEq("u32"));
        ASSERT_THAT(static_config.response_message_type_details().payload().type_name(), StrEq("u32"));
    }
    {
        auto service = node.service_builder(service_name)
                           .template request_response<iox::Slice<uint64_t>, iox::Slice<uint64_t>>()
                           .create()
                           .expect("");
        auto static_config = service.static_config();
        ASSERT_THAT(static_config.request_message_type_details().payload().type_name(), StrEq("u64"));
        ASSERT_THAT(static_config.response_message_type_details().payload().type_name(), StrEq("u64"));
    }
    {
        auto service = node.service_builder(service_name)
                           .template request_response<iox::Slice<int8_t>, iox::Slice<int8_t>>()
                           .create()
                           .expect("");
        auto static_config = service.static_config();
        ASSERT_THAT(static_config.request_message_type_details().payload().type_name(), StrEq("i8"));
        ASSERT_THAT(static_config.response_message_type_details().payload().type_name(), StrEq("i8"));
    }
    {
        auto service = node.service_builder(service_name)
                           .template request_response<iox::Slice<int16_t>, iox::Slice<int16_t>>()
                           .create()
                           .expect("");
        auto static_config = service.static_config();
        ASSERT_THAT(static_config.request_message_type_details().payload().type_name(), StrEq("i16"));
        ASSERT_THAT(static_config.response_message_type_details().payload().type_name(), StrEq("i16"));
    }
    {
        auto service = node.service_builder(service_name)
                           .template request_response<iox::Slice<int32_t>, iox::Slice<int32_t>>()
                           .create()
                           .expect("");
        auto static_config = service.static_config();
        ASSERT_THAT(static_config.request_message_type_details().payload().type_name(), StrEq("i32"));
        ASSERT_THAT(static_config.response_message_type_details().payload().type_name(), StrEq("i32"));
    }
    {
        auto service = node.service_builder(service_name)
                           .template request_response<iox::Slice<int64_t>, iox::Slice<int64_t>>()
                           .create()
                           .expect("");
        auto static_config = service.static_config();
        ASSERT_THAT(static_config.request_message_type_details().payload().type_name(), StrEq("i64"));
        ASSERT_THAT(static_config.response_message_type_details().payload().type_name(), StrEq("i64"));
    }
    {
        auto service = node.service_builder(service_name)
                           .template request_response<iox::Slice<float>, iox::Slice<float>>()
                           .create()
                           .expect("");
        auto static_config = service.static_config();
        ASSERT_THAT(static_config.request_message_type_details().payload().type_name(), StrEq("f32"));
        ASSERT_THAT(static_config.response_message_type_details().payload().type_name(), StrEq("f32"));
    }
    {
        auto service = node.service_builder(service_name)
                           .template request_response<iox::Slice<double>, iox::Slice<double>>()
                           .create()
                           .expect("");
        auto static_config = service.static_config();
        ASSERT_THAT(static_config.request_message_type_details().payload().type_name(), StrEq("f64"));
        ASSERT_THAT(static_config.response_message_type_details().payload().type_name(), StrEq("f64"));
    }
    {
        auto service = node.service_builder(service_name)
                           .template request_response<iox::Slice<bool>, iox::Slice<bool>>()
                           .create()
                           .expect("");
        auto static_config = service.static_config();
        ASSERT_THAT(static_config.request_message_type_details().payload().type_name(), StrEq("bool"));
        ASSERT_THAT(static_config.response_message_type_details().payload().type_name(), StrEq("bool"));
    }
}

TYPED_TEST(ServiceRequestResponseTest, PayloadTypeNameIsSetToInnerTypeNameIfProvided) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    const auto service_name = iox2_testing::generate_service_name();
    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto service = node.service_builder(service_name)
                       .template request_response<iox::Slice<Payload>, iox::Slice<Payload>>()
                       .create()
                       .expect("");

    auto static_config = service.static_config();
    ASSERT_THAT(static_config.request_message_type_details().payload().type_name(), StrEq("Payload"));
    ASSERT_THAT(static_config.response_message_type_details().payload().type_name(), StrEq("Payload"));
}

TYPED_TEST(ServiceRequestResponseTest, service_id_is_unique_per_service) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    const auto service_name_1 = iox2_testing::generate_service_name();
    const auto service_name_2 = iox2_testing::generate_service_name();
    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");

    auto service_1_create =
        node.service_builder(service_name_1).template request_response<uint64_t, uint64_t>().create().expect("");
    auto service_1_open =
        node.service_builder(service_name_1).template request_response<uint64_t, uint64_t>().open().expect("");
    auto service_2 =
        node.service_builder(service_name_2).template request_response<uint64_t, uint64_t>().create().expect("");

    ASSERT_THAT(service_1_create.service_id().c_str(), StrEq(service_1_open.service_id().c_str()));
    ASSERT_THAT(service_1_create.service_id().c_str(), Not(StrEq(service_2.service_id().c_str())));
}
// END tests for customizable payload and user header type name
} // namespace
