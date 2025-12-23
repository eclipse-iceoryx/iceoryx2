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

#include "iox2/bb/optional.hpp"
#include "iox2/node.hpp"
#include "iox2/service.hpp"

#include "test.hpp"
#include <array>
#include <gtest/gtest.h>

namespace {
using namespace iox2;

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
            .value());

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();

    {
        auto sut = node.service_builder(service_name).template request_response<uint64_t, uint64_t>().create().value();

        ASSERT_TRUE(
            Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::RequestResponse)
                .value());
    }

    ASSERT_FALSE(
        Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::Event).value());
    ASSERT_FALSE(
        Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::PublishSubscribe)
            .value());
}

TYPED_TEST(ServiceRequestResponseTest, service_name_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto sut = node.service_builder(service_name).template request_response<uint64_t, uint64_t>().create().value();

    ASSERT_THAT(sut.name().to_string().unchecked_access().c_str(),
                StrEq(service_name.to_string().unchecked_access().c_str()));
}

//NOLINTBEGIN(readability-function-cognitive-complexity), false positive caused by ASSERT_THAT
TYPED_TEST(ServiceRequestResponseTest, list_service_nodes_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto node_name_1 = NodeName::create("is there any of the herring left?").value();
    const auto node_name_2 = NodeName::create("nala and octo-wolf asked in unison").value();
    const auto service_name = iox2_testing::generate_service_name();

    auto node_1 = NodeBuilder().name(node_name_1).create<SERVICE_TYPE>().value();
    auto node_2 = NodeBuilder().name(node_name_2).create<SERVICE_TYPE>().value();

    auto sut_1 = node_1.service_builder(service_name).template request_response<uint64_t, uint64_t>().create().value();
    auto sut_2 = node_2.service_builder(service_name).template request_response<uint64_t, uint64_t>().open().value();

    auto counter = 0;
    auto verify_node = [&](const AliveNodeView<SERVICE_TYPE>& node_view) -> auto {
        counter++;
        if (node_view.id() == node_1.id()) {
            ASSERT_THAT(node_view.details()->name().to_string().unchecked_access().c_str(),
                        StrEq(node_1.name().to_string().unchecked_access().c_str()));
        } else {
            ASSERT_THAT(node_view.details()->name().to_string().unchecked_access().c_str(),
                        StrEq(node_2.name().to_string().unchecked_access().c_str()));
        }
    };

    auto result = sut_1.nodes([&](auto node_state) -> CallbackProgression {
        node_state.alive(verify_node);

        node_state.dead([](const auto&) -> auto { ASSERT_TRUE(false); });
        node_state.inaccessible([](const auto&) -> auto { ASSERT_TRUE(false); });
        node_state.undefined([](const auto&) -> auto { ASSERT_TRUE(false); });

        return CallbackProgression::Continue;
    });

    ASSERT_THAT(result.has_value(), Eq(true));
    ASSERT_THAT(counter, Eq(2));
}
//NOLINTEND(readability-function-cognitive-complexity)

TYPED_TEST(ServiceRequestResponseTest, creating_existing_service_fails) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    ASSERT_FALSE(
        Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::RequestResponse)
            .value());

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto sut = node.service_builder(service_name).template request_response<uint64_t, uint64_t>().create().value();
    auto sut_2 = node.service_builder(service_name).template request_response<uint64_t, uint64_t>().create();

    ASSERT_FALSE(sut_2.has_value());
    ASSERT_THAT(sut_2.error(), Eq(RequestResponseCreateError::AlreadyExists));
}

TYPED_TEST(ServiceRequestResponseTest, open_or_create_service_does_exist) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    ASSERT_FALSE(
        Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::RequestResponse)
            .value());

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();

    {
        auto sut = bb::Optional<PortFactoryRequestResponse<SERVICE_TYPE, uint64_t, void, uint64_t, void>>(
            node.service_builder(service_name)
                .template request_response<uint64_t, uint64_t>()
                .open_or_create()
                .value());

        ASSERT_TRUE(
            Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::RequestResponse)
                .value());

        auto sut_2 = bb::Optional<PortFactoryRequestResponse<SERVICE_TYPE, uint64_t, void, uint64_t, void>>(
            node.service_builder(service_name)
                .template request_response<uint64_t, uint64_t>()
                .open_or_create()
                .value());

        ASSERT_TRUE(
            Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::RequestResponse)
                .value());

        sut.reset();

        ASSERT_TRUE(
            Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::RequestResponse)
                .value());

        sut_2.reset();
    }

    ASSERT_FALSE(
        Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::RequestResponse)
            .value());
}

TYPED_TEST(ServiceRequestResponseTest, opening_non_existing_service_fails) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto sut = node.service_builder(service_name).template request_response<uint64_t, uint64_t>().open();
    ASSERT_FALSE(sut.has_value());
    ASSERT_THAT(sut.error(), Eq(RequestResponseOpenError::DoesNotExist));
}

TYPED_TEST(ServiceRequestResponseTest, opening_existing_service_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto sut_create =
        node.service_builder(service_name).template request_response<uint64_t, uint64_t>().create().value();
    auto sut = node.service_builder(service_name).template request_response<uint64_t, uint64_t>().open();
    ASSERT_TRUE(sut.has_value());
}

TYPED_TEST(ServiceRequestResponseTest, opening_existing_service_with_wrong_payload_type_fails) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto sut_create =
        node.service_builder(service_name).template request_response<uint64_t, uint64_t>().create().value();

    auto sut1 = node.service_builder(service_name).template request_response<double, uint64_t>().open();
    ASSERT_FALSE(sut1.has_value());
    ASSERT_THAT(sut1.error(), Eq(RequestResponseOpenError::IncompatibleRequestType));

    auto sut2 = node.service_builder(service_name).template request_response<uint64_t, double>().open();
    ASSERT_FALSE(sut2.has_value());
    ASSERT_THAT(sut2.error(), Eq(RequestResponseOpenError::IncompatibleResponseType));
}

TYPED_TEST(ServiceRequestResponseTest, opening_existing_service_with_wrong_user_header_type_fails) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto sut_create = node.service_builder(service_name)
                          .template request_response<uint64_t, uint64_t>()
                          .template request_user_header<uint64_t>()
                          .template response_user_header<uint64_t>()
                          .create()
                          .value();

    auto sut1 = node.service_builder(service_name)
                    .template request_response<uint64_t, uint64_t>()
                    .template request_user_header<double>()
                    .template response_user_header<uint64_t>()
                    .open();
    ASSERT_FALSE(sut1.has_value());
    ASSERT_THAT(sut1.error(), Eq(RequestResponseOpenError::IncompatibleRequestType));

    auto sut2 = node.service_builder(service_name)
                    .template request_response<uint64_t, uint64_t>()
                    .template request_user_header<uint64_t>()
                    .template response_user_header<double>()
                    .open();
    ASSERT_FALSE(sut2.has_value());
    ASSERT_THAT(sut2.error(), Eq(RequestResponseOpenError::IncompatibleResponseType));
}

TYPED_TEST(ServiceRequestResponseTest, open_or_create_existing_service_with_wrong_payload_type_fails) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto sut_create =
        node.service_builder(service_name).template request_response<uint64_t, uint64_t>().create().value();

    auto sut1 = node.service_builder(service_name).template request_response<double, uint64_t>().open_or_create();
    ASSERT_FALSE(sut1.has_value());
    ASSERT_THAT(sut1.error(), Eq(RequestResponseOpenOrCreateError::OpenIncompatibleRequestType));

    auto sut2 = node.service_builder(service_name).template request_response<uint64_t, double>().open_or_create();
    ASSERT_FALSE(sut2.has_value());
    ASSERT_THAT(sut2.error(), Eq(RequestResponseOpenOrCreateError::OpenIncompatibleResponseType));
}

TYPED_TEST(ServiceRequestResponseTest, send_copy_and_receive_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service = node.service_builder(service_name).template request_response<uint64_t, uint64_t>().create().value();

    auto sut_client = service.client_builder().create().value();
    auto sut_server = service.server_builder().create().value();

    const uint64_t request_payload = 123;
    auto pending_response = sut_client.send_copy(request_payload);
    ASSERT_TRUE(pending_response.has_value());

    auto has_requests = sut_server.has_requests();
    ASSERT_TRUE(has_requests.has_value());
    EXPECT_TRUE(has_requests.value());
    auto active_request = sut_server.receive().value();
    ASSERT_TRUE(active_request.has_value());
    EXPECT_THAT(active_request->payload(), Eq(request_payload));

    const uint64_t response_payload = 234;
    auto sent_response = active_request->send_copy(response_payload);
    ASSERT_TRUE(sent_response.has_value());
    ASSERT_TRUE(pending_response->has_response());

    auto received_response = pending_response->receive().value();
    ASSERT_TRUE(received_response.has_value());
    EXPECT_THAT(received_response->payload(), Eq(response_payload));
}

