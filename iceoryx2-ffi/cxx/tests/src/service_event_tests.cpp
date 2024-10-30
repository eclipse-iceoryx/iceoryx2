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

#include <cstdlib>

#include "iox2/node.hpp"
#include "iox2/node_name.hpp"
#include "iox2/service.hpp"

#include "test.hpp"

namespace {
using namespace iox2;

constexpr iox::units::Duration TIMEOUT = iox::units::Duration::fromMilliseconds(50);

template <typename T>
struct ServiceEventTest : public ::testing::Test {
    ServiceEventTest()
        : service_name { iox2_testing::generate_service_name() }
        , node { NodeBuilder().create<T::TYPE>().expect("") }
        , service { node.service_builder(service_name).event().create().expect("") }
        , notifier { service.notifier_builder().create().expect("") }
        , listener { service.listener_builder().create().expect("") }
        , event_id_1 { EventId(event_id_counter.fetch_add(1)) }
        , event_id_2 { EventId(event_id_counter.fetch_add(1)) } {
    }

    static std::atomic<size_t> event_id_counter;
    static constexpr ServiceType TYPE = T::TYPE;
    //NOLINTBEGIN(misc-non-private-member-variables-in-classes), required for tests
    ServiceName service_name;
    Node<T::TYPE> node;
    PortFactoryEvent<T::TYPE> service;
    Notifier<T::TYPE> notifier;
    Listener<T::TYPE> listener;
    EventId event_id_1;
    EventId event_id_2;
    //NOLINTEND(misc-non-private-member-variables-in-classes)
};

template <typename T>
std::atomic<size_t> ServiceEventTest<T>::event_id_counter { 0 };

TYPED_TEST_SUITE(ServiceEventTest, iox2_testing::ServiceTypes);

TYPED_TEST(ServiceEventTest, created_service_does_exist) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    ASSERT_FALSE(
        Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::Event).expect(""));

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");

    {
        auto sut = node.service_builder(service_name).event().create().expect("");

        ASSERT_TRUE(Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::Event)
                        .expect(""));
    }

    ASSERT_FALSE(
        Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::Event).expect(""));
}

TYPED_TEST(ServiceEventTest, creating_existing_service_fails) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut = node.service_builder(service_name).event().create().expect("");

    auto sut_2 = node.service_builder(service_name).event().create();
    ASSERT_TRUE(sut_2.has_error());
    ASSERT_THAT(sut_2.error(), Eq(EventCreateError::AlreadyExists));
}

TYPED_TEST(ServiceEventTest, service_settings_are_applied) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    constexpr uint64_t NUMBER_OF_NOTIFIERS = 5;
    constexpr uint64_t NUMBER_OF_LISTENERS = 7;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut = node.service_builder(service_name)
                   .event()
                   .max_notifiers(NUMBER_OF_NOTIFIERS)
                   .max_listeners(NUMBER_OF_LISTENERS)
                   .create()
                   .expect("");

    auto static_config = sut.static_config();

    ASSERT_THAT(static_config.max_notifiers(), Eq(NUMBER_OF_NOTIFIERS));
    ASSERT_THAT(static_config.max_listeners(), Eq(NUMBER_OF_LISTENERS));
}

TYPED_TEST(ServiceEventTest, open_fails_with_incompatible_max_notifiers_requirements) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    constexpr uint64_t NUMBER_OF_NOTIFIERS = 5;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut = node.service_builder(service_name).event().max_notifiers(NUMBER_OF_NOTIFIERS).create().expect("");
    auto sut_fail = node.service_builder(service_name).event().max_notifiers(NUMBER_OF_NOTIFIERS + 1).open();

    ASSERT_TRUE(sut_fail.has_error());
    ASSERT_THAT(sut_fail.error(), Eq(EventOpenError::DoesNotSupportRequestedAmountOfNotifiers));
}

TYPED_TEST(ServiceEventTest, open_fails_with_incompatible_max_listeners_requirements) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    constexpr uint64_t NUMBER_OF_LISTENERS = 7;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut = node.service_builder(service_name).event().max_listeners(NUMBER_OF_LISTENERS).create().expect("");
    auto sut_fail = node.service_builder(service_name).event().max_listeners(NUMBER_OF_LISTENERS + 1).open();

    ASSERT_TRUE(sut_fail.has_error());
    ASSERT_THAT(sut_fail.error(), Eq(EventOpenError::DoesNotSupportRequestedAmountOfListeners));
}