TYPED_TEST(ServiceRequestResponseTest, loan_uninit_write_payload_send_receive_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service = node.service_builder(service_name).template request_response<uint64_t, uint64_t>().create().value();

    auto sut_client = service.client_builder().create().value();
    auto sut_server = service.server_builder().create().value();

    const uint64_t request_payload = 345;
    auto request_uninit = sut_client.loan_uninit().value();
    request_uninit.payload_mut() = request_payload;
    EXPECT_THAT(request_uninit.payload(), Eq(request_payload));
    auto pending_response = send(assume_init(std::move(request_uninit))).value();

    auto active_request = sut_server.receive().value();
    ASSERT_TRUE(active_request.has_value());
    EXPECT_THAT(active_request->payload(), Eq(request_payload));

    const uint64_t response_payload = 456;
    auto response_uninit = active_request->loan_uninit().value();
    response_uninit.payload_mut() = response_payload;
    EXPECT_THAT(response_uninit.payload(), Eq(response_payload));
    send(assume_init(std::move(response_uninit))).value();

    auto received_response = pending_response.receive().value();
    ASSERT_TRUE(received_response.has_value());
    EXPECT_THAT(received_response->payload(), Eq(response_payload));
}

TYPED_TEST(ServiceRequestResponseTest, loan_send_receive_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    struct Payload {
        uint64_t p { 3 };
    };

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service = node.service_builder(service_name).template request_response<Payload, Payload>().create().value();

    auto sut_client = service.client_builder().create().value();
    auto sut_server = service.server_builder().create().value();

    auto request = sut_client.loan().value();
    EXPECT_THAT(request.payload().p, Eq(3));

    auto pending_response = send(std::move(request)).value();
    EXPECT_THAT(pending_response.payload().p, Eq(3));

    auto active_request = sut_server.receive().value();
    ASSERT_TRUE(active_request.has_value());
    EXPECT_THAT(active_request.value().payload().p, Eq(3));

    auto response = active_request->loan().value();
    response.payload_mut().p = 0;
    send(std::move(response)).value();

    auto received_response = pending_response.receive().value();
    ASSERT_TRUE(received_response.has_value());
    EXPECT_THAT(received_response.value().payload().p, Eq(0));
}

TYPED_TEST(ServiceRequestResponseTest, loan_request_default_constructs_request_header) {
    constexpr uint64_t RAND_A = 123;
    constexpr uint32_t RAND_B = 456;
    using UserHeader = CustomTestHeader<RAND_A, RAND_B>;
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service = node.service_builder(service_name)
                       .template request_response<uint64_t, uint64_t>()
                       .template request_user_header<UserHeader>()
                       .create()
                       .value();

    auto client = service.client_builder().create().value();
    auto server = service.server_builder().create().value();

    auto sut = client.loan().value();
    ASSERT_THAT(sut.user_header(), Eq(UserHeader()));
}

TYPED_TEST(ServiceRequestResponseTest, loan_uninit_request_default_constructs_request_header) {
    constexpr uint64_t RAND_A = 1239;
    constexpr uint32_t RAND_B = 4569;
    using UserHeader = CustomTestHeader<RAND_A, RAND_B>;
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service = node.service_builder(service_name)
                       .template request_response<uint64_t, uint64_t>()
                       .template request_user_header<UserHeader>()
                       .create()
                       .value();

    auto client = service.client_builder().create().value();
    auto server = service.server_builder().create().value();

    auto sut = client.loan_uninit().value();
    ASSERT_THAT(sut.user_header(), Eq(UserHeader()));
}

TYPED_TEST(ServiceRequestResponseTest, loan_slice_request_default_constructs_request_header) {
    constexpr uint64_t RAND_A = 12399;
    constexpr uint32_t RAND_B = 45699;
    using UserHeader = CustomTestHeader<RAND_A, RAND_B>;
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service = node.service_builder(service_name)
                       .template request_response<bb::Slice<uint64_t>, uint64_t>()
                       .template request_user_header<UserHeader>()
                       .create()
                       .value();

    auto client = service.client_builder().create().value();
    auto server = service.server_builder().create().value();

    auto sut = client.loan_slice(1).value();
    ASSERT_THAT(sut.user_header(), Eq(UserHeader()));
}

TYPED_TEST(ServiceRequestResponseTest, loan_slice_uninit_request_default_constructs_request_header) {
    constexpr uint64_t RAND_A = 123991;
    constexpr uint32_t RAND_B = 456991;
    using UserHeader = CustomTestHeader<RAND_A, RAND_B>;
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service = node.service_builder(service_name)
                       .template request_response<bb::Slice<uint64_t>, uint64_t>()
                       .template request_user_header<UserHeader>()
                       .create()
                       .value();

    auto client = service.client_builder().create().value();
    auto server = service.server_builder().create().value();

    auto sut = client.loan_slice_uninit(1).value();
    ASSERT_THAT(sut.user_header(), Eq(UserHeader()));
}

TYPED_TEST(ServiceRequestResponseTest, loan_response_default_constructs_response_header) {
    constexpr uint64_t RAND_A = 1239917;
    constexpr uint32_t RAND_B = 4569917;
    using UserHeader = CustomTestHeader<RAND_A, RAND_B>;
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service = node.service_builder(service_name)
                       .template request_response<uint64_t, uint64_t>()
                       .template response_user_header<UserHeader>()
                       .create()
                       .value();

    auto client = service.client_builder().create().value();
    auto server = service.server_builder().create().value();

    auto pending_response = client.send_copy(0);
    auto active_request = server.receive().value().value();
    auto sut = active_request.loan().value();
    ASSERT_THAT(sut.user_header(), Eq(UserHeader()));
}

TYPED_TEST(ServiceRequestResponseTest, loan_uninit_response_default_constructs_response_header) {
    constexpr uint64_t RAND_A = 129917;
    constexpr uint32_t RAND_B = 459917;
    using UserHeader = CustomTestHeader<RAND_A, RAND_B>;
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service = node.service_builder(service_name)
                       .template request_response<uint64_t, uint64_t>()
                       .template response_user_header<UserHeader>()
                       .create()
                       .value();

    auto client = service.client_builder().create().value();
    auto server = service.server_builder().create().value();

    auto pending_response = client.send_copy(0);
    auto active_request = server.receive().value().value();
    auto sut = active_request.loan_uninit().value();
    ASSERT_THAT(sut.user_header(), Eq(UserHeader()));
}

TYPED_TEST(ServiceRequestResponseTest, loan_slice_response_default_constructs_response_header) {
    constexpr uint64_t RAND_A = 19917;
    constexpr uint32_t RAND_B = 49917;
    using UserHeader = CustomTestHeader<RAND_A, RAND_B>;
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service = node.service_builder(service_name)
                       .template request_response<uint64_t, bb::Slice<uint64_t>>()
                       .template response_user_header<UserHeader>()
                       .create()
                       .value();

    auto client = service.client_builder().create().value();
    auto server = service.server_builder().create().value();

    auto pending_response = client.send_copy(0);
    auto active_request = server.receive().value().value();
    auto sut = active_request.loan_slice(1).value();
    ASSERT_THAT(sut.user_header(), Eq(UserHeader()));
}

TYPED_TEST(ServiceRequestResponseTest, loan_slice_uninit_response_default_constructs_response_header) {
    constexpr uint64_t RAND_A = 199017;
    constexpr uint32_t RAND_B = 499017;
    using UserHeader = CustomTestHeader<RAND_A, RAND_B>;
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service = node.service_builder(service_name)
                       .template request_response<uint64_t, bb::Slice<uint64_t>>()
                       .template response_user_header<UserHeader>()
                       .create()
                       .value();

    auto client = service.client_builder().create().value();
    auto server = service.server_builder().create().value();

    auto pending_response = client.send_copy(0);
    auto active_request = server.receive().value().value();
    auto sut = active_request.loan_slice_uninit(1).value();
    ASSERT_THAT(sut.user_header(), Eq(UserHeader()));
}

struct DummyData {
    static constexpr uint64_t DEFAULT_VALUE_A = 42;
    static constexpr bool DEFAULT_VALUE_Z { false };
    uint64_t a { DEFAULT_VALUE_A };
    bool z { DEFAULT_VALUE_Z };
};

//NOLINTBEGIN(readability-function-cognitive-complexity), false positive caused by ASSERT_THAT
TYPED_TEST(ServiceRequestResponseTest, send_slice_copy_and_receive_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    constexpr auto SLICE_MAX_LENGTH = 10;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service = node.service_builder(service_name)
                       .template request_response<bb::Slice<DummyData>, bb::Slice<DummyData>>()
                       .create()
                       .value();

    auto sut_client = service.client_builder().initial_max_slice_len(SLICE_MAX_LENGTH).create().value();
    auto sut_server = service.server_builder().initial_max_slice_len(SLICE_MAX_LENGTH).create().value();

    iox2::legacy::UninitializedArray<DummyData, SLICE_MAX_LENGTH, iox2::legacy::ZeroedBuffer> elements;
    for (auto& item : elements) {
        new (&item) DummyData {};
    }
    auto payload = bb::ImmutableSlice<DummyData>(elements.begin(), SLICE_MAX_LENGTH);
    auto pending_response = sut_client.send_slice_copy(payload);
    ASSERT_TRUE(pending_response.has_value());
    EXPECT_THAT(pending_response->payload().number_of_elements(), Eq(SLICE_MAX_LENGTH));

    auto active_request = sut_server.receive().value();
    ASSERT_TRUE(active_request.has_value());
    auto received_request = std::move(active_request.value());

    auto iterations = 0;
    for (const auto& item : received_request.payload()) {
        ASSERT_THAT(item.a, Eq(DummyData::DEFAULT_VALUE_A));
        ASSERT_THAT(item.z, Eq(DummyData::DEFAULT_VALUE_Z));
        ++iterations;
    }
    EXPECT_THAT(received_request.payload().number_of_elements(), Eq(SLICE_MAX_LENGTH));
    EXPECT_THAT(iterations, Eq(SLICE_MAX_LENGTH));

    received_request.send_slice_copy(payload).value();

    auto received_response = pending_response->receive().value();
    ASSERT_TRUE(received_response.has_value());
    iterations = 0;
    for (const auto& item : received_response->payload()) {
        ASSERT_THAT(item.a, Eq(DummyData::DEFAULT_VALUE_A));
        ASSERT_THAT(item.z, Eq(DummyData::DEFAULT_VALUE_Z));
        ++iterations;
    }
    EXPECT_THAT(received_response->payload().number_of_elements(), Eq(SLICE_MAX_LENGTH));
    EXPECT_THAT(iterations, Eq(SLICE_MAX_LENGTH));
}
// NOLINTEND(readability-function-cognitive-complexity)

//NOLINTBEGIN(readability-function-cognitive-complexity), false positive caused by ASSERT_THAT
TYPED_TEST(ServiceRequestResponseTest, loan_slice_uninit_write_payload_send_receive_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    constexpr auto SLICE_MAX_LENGTH = 5;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service = node.service_builder(service_name)
                       .template request_response<bb::Slice<DummyData>, bb::Slice<DummyData>>()
                       .create()
                       .value();

    auto sut_client = service.client_builder().initial_max_slice_len(SLICE_MAX_LENGTH).create().value();
    auto sut_server = service.server_builder().initial_max_slice_len(SLICE_MAX_LENGTH).create().value();

    auto request_uninit = sut_client.loan_slice_uninit(SLICE_MAX_LENGTH);
    ASSERT_TRUE(request_uninit.has_value());
    EXPECT_THAT(request_uninit.value().payload().number_of_elements(), Eq(SLICE_MAX_LENGTH));

    iox2::legacy::UninitializedArray<DummyData, SLICE_MAX_LENGTH, iox2::legacy::ZeroedBuffer> elements;
    for (auto& item : elements) {
        new (&item) DummyData {};
    }
    auto payload = bb::ImmutableSlice<DummyData>(elements.begin(), SLICE_MAX_LENGTH);
    auto request = request_uninit->write_from_slice(payload);
    EXPECT_THAT(request.payload().number_of_elements(), Eq(SLICE_MAX_LENGTH));
    auto pending_response = send(std::move(request)).value();

    auto active_request = sut_server.receive().value();
    ASSERT_TRUE(active_request.has_value());
    auto received_request = std::move(active_request.value());
    auto iterations = 0;
    for (const auto& item : received_request.payload()) {
        ASSERT_THAT(item.a, Eq(DummyData::DEFAULT_VALUE_A));
        ASSERT_THAT(item.z, Eq(DummyData::DEFAULT_VALUE_Z));
        ++iterations;
    }
    EXPECT_THAT(received_request.payload().number_of_elements(), Eq(SLICE_MAX_LENGTH));
    EXPECT_THAT(iterations, Eq(SLICE_MAX_LENGTH));

    auto response_uninit = received_request.loan_slice_uninit(SLICE_MAX_LENGTH).value();
    auto response = response_uninit.write_from_slice(payload);
    iterations = 0;
    for (const auto& item : response.payload()) {
        ASSERT_THAT(item.a, Eq(DummyData::DEFAULT_VALUE_A));
        ASSERT_THAT(item.z, Eq(DummyData::DEFAULT_VALUE_Z));
        ++iterations;
    }
    EXPECT_THAT(response.payload().number_of_elements(), Eq(SLICE_MAX_LENGTH));
    EXPECT_THAT(iterations, Eq(SLICE_MAX_LENGTH));
    send(std::move(response)).value();

    auto received_response = pending_response.receive().value();
    ASSERT_TRUE(received_response.has_value());
    iterations = 0;
    for (const auto& item : received_response->payload()) {
        ASSERT_THAT(item.a, Eq(DummyData::DEFAULT_VALUE_A));
        ASSERT_THAT(item.z, Eq(DummyData::DEFAULT_VALUE_Z));
        ++iterations;
    }
    EXPECT_THAT(received_response->payload().number_of_elements(), Eq(SLICE_MAX_LENGTH));
    EXPECT_THAT(iterations, Eq(SLICE_MAX_LENGTH));
}
// NOLINTEND(readability-function-cognitive-complexity)

//NOLINTBEGIN(readability-function-cognitive-complexity), false positive caused by ASSERT_THAT
TYPED_TEST(ServiceRequestResponseTest, loan_slice_write_payload_send_receive_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    constexpr auto SLICE_MAX_LENGTH = 10;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service = node.service_builder(service_name)
                       .template request_response<bb::Slice<DummyData>, bb::Slice<DummyData>>()
                       .create()
                       .value();

    auto sut_client = service.client_builder().initial_max_slice_len(SLICE_MAX_LENGTH).create().value();
    auto sut_server = service.server_builder().initial_max_slice_len(SLICE_MAX_LENGTH).create().value();

    auto request = sut_client.loan_slice(SLICE_MAX_LENGTH);
    ASSERT_TRUE(request.has_value());
    EXPECT_THAT(request->payload().number_of_elements(), Eq(SLICE_MAX_LENGTH));

    auto pending_response = send(std::move(*request)).value();

    auto active_request = sut_server.receive().value();
    ASSERT_TRUE(active_request.has_value());
    auto received_request = std::move(active_request.value());
    auto iterations = 0;
    for (const auto& item : received_request.payload()) {
        ASSERT_THAT(item.a, Eq(DummyData::DEFAULT_VALUE_A));
        ASSERT_THAT(item.z, Eq(DummyData::DEFAULT_VALUE_Z));
        ++iterations;
    }
    EXPECT_THAT(received_request.payload().number_of_elements(), Eq(SLICE_MAX_LENGTH));
    EXPECT_THAT(iterations, Eq(SLICE_MAX_LENGTH));

    auto response = received_request.loan_slice(SLICE_MAX_LENGTH).value();
    send(std::move(response)).value();

    auto received_response = pending_response.receive().value();
    ASSERT_TRUE(received_response.has_value());
    iterations = 0;
    for (const auto& item : received_response->payload()) {
        ASSERT_THAT(item.a, Eq(DummyData::DEFAULT_VALUE_A));
        ASSERT_THAT(item.z, Eq(DummyData::DEFAULT_VALUE_Z));
        ++iterations;
    }
    EXPECT_THAT(received_response->payload().number_of_elements(), Eq(SLICE_MAX_LENGTH));
    EXPECT_THAT(iterations, Eq(SLICE_MAX_LENGTH));
}
// NOLINTEND(readability-function-cognitive-complexity)

TYPED_TEST(ServiceRequestResponseTest, write_payload_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service = node.service_builder(service_name).template request_response<uint64_t, uint64_t>().create().value();

    auto sut_client = service.client_builder().create().value();
    auto sut_server = service.server_builder().create().value();

    auto request_uninit = sut_client.loan_uninit().value();
    uint64_t request_payload = 3;
    auto request = request_uninit.write_payload(std::move(request_payload));
    EXPECT_THAT(request.payload(), Eq(request_payload));
    auto pending_response = send(std::move(request)).value();

    auto active_request = sut_server.receive().value();
    ASSERT_TRUE(active_request.has_value());
    EXPECT_THAT(active_request->payload(), Eq(request_payload));

    uint64_t response_payload = 4;
    auto response_uninit = active_request->loan_uninit().value();
    auto response = response_uninit.write_payload(std::move(response_payload));
    EXPECT_THAT(response.payload(), Eq(response_payload));
    send(std::move(response)).value();

    auto received_response = pending_response.receive().value();
    ASSERT_TRUE(received_response.has_value());
    EXPECT_THAT(received_response->payload(), Eq(response_payload));
}