TYPED_TEST(ServiceEventTest, open_or_create_service_does_exist) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    ASSERT_FALSE(
        Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::Event).expect(""));

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");

    {
        auto sut = iox::optional<PortFactoryEvent<SERVICE_TYPE>>(
            node.service_builder(service_name).event().open_or_create().expect(""));

        ASSERT_TRUE(Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::Event)
                        .expect(""));

        auto sut_2 = iox::optional<PortFactoryEvent<SERVICE_TYPE>>(
            node.service_builder(service_name).event().open_or_create().expect(""));

        ASSERT_TRUE(Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::Event)
                        .expect(""));

        sut.reset();

        ASSERT_TRUE(Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::Event)
                        .expect(""));

        sut_2.reset();
    }

    ASSERT_FALSE(
        Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::Event).expect(""));
}

TYPED_TEST(ServiceEventTest, opening_non_existing_service_fails) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut = node.service_builder(service_name).event().open();
    ASSERT_TRUE(sut.has_error());
    ASSERT_THAT(sut.error(), Eq(EventOpenError::DoesNotExist));
}

TYPED_TEST(ServiceEventTest, opening_existing_service_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut_create = node.service_builder(service_name).event().create();
    auto sut = node.service_builder(service_name).event().open();
    ASSERT_TRUE(sut.has_value());
}

TYPED_TEST(ServiceEventTest, service_name_is_set) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut = node.service_builder(service_name).event().create().expect("");

    auto sut_service_name = sut.name();
    ASSERT_THAT(service_name.to_string(), Eq(sut_service_name.to_string()));
}

TYPED_TEST(ServiceEventTest, notification_is_received_with_try_wait_one) {
    this->notifier.notify().expect("");

    auto result = this->listener.try_wait_one().expect("");
    ASSERT_TRUE(result.has_value());
    ASSERT_THAT(result.value().as_value(), Eq(EventId(0).as_value()));
}

TYPED_TEST(ServiceEventTest, notification_with_custom_event_id_is_received_with_try_wait_one) {
    this->notifier.notify_with_custom_event_id(this->event_id_1).expect("");

    auto result = this->listener.try_wait_one().expect("");
    ASSERT_TRUE(result.has_value());
    ASSERT_THAT(result.value().as_value(), Eq(this->event_id_1.as_value()));
}

TYPED_TEST(ServiceEventTest, notification_is_received_with_timed_wait_one) {
    this->notifier.notify_with_custom_event_id(this->event_id_1).expect("");

    auto result = this->listener.timed_wait_one(TIMEOUT).expect("");
    ASSERT_TRUE(result.has_value());
    ASSERT_THAT(result.value().as_value(), Eq(this->event_id_1.as_value()));
}

TYPED_TEST(ServiceEventTest, notification_is_received_with_blocking_wait_one) {
    this->notifier.notify_with_custom_event_id(this->event_id_1).expect("");

    auto result = this->listener.timed_wait_one(TIMEOUT).expect("");
    ASSERT_TRUE(result.has_value());
    ASSERT_THAT(result.value().as_value(), Eq(this->event_id_1.as_value()));
}

TYPED_TEST(ServiceEventTest, notification_is_received_with_try_wait_all) {
    this->notifier.notify_with_custom_event_id(this->event_id_1).expect("");
    this->notifier.notify_with_custom_event_id(this->event_id_2).expect("");

    std::set<size_t> received_ids;
    this->listener.try_wait_all([&](auto event_id) { ASSERT_TRUE(received_ids.emplace(event_id.as_value()).second); })
        .expect("");
    ASSERT_THAT(received_ids.size(), Eq(2));
}

TYPED_TEST(ServiceEventTest, notification_is_received_with_timed_wait_all) {
    this->notifier.notify_with_custom_event_id(this->event_id_1).expect("");
    this->notifier.notify_with_custom_event_id(this->event_id_2).expect("");

    std::set<size_t> received_ids;
    this->listener
        .timed_wait_all([&](auto event_id) { ASSERT_TRUE(received_ids.emplace(event_id.as_value()).second); }, TIMEOUT)
        .expect("");
    ASSERT_THAT(received_ids.size(), Eq(2));
}