//NOLINTBEGIN(readability-function-cognitive-complexity), false positive caused by ASSERT_THAT
TYPED_TEST(ServiceRequestResponseTest, write_from_fn_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    constexpr auto SLICE_MAX_LENGTH = 10;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service = node.service_builder(service_name)
                       .template request_response<bb::Slice<DummyData>, bb::Slice<DummyData>>()
                       .create()
                       .value();

    auto sut_client = service.client_builder().initial_max_slice_len(SLICE_MAX_LENGTH).create().value();
    auto sut_server = service.server_builder().initial_max_slice_len(SLICE_MAX_LENGTH).create().value();

    auto request_uninit = sut_client.loan_slice_uninit(SLICE_MAX_LENGTH).value();
    EXPECT_THAT(request_uninit.payload().number_of_elements(), Eq(SLICE_MAX_LENGTH));

    auto request = request_uninit.write_from_fn(
        [](auto index) -> auto { return DummyData { DummyData::DEFAULT_VALUE_A + index, index % 2 == 0 }; });
    auto pending_response = send(std::move(request)).value();

    auto active_request = sut_server.receive().value();
    ASSERT_TRUE(active_request.has_value());
    auto received_request = std::move(active_request.value());
    uint64_t iterations = 0;
    for (const auto& item : received_request.payload()) {
        ASSERT_THAT(item.a, Eq(DummyData::DEFAULT_VALUE_A + iterations));
        ASSERT_THAT(item.z, Eq(iterations % 2 == 0));
        ++iterations;
    }
    EXPECT_THAT(received_request.payload().number_of_elements(), Eq(SLICE_MAX_LENGTH));
    EXPECT_THAT(iterations, Eq(SLICE_MAX_LENGTH));

    auto response_uninit = received_request.loan_slice_uninit(SLICE_MAX_LENGTH).value();
    auto response = response_uninit.write_from_fn(
        [](auto index) -> auto { return DummyData { DummyData::DEFAULT_VALUE_Z + index, index % 2 == 0 }; });
    send(std::move(response)).value();

    auto received_response = pending_response.receive().value();
    ASSERT_TRUE(received_response.has_value());
    iterations = 0;
    for (const auto& item : received_response->payload()) {
        ASSERT_THAT(item.a, Eq(DummyData::DEFAULT_VALUE_Z + iterations));
        ASSERT_THAT(item.z, Eq(iterations % 2 == 0));
        ++iterations;
    }
    EXPECT_THAT(received_response->payload().number_of_elements(), Eq(SLICE_MAX_LENGTH));
    EXPECT_THAT(iterations, Eq(SLICE_MAX_LENGTH));
}
// NOLINTEND(readability-function-cognitive-complexity)

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

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
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
                       .value();

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

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service = node.service_builder(service_name)
                       .template request_response<uint64_t, uint64_t>()
                       .max_clients(NUMBER_OF_CLIENTS)
                       .create()
                       .value();

    auto service_fail = node.service_builder(service_name)
                            .template request_response<uint64_t, uint64_t>()
                            .max_clients(NUMBER_OF_CLIENTS + 1)
                            .open();

    ASSERT_FALSE(service_fail.has_value());
    ASSERT_THAT(service_fail.error(), Eq(RequestResponseOpenError::DoesNotSupportRequestedAmountOfClients));
}

TYPED_TEST(ServiceRequestResponseTest, open_fails_with_incompatible_server_requirement) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    constexpr uint64_t NUMBER_OF_SERVERS = 12;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service = node.service_builder(service_name)
                       .template request_response<uint64_t, uint64_t>()
                       .max_servers(NUMBER_OF_SERVERS)
                       .create()
                       .value();

    auto service_fail = node.service_builder(service_name)
                            .template request_response<uint64_t, uint64_t>()
                            .max_servers(NUMBER_OF_SERVERS + 1)
                            .open();

    ASSERT_FALSE(service_fail.has_value());
    ASSERT_THAT(service_fail.error(), Eq(RequestResponseOpenError::DoesNotSupportRequestedAmountOfServers));
}

TYPED_TEST(ServiceRequestResponseTest, send_receive_with_user_header_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service = node.service_builder(service_name)
                       .template request_response<uint64_t, uint64_t>()
                       .template request_user_header<uint64_t>()
                       .template response_user_header<uint64_t>()
                       .create()
                       .value();

    auto sut_client = service.client_builder().create().value();
    auto sut_server = service.server_builder().create().value();

    const uint64_t request_user_header = 4;
    const uint64_t response_user_header = 1;

    auto request_uninit = sut_client.loan_uninit().value();
    request_uninit.user_header_mut() = request_user_header;
    EXPECT_THAT(request_uninit.user_header(), Eq(request_user_header));
    auto pending_response = send(assume_init(std::move(request_uninit))).value();
    EXPECT_THAT(pending_response.user_header(), Eq(request_user_header));

    auto active_request = sut_server.receive().value();
    ASSERT_TRUE(active_request.has_value());
    EXPECT_THAT(active_request->user_header(), Eq(request_user_header));

    auto response = active_request->loan().value();
    response.payload_mut() = 2;
    response.user_header_mut() = response_user_header;
    send(std::move(response)).value();

    auto received_response = pending_response.receive().value();
    ASSERT_TRUE(received_response.has_value());
    EXPECT_THAT(received_response->user_header(), Eq(response_user_header));

    auto response_uninit = active_request->loan_uninit().value();
    response_uninit.user_header_mut() = response_user_header;
    EXPECT_THAT(response_uninit.user_header(), Eq(response_user_header));
}

TYPED_TEST(ServiceRequestResponseTest, number_of_server_connections_is_set_correctly) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service = node.service_builder(service_name).template request_response<uint64_t, uint64_t>().create().value();

    auto server1 = service.server_builder().create().value();
    auto server2 = service.server_builder().create().value();
    auto client = service.client_builder().create().value();

    const uint64_t payload = 123;
    auto pending_response = client.send_copy(payload).value();
    EXPECT_THAT(pending_response.number_of_server_connections(), Eq(2));
}

TYPED_TEST(ServiceRequestResponseTest, server_applies_initial_max_slice_length) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();
    constexpr uint64_t INITIAL_MAX_SLICE_LEN = 1990;

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service =
        node.service_builder(service_name).template request_response<uint64_t, bb::Slice<uint64_t>>().create().value();

    auto sut_server = service.server_builder().initial_max_slice_len(INITIAL_MAX_SLICE_LEN).create().value();

    ASSERT_THAT(sut_server.initial_max_slice_len(), Eq(INITIAL_MAX_SLICE_LEN));
}

TYPED_TEST(ServiceRequestResponseTest, client_applies_unable_to_deliver_strategy) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service = node.service_builder(service_name).template request_response<uint64_t, uint64_t>().create().value();

    auto sut_client_1 =
        service.client_builder().unable_to_deliver_strategy(UnableToDeliverStrategy::Block).create().value();
    auto sut_client_2 =
        service.client_builder().unable_to_deliver_strategy(UnableToDeliverStrategy::DiscardSample).create().value();

    ASSERT_THAT(sut_client_1.unable_to_deliver_strategy(), Eq(UnableToDeliverStrategy::Block));
    ASSERT_THAT(sut_client_2.unable_to_deliver_strategy(), Eq(UnableToDeliverStrategy::DiscardSample));
}

TYPED_TEST(ServiceRequestResponseTest, client_applies_initial_max_slice_length) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();
    constexpr uint64_t INITIAL_MAX_SLICE_LEN = 2008;

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service =
        node.service_builder(service_name).template request_response<bb::Slice<uint64_t>, uint64_t>().create().value();

    auto sut_client = service.client_builder().initial_max_slice_len(INITIAL_MAX_SLICE_LEN).create().value();

    ASSERT_THAT(sut_client.initial_max_slice_len(), Eq(INITIAL_MAX_SLICE_LEN));
}

TYPED_TEST(ServiceRequestResponseTest, number_of_clients_servers_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service = node.service_builder(service_name).template request_response<uint64_t, uint64_t>().create().value();

    ASSERT_THAT(service.dynamic_config().number_of_clients(), Eq(0));
    ASSERT_THAT(service.dynamic_config().number_of_servers(), Eq(0));

    {
        auto sut_client = service.client_builder().create().value();
        ASSERT_THAT(service.dynamic_config().number_of_clients(), Eq(1));
        ASSERT_THAT(service.dynamic_config().number_of_servers(), Eq(0));

        auto sut_server = service.server_builder().create().value();
        ASSERT_THAT(service.dynamic_config().number_of_clients(), Eq(1));
        ASSERT_THAT(service.dynamic_config().number_of_servers(), Eq(1));
    }

    ASSERT_THAT(service.dynamic_config().number_of_clients(), Eq(0));
    ASSERT_THAT(service.dynamic_config().number_of_servers(), Eq(0));
}

TYPED_TEST(ServiceRequestResponseTest, create_with_attributes_sets_attributes) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    auto key = *Attribute::Key::from_utf8("nice key");
    auto value = *Attribute::Value::from_utf8("with a shiny value");
    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto attribute_specifier = AttributeSpecifier();
    attribute_specifier.define(key, value).value();
    auto service_create = node.service_builder(service_name)
                              .template request_response<uint64_t, uint64_t>()
                              .create_with_attributes(attribute_specifier)
                              .value();

    auto service_open =
        node.service_builder(service_name).template request_response<uint64_t, uint64_t>().open().value();


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

    auto key = *Attribute::Key::from_utf8("which song does hypnotoad sing?");
    auto value = *Attribute::Value::from_utf8("is it 'all my hypnoflies'?");
    auto missing_key = *Attribute::Key::from_utf8("no it's 'nala-la-la-la'!");
    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto attribute_verifier = AttributeVerifier();
    attribute_verifier.require(key, value).value();
    auto service_create = node.service_builder(service_name)
                              .template request_response<uint64_t, uint64_t>()
                              .open_or_create_with_attributes(attribute_verifier)
                              .value();

    attribute_verifier.require_key(missing_key).value();
    auto service_open_or_create = node.service_builder(service_name)
                                      .template request_response<uint64_t, uint64_t>()
                                      .open_or_create_with_attributes(attribute_verifier);

    ASSERT_THAT(service_open_or_create.has_value(), Eq(false));
    ASSERT_THAT(service_open_or_create.error(), Eq(RequestResponseOpenOrCreateError::OpenIncompatibleAttributes));

    auto service_open = node.service_builder(service_name)
                            .template request_response<uint64_t, uint64_t>()
                            .open_with_attributes(attribute_verifier);

    ASSERT_THAT(service_open.has_value(), Eq(false));
    ASSERT_THAT(service_open.error(), Eq(RequestResponseOpenError::IncompatibleAttributes));
}

TYPED_TEST(ServiceRequestResponseTest, origin_is_set_correctly) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service = node.service_builder(service_name).template request_response<uint64_t, uint64_t>().create().value();

    auto sut_client = service.client_builder().create().value();
    auto sut_server = service.server_builder().create().value();

    auto request_uninit = sut_client.loan_uninit().value();
    EXPECT_TRUE(request_uninit.header().client_port_id() == sut_client.id());

    auto pending_response = send(assume_init(std::move(request_uninit))).value();
    EXPECT_TRUE(pending_response.header().client_port_id() == sut_client.id());

    auto active_request = sut_server.receive().value();
    EXPECT_TRUE(active_request->origin() == sut_client.id());
    EXPECT_TRUE(active_request->header().client_port_id() == sut_client.id());

    auto response_uninit = active_request->loan_uninit().value();
    EXPECT_TRUE(response_uninit.header().server_port_id() == sut_server.id());
    send(assume_init(std::move(response_uninit))).value();

    auto response = pending_response.receive().value();
    ASSERT_TRUE(response.has_value());
    EXPECT_TRUE(response->origin() == sut_server.id());
    EXPECT_TRUE(response->header().server_port_id() == sut_server.id());
}

TYPED_TEST(ServiceRequestResponseTest, is_connected_works_for_active_request) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service = node.service_builder(service_name).template request_response<uint64_t, uint64_t>().create().value();

    auto sut_client = service.client_builder().create().value();
    auto sut_server = service.server_builder().create().value();

    bb::Optional<PendingResponse<SERVICE_TYPE, uint64_t, void, uint64_t, void>> pending_response;
    pending_response.emplace(sut_client.send_copy(3).value());

    auto active_request = sut_server.receive().value();
    EXPECT_TRUE(active_request->is_connected());

    pending_response.reset();
    EXPECT_FALSE(active_request->is_connected());
}

TYPED_TEST(ServiceRequestResponseTest, is_connected_works_for_pending_response) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service = node.service_builder(service_name).template request_response<uint64_t, uint64_t>().create().value();

    auto client = service.client_builder().create().value();
    auto server1 = service.server_builder().create().value();
    auto server2 = service.server_builder().create().value();

    auto pending_response = client.send_copy(3).value();
    EXPECT_TRUE(pending_response.is_connected());

    auto tmp = server1.receive().value();
    ASSERT_TRUE(tmp.has_value());
    bb::Optional<ActiveRequest<SERVICE_TYPE, uint64_t, void, uint64_t, void>> active_request_1;
    active_request_1.emplace(std::move(tmp.value()));
    tmp = server2.receive().value();
    ASSERT_TRUE(tmp.has_value());
    bb::Optional<ActiveRequest<SERVICE_TYPE, uint64_t, void, uint64_t, void>> active_request_2;
    active_request_2.emplace(std::move(tmp.value()));
    EXPECT_TRUE(pending_response.is_connected());

    active_request_1.reset();
    EXPECT_TRUE(pending_response.is_connected());

    active_request_2.reset();
    EXPECT_FALSE(pending_response.is_connected());
}

TYPED_TEST(ServiceRequestResponseTest, client_reallocates_memory_when_allocation_strategy_is_set) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    constexpr uint64_t INITIAL_SIZE = 128;

    const auto service_name = iox2_testing::generate_service_name();
    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service =
        node.service_builder(service_name).template request_response<bb::Slice<uint64_t>, uint64_t>().create().value();

    auto client = service.client_builder()
                      .initial_max_slice_len(INITIAL_SIZE)
                      .allocation_strategy(AllocationStrategy::BestFit)
                      .create()
                      .value();

    {
        auto request = client.loan_slice(INITIAL_SIZE);
        ASSERT_TRUE(request.has_value());
    }

    {
        auto request = client.loan_slice(INITIAL_SIZE * INITIAL_SIZE);
        ASSERT_TRUE(request.has_value());
    }

    {
        auto request = client.loan_slice(INITIAL_SIZE * INITIAL_SIZE * INITIAL_SIZE);
        ASSERT_TRUE(request.has_value());
    }
}

TYPED_TEST(ServiceRequestResponseTest, client_does_not_reallocate_when_allocation_strategy_is_static) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    constexpr uint64_t INITIAL_SIZE = 128;

    const auto service_name = iox2_testing::generate_service_name();
    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service =
        node.service_builder(service_name).template request_response<bb::Slice<uint64_t>, uint64_t>().create().value();

    auto client = service.client_builder()
                      .initial_max_slice_len(INITIAL_SIZE)
                      .allocation_strategy(AllocationStrategy::Static)
                      .create()
                      .value();

    auto request_1 = client.loan_slice(INITIAL_SIZE);
    ASSERT_TRUE(request_1.has_value());

    auto request_2 = client.loan_slice(INITIAL_SIZE * INITIAL_SIZE);
    ASSERT_FALSE(request_2.has_value());
    ASSERT_THAT(request_2.error(), Eq(LoanError::ExceedsMaxLoanSize));

    auto request_3 = client.loan_slice(INITIAL_SIZE * INITIAL_SIZE * INITIAL_SIZE);
    ASSERT_FALSE(request_3.has_value());
    ASSERT_THAT(request_3.error(), Eq(LoanError::ExceedsMaxLoanSize));
}

TYPED_TEST(ServiceRequestResponseTest, server_reallocates_memory_when_allocation_strategy_is_set) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    constexpr uint64_t INITIAL_SIZE = 128;

    const auto service_name = iox2_testing::generate_service_name();
    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service = node.service_builder(service_name)
                       .template request_response<uint64_t, bb::Slice<uint64_t>>()
                       .max_clients(1)
                       .max_servers(1)
                       .create()
                       .value();

    auto client = service.client_builder().create().value();
    auto server = service.server_builder()
                      .initial_max_slice_len(INITIAL_SIZE)
                      .allocation_strategy(AllocationStrategy::BestFit)
                      .create()
                      .value();

    auto pending_response = client.send_copy(0).value();
    auto active_request = server.receive().value();
    ASSERT_TRUE(active_request.has_value());

    {
        auto response = active_request->loan_slice(INITIAL_SIZE);
        ASSERT_TRUE(response.has_value());
    }

    {
        auto response = active_request->loan_slice(INITIAL_SIZE * INITIAL_SIZE);
        ASSERT_TRUE(response.has_value());
    }

    {
        auto response = active_request->loan_slice(INITIAL_SIZE * INITIAL_SIZE * INITIAL_SIZE);
        ASSERT_TRUE(response.has_value());
    }
}

TYPED_TEST(ServiceRequestResponseTest, server_does_not_reallocate_when_allocation_strategy_is_static) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    constexpr uint64_t INITIAL_SIZE = 128;

    const auto service_name = iox2_testing::generate_service_name();
    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service =
        node.service_builder(service_name).template request_response<uint64_t, bb::Slice<uint64_t>>().create().value();

    auto client = service.client_builder().create().value();
    auto server = service.server_builder()
                      .initial_max_slice_len(INITIAL_SIZE)
                      .allocation_strategy(AllocationStrategy::Static)
                      .create()
                      .value();

    auto pending_response = client.send_copy(0).value();
    auto active_request = server.receive().value();
    ASSERT_TRUE(active_request.has_value());

    auto response_1 = active_request->loan_slice(INITIAL_SIZE);
    ASSERT_TRUE(response_1.has_value());

    auto response_2 = active_request->loan_slice(INITIAL_SIZE * INITIAL_SIZE);
    ASSERT_FALSE(response_2.has_value());
    ASSERT_THAT(response_2.error(), Eq(LoanError::ExceedsMaxLoanSize));

    auto response_3 = active_request->loan_slice(INITIAL_SIZE * INITIAL_SIZE * INITIAL_SIZE);
    ASSERT_FALSE(response_3.has_value());
    ASSERT_THAT(response_3.error(), Eq(LoanError::ExceedsMaxLoanSize));
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

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto sut_create = node.service_builder(service_name).template request_response<Payload, Payload>().create().value();
    auto sut_open = node.service_builder(service_name).template request_response<Payload, Payload>().open();
    ASSERT_TRUE(sut_open.has_value());
}

TYPED_TEST(ServiceRequestResponseTest,
           opening_existing_service_with_different_payload_but_same_set_payload_type_name_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto sut_create = node.service_builder(service_name).template request_response<Payload, Payload>().create().value();
    auto sut_open = node.service_builder(service_name)
                        .template request_response<DifferentPayloadWithSameTypeName, DifferentPayloadWithSameTypeName>()
                        .open();
    ASSERT_TRUE(sut_open.has_value());
}