TYPED_TEST(ServiceEventTest, notification_is_received_with_blocking_wait_all) {
    this->notifier.notify_with_custom_event_id(this->event_id_1).expect("");
    this->notifier.notify_with_custom_event_id(this->event_id_2).expect("");

    std::set<size_t> received_ids;
    this->listener
        .blocking_wait_all([&](auto event_id) { ASSERT_TRUE(received_ids.emplace(event_id.as_value()).second); })
        .expect("");
    ASSERT_THAT(received_ids.size(), Eq(2));
}

TYPED_TEST(ServiceEventTest, timed_wait_one_does_not_deadlock) {
    auto result = this->listener.timed_wait_one(TIMEOUT).expect("");
    ASSERT_FALSE(result.has_value());
}

TYPED_TEST(ServiceEventTest, timed_wait_all_does_not_deadlock) {
    this->listener.timed_wait_all([](auto) {}, TIMEOUT).expect("");
}

TYPED_TEST(ServiceEventTest, service_can_be_opened_when_there_is_a_notifier) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    const auto event_id = EventId(54);
    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut =
        iox::optional<PortFactoryEvent<SERVICE_TYPE>>(node.service_builder(service_name).event().create().expect(""));
    auto listener = iox::optional<Listener<SERVICE_TYPE>>(sut->listener_builder().create().expect(""));
    auto notifier = iox::optional<Notifier<SERVICE_TYPE>>(sut->notifier_builder().create().expect(""));

    sut.reset();
    {
        auto temp_sut = node.service_builder(service_name).event().open();
        ASSERT_THAT(temp_sut.has_value(), Eq(true));
    }
    {
        auto temp_sut = node.service_builder(service_name).event().create();
        ASSERT_THAT(temp_sut.error(), Eq(EventCreateError::AlreadyExists));
    }
    listener.reset();

    sut = iox::optional<PortFactoryEvent<SERVICE_TYPE>>(node.service_builder(service_name).event().open().expect(""));
    listener = iox::optional<Listener<SERVICE_TYPE>>(sut->listener_builder().create().expect(""));
    notifier->notify_with_custom_event_id(event_id).expect("");
    auto notification = listener->try_wait_one().expect("");
    ASSERT_THAT(notification->as_value(), Eq(event_id.as_value()));

    listener.reset();
    sut.reset();
    notifier.reset();

    {
        auto temp_sut = node.service_builder(service_name).event().open();
        ASSERT_THAT(temp_sut.error(), Eq(EventOpenError::DoesNotExist));
    }
    {
        auto temp_sut = node.service_builder(service_name).event().create();
        ASSERT_THAT(temp_sut.has_value(), Eq(true));
    }
}

TYPED_TEST(ServiceEventTest, service_can_be_opened_when_there_is_a_listener) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    const auto event_id = EventId(24);
    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut =
        iox::optional<PortFactoryEvent<SERVICE_TYPE>>(node.service_builder(service_name).event().create().expect(""));
    auto listener = iox::optional<Listener<SERVICE_TYPE>>(sut->listener_builder().create().expect(""));
    auto notifier = iox::optional<Notifier<SERVICE_TYPE>>(sut->notifier_builder().create().expect(""));

    sut.reset();
    {
        auto temp_sut = node.service_builder(service_name).event().open();
        ASSERT_THAT(temp_sut.has_value(), Eq(true));
    }
    {
        auto temp_sut = node.service_builder(service_name).event().create();
        ASSERT_THAT(temp_sut.error(), Eq(EventCreateError::AlreadyExists));
    }
    notifier.reset();

    sut = iox::optional<PortFactoryEvent<SERVICE_TYPE>>(node.service_builder(service_name).event().open().expect(""));
    notifier = iox::optional<Notifier<SERVICE_TYPE>>(sut->notifier_builder().create().expect(""));
    notifier->notify_with_custom_event_id(event_id).expect("");
    auto notification = listener->try_wait_one().expect("");
    ASSERT_THAT(notification->as_value(), Eq(event_id.as_value()));

    notifier.reset();
    sut.reset();
    listener.reset();

    {
        auto temp_sut = node.service_builder(service_name).event().open();
        ASSERT_THAT(temp_sut.error(), Eq(EventOpenError::DoesNotExist));
    }
    {
        auto temp_sut = node.service_builder(service_name).event().create();
        ASSERT_THAT(temp_sut.has_value(), Eq(true));
    }
}
} // namespace