TYPED_TEST(ServiceRequestResponseTest, opening_existing_service_without_payload_type_name_fails) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    const auto service_name_req = iox2_testing::generate_service_name();
    const auto service_name_res = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto sut_create_req =
        node.service_builder(service_name_req).template request_response<Payload, uint64_t>().create().value();
    auto sut_create_res =
        node.service_builder(service_name_res).template request_response<uint64_t, Payload>().create().value();

    struct Payload {
        int32_t x;
        double y;
    };

    auto sut_open_req = node.service_builder(service_name_req).template request_response<Payload, uint64_t>().open();
    ASSERT_FALSE(sut_open_req.has_value());
    EXPECT_EQ(sut_open_req.error(), RequestResponseOpenError::IncompatibleRequestType);
    auto sut_open_res = node.service_builder(service_name_res).template request_response<uint64_t, Payload>().open();
    ASSERT_FALSE(sut_open_res.has_value());
    EXPECT_EQ(sut_open_res.error(), RequestResponseOpenError::IncompatibleResponseType);
}

TYPED_TEST(ServiceRequestResponseTest,
           opening_existing_service_with_same_payload_but_different_payload_type_name_fails) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto sut_create = node.service_builder(service_name).template request_response<Payload, Payload>().create().value();

    auto sut_open_req = node.service_builder(service_name).template request_response<other::Payload, Payload>().open();
    ASSERT_FALSE(sut_open_req.has_value());
    EXPECT_EQ(sut_open_req.error(), RequestResponseOpenError::IncompatibleRequestType);
    auto sut_open_res = node.service_builder(service_name).template request_response<Payload, other::Payload>().open();
    ASSERT_FALSE(sut_open_res.has_value());
    EXPECT_EQ(sut_open_res.error(), RequestResponseOpenError::IncompatibleResponseType);
}

TYPED_TEST(ServiceRequestResponseTest, opening_existing_service_with_same_payload_type_name_but_different_size_fails) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto sut_create = node.service_builder(service_name).template request_response<Payload, Payload>().create().value();

    auto sut_open_req = node.service_builder(service_name)
                            .template request_response<PayloadWithSameTypeNameButDifferentSize, Payload>()
                            .open();
    ASSERT_FALSE(sut_open_req.has_value());
    EXPECT_EQ(sut_open_req.error(), RequestResponseOpenError::IncompatibleRequestType);
    auto sut_open_res = node.service_builder(service_name)
                            .template request_response<Payload, PayloadWithSameTypeNameButDifferentSize>()
                            .open();
    ASSERT_FALSE(sut_open_res.has_value());
    EXPECT_EQ(sut_open_res.error(), RequestResponseOpenError::IncompatibleResponseType);
}

TYPED_TEST(ServiceRequestResponseTest,
           opening_existing_service_with_same_payload_type_name_but_different_alignment_fails) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto sut_create = node.service_builder(service_name).template request_response<Payload, Payload>().create().value();

    auto sut_open_req = node.service_builder(service_name)
                            .template request_response<PayloadWithSameTypeNameButDifferentAlignment, Payload>()
                            .open();
    ASSERT_FALSE(sut_open_req.has_value());
    EXPECT_EQ(sut_open_req.error(), RequestResponseOpenError::IncompatibleRequestType);
    auto sut_open_res = node.service_builder(service_name)
                            .template request_response<Payload, PayloadWithSameTypeNameButDifferentAlignment>()
                            .open();
    ASSERT_FALSE(sut_open_res.has_value());
    EXPECT_EQ(sut_open_res.error(), RequestResponseOpenError::IncompatibleResponseType);
}

TYPED_TEST(ServiceRequestResponseTest, opening_existing_service_with_set_user_header_type_name_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto sut_create = node.service_builder(service_name)
                          .template request_response<uint8_t, uint8_t>()
                          .template request_user_header<CustomHeader>()
                          .template response_user_header<CustomHeader>()
                          .create()
                          .value();
    auto sut_open = node.service_builder(service_name)
                        .template request_response<uint8_t, uint8_t>()
                        .template request_user_header<CustomHeader>()
                        .template response_user_header<CustomHeader>()
                        .open();
    ASSERT_TRUE(sut_open.has_value());
}

TYPED_TEST(ServiceRequestResponseTest,
           opening_existing_service_with_different_header_but_same_set_user_header_type_name_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto sut_create = node.service_builder(service_name)
                          .template request_response<uint8_t, uint8_t>()
                          .template request_user_header<CustomHeader>()
                          .template response_user_header<CustomHeader>()
                          .create()
                          .value();
    auto sut_open = node.service_builder(service_name)
                        .template request_response<uint8_t, uint8_t>()
                        .template request_user_header<DifferentCustomHeaderWithSameTypeName>()
                        .template response_user_header<DifferentCustomHeaderWithSameTypeName>()
                        .open();
    ASSERT_TRUE(sut_open.has_value());
}

TYPED_TEST(ServiceRequestResponseTest, opening_existing_service_without_user_header_type_name_fails) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    const auto service_name_req = iox2_testing::generate_service_name();
    const auto service_name_res = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto sut_create_req = node.service_builder(service_name_req)
                              .template request_response<uint8_t, uint8_t>()
                              .template request_user_header<CustomHeader>()
                              .create()
                              .value();
    auto sut_create_res = node.service_builder(service_name_res)
                              .template request_response<uint8_t, uint8_t>()
                              .template response_user_header<CustomHeader>()
                              .create()
                              .value();

    struct CustomHeader {
        uint64_t a;
        uint8_t b;
    };
    auto sut_open_req = node.service_builder(service_name_req)
                            .template request_response<uint8_t, uint8_t>()
                            .template request_user_header<CustomHeader>()
                            .open();
    ASSERT_FALSE(sut_open_req.has_value());
    EXPECT_EQ(sut_open_req.error(), RequestResponseOpenError::IncompatibleRequestType);
    auto sut_open_res = node.service_builder(service_name_res)
                            .template request_response<uint8_t, uint8_t>()
                            .template response_user_header<CustomHeader>()
                            .open();
    ASSERT_FALSE(sut_open_res.has_value());
    EXPECT_EQ(sut_open_res.error(), RequestResponseOpenError::IncompatibleResponseType);
}

TYPED_TEST(ServiceRequestResponseTest,
           opening_existing_service_with_same_header_but_different_user_header_type_name_fails) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto sut_create = node.service_builder(service_name)
                          .template request_response<uint8_t, uint8_t>()
                          .template request_user_header<CustomHeader>()
                          .template response_user_header<CustomHeader>()
                          .create()
                          .value();

    auto sut_open_req = node.service_builder(service_name)
                            .template request_response<uint8_t, uint8_t>()
                            .template request_user_header<other::CustomHeader>()
                            .template response_user_header<CustomHeader>()
                            .open();
    ASSERT_FALSE(sut_open_req.has_value());
    EXPECT_EQ(sut_open_req.error(), RequestResponseOpenError::IncompatibleRequestType);

    auto sut_open_res = node.service_builder(service_name)
                            .template request_response<uint8_t, uint8_t>()
                            .template request_user_header<CustomHeader>()
                            .template response_user_header<other::CustomHeader>()
                            .open();
    ASSERT_FALSE(sut_open_res.has_value());
    EXPECT_EQ(sut_open_res.error(), RequestResponseOpenError::IncompatibleResponseType);
}

TYPED_TEST(ServiceRequestResponseTest, opening_existing_service_with_same_header_type_but_different_size_fails) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto sut_create = node.service_builder(service_name)
                          .template request_response<uint8_t, uint8_t>()
                          .template request_user_header<CustomHeader>()
                          .template response_user_header<CustomHeader>()
                          .create()
                          .value();

    auto sut_open_req = node.service_builder(service_name)
                            .template request_response<uint8_t, uint8_t>()
                            .template request_user_header<CustomHeaderWithSameTypeNameButDifferentSize>()
                            .template response_user_header<CustomHeader>()
                            .open();
    ASSERT_FALSE(sut_open_req.has_value());
    EXPECT_EQ(sut_open_req.error(), RequestResponseOpenError::IncompatibleRequestType);

    auto sut_open_res = node.service_builder(service_name)
                            .template request_response<uint8_t, uint8_t>()
                            .template request_user_header<CustomHeader>()
                            .template response_user_header<CustomHeaderWithSameTypeNameButDifferentSize>()
                            .open();
    ASSERT_FALSE(sut_open_res.has_value());
    EXPECT_EQ(sut_open_res.error(), RequestResponseOpenError::IncompatibleResponseType);
}

TYPED_TEST(ServiceRequestResponseTest, opening_existing_service_with_same_header_type_but_different_alignment_fails) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto sut_create = node.service_builder(service_name)
                          .template request_response<uint8_t, uint8_t>()
                          .template request_user_header<CustomHeader>()
                          .template response_user_header<CustomHeader>()
                          .create()
                          .value();

    auto sut_open_req = node.service_builder(service_name)
                            .template request_response<uint8_t, uint8_t>()
                            .template request_user_header<CustomHeaderWithSameTypeNameButDifferentAlignment>()
                            .template response_user_header<CustomHeader>()
                            .open();
    ASSERT_FALSE(sut_open_req.has_value());
    EXPECT_EQ(sut_open_req.error(), RequestResponseOpenError::IncompatibleRequestType);

    auto sut_open_res = node.service_builder(service_name)
                            .template request_response<uint8_t, uint8_t>()
                            .template request_user_header<CustomHeader>()
                            .template response_user_header<CustomHeaderWithSameTypeNameButDifferentAlignment>()
                            .open();
    ASSERT_FALSE(sut_open_res.has_value());
    EXPECT_EQ(sut_open_res.error(), RequestResponseOpenError::IncompatibleResponseType);
}

TYPED_TEST(ServiceRequestResponseTest,
           payload_type_name_is_set_to_rust_equivalent_for_fixed_size_integers_floats_and_slices) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    const auto service_name = iox2_testing::generate_service_name();
    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    {
        auto service =
            node.service_builder(service_name).template request_response<uint8_t, uint8_t>().create().value();
        auto static_config = service.static_config();
        ASSERT_THAT(static_config.request_message_type_details().payload().type_name(), StrEq("u8"));
        ASSERT_THAT(static_config.response_message_type_details().payload().type_name(), StrEq("u8"));
    }
    {
        auto service =
            node.service_builder(service_name).template request_response<uint16_t, uint16_t>().create().value();
        auto static_config = service.static_config();
        ASSERT_THAT(static_config.request_message_type_details().payload().type_name(), StrEq("u16"));
        ASSERT_THAT(static_config.response_message_type_details().payload().type_name(), StrEq("u16"));
    }
    {
        auto service =
            node.service_builder(service_name).template request_response<uint32_t, uint32_t>().create().value();
        auto static_config = service.static_config();
        ASSERT_THAT(static_config.request_message_type_details().payload().type_name(), StrEq("u32"));
        ASSERT_THAT(static_config.response_message_type_details().payload().type_name(), StrEq("u32"));
    }
    {
        auto service =
            node.service_builder(service_name).template request_response<uint64_t, uint64_t>().create().value();
        auto static_config = service.static_config();
        ASSERT_THAT(static_config.request_message_type_details().payload().type_name(), StrEq("u64"));
        ASSERT_THAT(static_config.response_message_type_details().payload().type_name(), StrEq("u64"));
    }
    {
        auto service = node.service_builder(service_name).template request_response<int8_t, int8_t>().create().value();
        auto static_config = service.static_config();
        ASSERT_THAT(static_config.request_message_type_details().payload().type_name(), StrEq("i8"));
        ASSERT_THAT(static_config.response_message_type_details().payload().type_name(), StrEq("i8"));
    }
    {
        auto service =
            node.service_builder(service_name).template request_response<int16_t, int16_t>().create().value();
        auto static_config = service.static_config();
        ASSERT_THAT(static_config.request_message_type_details().payload().type_name(), StrEq("i16"));
        ASSERT_THAT(static_config.response_message_type_details().payload().type_name(), StrEq("i16"));
    }
    {
        auto service =
            node.service_builder(service_name).template request_response<int32_t, int32_t>().create().value();
        auto static_config = service.static_config();
        ASSERT_THAT(static_config.request_message_type_details().payload().type_name(), StrEq("i32"));
        ASSERT_THAT(static_config.response_message_type_details().payload().type_name(), StrEq("i32"));
    }
    {
        auto service =
            node.service_builder(service_name).template request_response<int64_t, int64_t>().create().value();
        auto static_config = service.static_config();
        ASSERT_THAT(static_config.request_message_type_details().payload().type_name(), StrEq("i64"));
        ASSERT_THAT(static_config.response_message_type_details().payload().type_name(), StrEq("i64"));
    }
    {
        auto service = node.service_builder(service_name).template request_response<float, float>().create().value();
        auto static_config = service.static_config();
        ASSERT_THAT(static_config.request_message_type_details().payload().type_name(), StrEq("f32"));
        ASSERT_THAT(static_config.response_message_type_details().payload().type_name(), StrEq("f32"));
    }
    {
        auto service = node.service_builder(service_name).template request_response<double, double>().create().value();
        auto static_config = service.static_config();
        ASSERT_THAT(static_config.request_message_type_details().payload().type_name(), StrEq("f64"));
        ASSERT_THAT(static_config.response_message_type_details().payload().type_name(), StrEq("f64"));
    }
    {
        auto service = node.service_builder(service_name)
                           .template request_response<bb::Slice<uint8_t>, bb::Slice<uint8_t>>()
                           .create()
                           .value();
        auto static_config = service.static_config();
        ASSERT_THAT(static_config.request_message_type_details().payload().type_name(), StrEq("u8"));
        ASSERT_THAT(static_config.response_message_type_details().payload().type_name(), StrEq("u8"));
    }
    {
        auto service = node.service_builder(service_name)
                           .template request_response<bb::Slice<uint16_t>, bb::Slice<uint16_t>>()
                           .create()
                           .value();
        auto static_config = service.static_config();
        ASSERT_THAT(static_config.request_message_type_details().payload().type_name(), StrEq("u16"));
        ASSERT_THAT(static_config.response_message_type_details().payload().type_name(), StrEq("u16"));
    }
    {
        auto service = node.service_builder(service_name)
                           .template request_response<bb::Slice<uint32_t>, bb::Slice<uint32_t>>()
                           .create()
                           .value();
        auto static_config = service.static_config();
        ASSERT_THAT(static_config.request_message_type_details().payload().type_name(), StrEq("u32"));
        ASSERT_THAT(static_config.response_message_type_details().payload().type_name(), StrEq("u32"));
    }
    {
        auto service = node.service_builder(service_name)
                           .template request_response<bb::Slice<uint64_t>, bb::Slice<uint64_t>>()
                           .create()
                           .value();
        auto static_config = service.static_config();
        ASSERT_THAT(static_config.request_message_type_details().payload().type_name(), StrEq("u64"));
        ASSERT_THAT(static_config.response_message_type_details().payload().type_name(), StrEq("u64"));
    }
    {
        auto service = node.service_builder(service_name)
                           .template request_response<bb::Slice<int8_t>, bb::Slice<int8_t>>()
                           .create()
                           .value();
        auto static_config = service.static_config();
        ASSERT_THAT(static_config.request_message_type_details().payload().type_name(), StrEq("i8"));
        ASSERT_THAT(static_config.response_message_type_details().payload().type_name(), StrEq("i8"));
    }
    {
        auto service = node.service_builder(service_name)
                           .template request_response<bb::Slice<int16_t>, bb::Slice<int16_t>>()
                           .create()
                           .value();
        auto static_config = service.static_config();
        ASSERT_THAT(static_config.request_message_type_details().payload().type_name(), StrEq("i16"));
        ASSERT_THAT(static_config.response_message_type_details().payload().type_name(), StrEq("i16"));
    }
    {
        auto service = node.service_builder(service_name)
                           .template request_response<bb::Slice<int32_t>, bb::Slice<int32_t>>()
                           .create()
                           .value();
        auto static_config = service.static_config();
        ASSERT_THAT(static_config.request_message_type_details().payload().type_name(), StrEq("i32"));
        ASSERT_THAT(static_config.response_message_type_details().payload().type_name(), StrEq("i32"));
    }
    {
        auto service = node.service_builder(service_name)
                           .template request_response<bb::Slice<int64_t>, bb::Slice<int64_t>>()
                           .create()
                           .value();
        auto static_config = service.static_config();
        ASSERT_THAT(static_config.request_message_type_details().payload().type_name(), StrEq("i64"));
        ASSERT_THAT(static_config.response_message_type_details().payload().type_name(), StrEq("i64"));
    }
    {
        auto service = node.service_builder(service_name)
                           .template request_response<bb::Slice<float>, bb::Slice<float>>()
                           .create()
                           .value();
        auto static_config = service.static_config();
        ASSERT_THAT(static_config.request_message_type_details().payload().type_name(), StrEq("f32"));
        ASSERT_THAT(static_config.response_message_type_details().payload().type_name(), StrEq("f32"));
    }
    {
        auto service = node.service_builder(service_name)
                           .template request_response<bb::Slice<double>, bb::Slice<double>>()
                           .create()
                           .value();
        auto static_config = service.static_config();
        ASSERT_THAT(static_config.request_message_type_details().payload().type_name(), StrEq("f64"));
        ASSERT_THAT(static_config.response_message_type_details().payload().type_name(), StrEq("f64"));
    }
}

TYPED_TEST(ServiceRequestResponseTest, payload_type_name_is_set_to_inner_type_name_if_provided) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    const auto service_name = iox2_testing::generate_service_name();
    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service = node.service_builder(service_name)
                       .template request_response<bb::Slice<Payload>, bb::Slice<Payload>>()
                       .create()
                       .value();

    auto static_config = service.static_config();
    ASSERT_THAT(static_config.request_message_type_details().payload().type_name(), StrEq("Payload"));
    ASSERT_THAT(static_config.response_message_type_details().payload().type_name(), StrEq("Payload"));
}
// END tests for customizable payload and user header type name

TYPED_TEST(ServiceRequestResponseTest, service_id_is_unique_per_service) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    const auto service_name_1 = iox2_testing::generate_service_name();
    const auto service_name_2 = iox2_testing::generate_service_name();
    auto node = NodeBuilder().create<SERVICE_TYPE>().value();

    auto service_1_create =
        node.service_builder(service_name_1).template request_response<uint64_t, uint64_t>().create().value();
    auto service_1_open =
        node.service_builder(service_name_1).template request_response<uint64_t, uint64_t>().open().value();
    auto service_2 =
        node.service_builder(service_name_2).template request_response<uint64_t, uint64_t>().create().value();

    ASSERT_THAT(service_1_create.service_id().c_str(), StrEq(service_1_open.service_id().c_str()));
    ASSERT_THAT(service_1_create.service_id().c_str(), Not(StrEq(service_2.service_id().c_str())));
}

TYPED_TEST(ServiceRequestResponseTest, listing_all_clients_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    constexpr uint64_t NUMBER_OF_CLIENTS = 16;

    const auto service_name = iox2_testing::generate_service_name();
    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto sut = node.service_builder(service_name)
                   .template request_response<uint64_t, uint64_t>()
                   .max_clients(NUMBER_OF_CLIENTS)
                   .create()
                   .value();

    std::vector<iox2::Client<SERVICE_TYPE, uint64_t, void, uint64_t, void>> clients;
    clients.reserve(NUMBER_OF_CLIENTS);
    for (uint64_t idx = 0; idx < NUMBER_OF_CLIENTS; ++idx) {
        clients.push_back(sut.client_builder().create().value());
    }

    std::vector<UniqueClientId> client_ids;
    client_ids.reserve(NUMBER_OF_CLIENTS);
    sut.dynamic_config().list_clients([&](auto client_details_view) -> auto {
        client_ids.push_back(client_details_view.client_id());
        return CallbackProgression::Continue;
    });

    ASSERT_THAT(client_ids.size(), Eq(NUMBER_OF_CLIENTS));
    for (auto& client : clients) {
        auto iter = std::find(client_ids.begin(), client_ids.end(), client.id());
        ASSERT_THAT(iter, Ne(client_ids.end()));
    }
}

TYPED_TEST(ServiceRequestResponseTest, listing_all_clients_stops_on_request) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    constexpr uint64_t NUMBER_OF_CLIENTS = 13;

    const auto service_name = iox2_testing::generate_service_name();
    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto sut = node.service_builder(service_name)
                   .template request_response<uint64_t, uint64_t>()
                   .max_clients(NUMBER_OF_CLIENTS)
                   .create()
                   .value();

    std::vector<iox2::Client<SERVICE_TYPE, uint64_t, void, uint64_t, void>> clients;
    clients.reserve(NUMBER_OF_CLIENTS);
    for (uint64_t idx = 0; idx < NUMBER_OF_CLIENTS; ++idx) {
        clients.push_back(sut.client_builder().create().value());
    }

    auto counter = 0;
    sut.dynamic_config().list_clients([&](auto) -> auto {
        counter++;
        return CallbackProgression::Stop;
    });

    ASSERT_THAT(counter, Eq(1));
}

TYPED_TEST(ServiceRequestResponseTest, client_details_are_correct) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    constexpr uint64_t MAX_SLICE_LEN = 9;

    const auto service_name = iox2_testing::generate_service_name();
    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto sut =
        node.service_builder(service_name).template request_response<bb::Slice<uint64_t>, uint64_t>().create().value();

    iox2::Client<SERVICE_TYPE, bb::Slice<uint64_t>, void, uint64_t, void> client =
        sut.client_builder().initial_max_slice_len(MAX_SLICE_LEN).create().value();

    auto counter = 0;
    sut.dynamic_config().list_clients([&](auto client_details_view) -> auto {
        counter++;
        EXPECT_TRUE(client_details_view.client_id() == client.id());
        EXPECT_TRUE(client_details_view.node_id() == node.id());
        EXPECT_TRUE(client_details_view.max_slice_len() == MAX_SLICE_LEN);
        return CallbackProgression::Stop;
    });

    ASSERT_THAT(counter, Eq(1));
}

TYPED_TEST(ServiceRequestResponseTest, listing_all_servers_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    constexpr uint64_t NUMBER_OF_SERVERS = 16;

    const auto service_name = iox2_testing::generate_service_name();
    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto sut = node.service_builder(service_name)
                   .template request_response<uint64_t, uint64_t>()
                   .max_servers(NUMBER_OF_SERVERS)
                   .create()
                   .value();

    std::vector<iox2::Server<SERVICE_TYPE, uint64_t, void, uint64_t, void>> servers;
    servers.reserve(NUMBER_OF_SERVERS);
    for (uint64_t idx = 0; idx < NUMBER_OF_SERVERS; ++idx) {
        servers.push_back(sut.server_builder().create().value());
    }

    std::vector<UniqueServerId> server_ids;
    server_ids.reserve(NUMBER_OF_SERVERS);
    sut.dynamic_config().list_servers([&](auto server_details_view) -> auto {
        server_ids.push_back(server_details_view.server_id());
        return CallbackProgression::Continue;
    });

    ASSERT_THAT(server_ids.size(), Eq(NUMBER_OF_SERVERS));
    for (auto& server : servers) {
        auto iter = std::find(server_ids.begin(), server_ids.end(), server.id());
        ASSERT_THAT(iter, Ne(server_ids.end()));
    }
}

TYPED_TEST(ServiceRequestResponseTest, listing_all_servers_stops_on_request) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    constexpr uint64_t NUMBER_OF_SERVERS = 13;

    const auto service_name = iox2_testing::generate_service_name();
    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto sut = node.service_builder(service_name)
                   .template request_response<uint64_t, uint64_t>()
                   .max_servers(NUMBER_OF_SERVERS)
                   .create()
                   .value();

    std::vector<iox2::Server<SERVICE_TYPE, uint64_t, void, uint64_t, void>> servers;
    servers.reserve(NUMBER_OF_SERVERS);
    for (uint64_t idx = 0; idx < NUMBER_OF_SERVERS; ++idx) {
        servers.push_back(sut.server_builder().create().value());
    }

    auto counter = 0;
    sut.dynamic_config().list_servers([&](auto) -> auto {
        counter++;
        return CallbackProgression::Stop;
    });

    ASSERT_THAT(counter, Eq(1));
}

TYPED_TEST(ServiceRequestResponseTest, server_details_are_correct) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    constexpr uint64_t MAX_SLICE_LEN = 9;

    const auto service_name = iox2_testing::generate_service_name();
    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto sut =
        node.service_builder(service_name).template request_response<uint64_t, bb::Slice<uint64_t>>().create().value();

    iox2::Server<SERVICE_TYPE, uint64_t, void, bb::Slice<uint64_t>, void> server =
        sut.server_builder().initial_max_slice_len(MAX_SLICE_LEN).create().value();

    auto counter = 0;
    sut.dynamic_config().list_servers([&](auto server_details_view) -> auto {
        counter++;
        EXPECT_TRUE(server_details_view.server_id() == server.id());
        EXPECT_TRUE(server_details_view.node_id() == node.id());
        EXPECT_TRUE(server_details_view.max_slice_len() == MAX_SLICE_LEN);
        return CallbackProgression::Stop;
    });

    ASSERT_THAT(counter, Eq(1));
}

TYPED_TEST(ServiceRequestResponseTest, only_max_clients_can_be_created) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service = node.service_builder(service_name)
                       .template request_response<uint64_t, uint64_t>()
                       .max_clients(1)
                       .create()
                       .value();
    auto client =
        bb::Optional<Client<SERVICE_TYPE, uint64_t, void, uint64_t, void>>(service.client_builder().create().value());

    auto failing_sut = service.client_builder().create();
    ASSERT_FALSE(failing_sut.has_value());

    client.reset();

    auto sut = service.client_builder().create();
    ASSERT_TRUE(sut.has_value());
}

TYPED_TEST(ServiceRequestResponseTest, only_max_servers_can_be_created) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service = node.service_builder(service_name)
                       .template request_response<uint64_t, uint64_t>()
                       .max_servers(1)
                       .create()
                       .value();
    auto server =
        bb::Optional<Server<SERVICE_TYPE, uint64_t, void, uint64_t, void>>(service.server_builder().create().value());

    auto failing_sut = service.server_builder().create();
    ASSERT_FALSE(failing_sut.has_value());

    server.reset();

    auto sut = service.server_builder().create();
    ASSERT_TRUE(sut.has_value());
}

TYPED_TEST(ServiceRequestResponseTest, client_can_request_graceful_disconnect) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service = node.service_builder(service_name).template request_response<uint64_t, uint64_t>().create().value();

    auto sut_client = service.client_builder().create().value();
    auto sut_server = service.server_builder().create().value();

    bb::Optional<PendingResponse<SERVICE_TYPE, uint64_t, void, uint64_t, void>> pending_response;
    pending_response.emplace(sut_client.send_copy(0).value());
    auto active_request = sut_server.receive().value().value();

    ASSERT_TRUE(pending_response->is_connected());
    ASSERT_TRUE(active_request.is_connected());
    ASSERT_FALSE(active_request.has_disconnect_hint());

    pending_response->set_disconnect_hint();

    ASSERT_TRUE(pending_response->is_connected());
    ASSERT_TRUE(active_request.is_connected());
    ASSERT_TRUE(active_request.has_disconnect_hint());

    pending_response.reset();

    ASSERT_FALSE(active_request.is_connected());
    ASSERT_FALSE(active_request.has_disconnect_hint());
}

} // namespace
