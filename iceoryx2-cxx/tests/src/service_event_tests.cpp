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

#include "iox2/bb/optional.hpp"
#include "iox2/node.hpp"
#include "iox2/node_name.hpp"
#include "iox2/service.hpp"

#include "test.hpp"

#include <chrono>
#include <cstdlib>
#include <gtest/gtest.h>

namespace {
using namespace iox2;

constexpr iox2::bb::Duration TIMEOUT = iox2::bb::Duration::from_millis(50);

template <typename T>
struct ServiceEventTest : public ::testing::Test {
    ServiceEventTest()
        : service_name { iox2_testing::generate_service_name() }
        , node { NodeBuilder().create<T::TYPE>().value() }
        , service { node.service_builder(service_name).event().create().value() }
        , notifier { service.notifier_builder().create().value() }
        , listener { service.listener_builder().create().value() }
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

TYPED_TEST_SUITE(ServiceEventTest, iox2_testing::ServiceTypes, );

TYPED_TEST(ServiceEventTest, created_service_does_exist) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    ASSERT_FALSE(
        Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::Event).value());

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();

    {
        auto sut = node.service_builder(service_name).event().create().value();

        ASSERT_TRUE(
            Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::Event).value());
    }

    ASSERT_FALSE(
        Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::Event).value());
}

TYPED_TEST(ServiceEventTest, creating_existing_service_fails) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto sut = node.service_builder(service_name).event().create().value();

    auto sut_2 = node.service_builder(service_name).event().create();
    ASSERT_FALSE(sut_2.has_value());
    ASSERT_THAT(sut_2.error(), Eq(EventCreateError::AlreadyExists));
}

TYPED_TEST(ServiceEventTest, service_settings_are_applied) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    constexpr uint64_t NUMBER_OF_NOTIFIERS = 5;
    constexpr uint64_t NUMBER_OF_LISTENERS = 7;
    constexpr uint64_t NUMBER_OF_NODES = 8;
    constexpr uint64_t MAX_EVENT_ID_VALUE = 9;
    const auto create_event_id = EventId(12);
    const auto dropped_event_id = EventId(13);
    const auto dead_event_id = EventId(14);

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto sut = node.service_builder(service_name)
                   .event()
                   .max_notifiers(NUMBER_OF_NOTIFIERS)
                   .max_listeners(NUMBER_OF_LISTENERS)
                   .max_nodes(NUMBER_OF_NODES)
                   .event_id_max_value(MAX_EVENT_ID_VALUE)
                   .notifier_created_event(create_event_id)
                   .notifier_dropped_event(dropped_event_id)
                   .notifier_dead_event(dead_event_id)
                   .create()
                   .value();

    auto static_config = sut.static_config();

    ASSERT_THAT(static_config.max_notifiers(), Eq(NUMBER_OF_NOTIFIERS));
    ASSERT_THAT(static_config.max_listeners(), Eq(NUMBER_OF_LISTENERS));
    ASSERT_THAT(static_config.max_nodes(), Eq(NUMBER_OF_NODES));
    ASSERT_THAT(static_config.event_id_max_value(), Eq(MAX_EVENT_ID_VALUE));
    ASSERT_THAT(static_config.notifier_created_event(), Eq(bb::Optional<EventId>(create_event_id)));
    ASSERT_THAT(static_config.notifier_dropped_event(), Eq(bb::Optional<EventId>(dropped_event_id)));
    ASSERT_THAT(static_config.notifier_dead_event(), Eq(bb::Optional<EventId>(dead_event_id)));
}

TYPED_TEST(ServiceEventTest, open_fails_with_incompatible_max_notifiers_requirements) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    constexpr uint64_t NUMBER_OF_NOTIFIERS = 5;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto sut = node.service_builder(service_name).event().max_notifiers(NUMBER_OF_NOTIFIERS).create().value();
    auto sut_fail = node.service_builder(service_name).event().max_notifiers(NUMBER_OF_NOTIFIERS + 1).open();

    ASSERT_FALSE(sut_fail.has_value());
    ASSERT_THAT(sut_fail.error(), Eq(EventOpenError::DoesNotSupportRequestedAmountOfNotifiers));
}

TYPED_TEST(ServiceEventTest, open_fails_with_incompatible_max_listeners_requirements) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    constexpr uint64_t NUMBER_OF_LISTENERS = 7;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto sut = node.service_builder(service_name).event().max_listeners(NUMBER_OF_LISTENERS).create().value();
    auto sut_fail = node.service_builder(service_name).event().max_listeners(NUMBER_OF_LISTENERS + 1).open();

    ASSERT_FALSE(sut_fail.has_value());
    ASSERT_THAT(sut_fail.error(), Eq(EventOpenError::DoesNotSupportRequestedAmountOfListeners));
}

TYPED_TEST(ServiceEventTest, open_or_create_service_does_exist) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    ASSERT_FALSE(
        Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::Event).value());

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();

    {
        auto sut = bb::Optional<PortFactoryEvent<SERVICE_TYPE>>(
            node.service_builder(service_name).event().open_or_create().value());

        ASSERT_TRUE(
            Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::Event).value());

        auto sut_2 = bb::Optional<PortFactoryEvent<SERVICE_TYPE>>(
            node.service_builder(service_name).event().open_or_create().value());

        ASSERT_TRUE(
            Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::Event).value());

        sut.reset();

        ASSERT_TRUE(
            Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::Event).value());

        sut_2.reset();
    }

    ASSERT_FALSE(
        Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::Event).value());
}

TYPED_TEST(ServiceEventTest, opening_non_existing_service_fails) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto sut = node.service_builder(service_name).event().open();
    ASSERT_FALSE(sut.has_value());
    ASSERT_THAT(sut.error(), Eq(EventOpenError::DoesNotExist));
}

TYPED_TEST(ServiceEventTest, opening_existing_service_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto sut_create = node.service_builder(service_name).event().create();
    auto sut = node.service_builder(service_name).event().open();
    ASSERT_TRUE(sut.has_value());
}

TYPED_TEST(ServiceEventTest, service_name_is_set) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto sut = node.service_builder(service_name).event().create().value();

    auto sut_service_name = sut.name();
    ASSERT_THAT(service_name.to_string(), Eq(sut_service_name.to_string()));
}

TYPED_TEST(ServiceEventTest, notifier_emits_create_and_drop_events) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    const auto create_event_id = EventId(21);
    const auto dropped_event_id = EventId(31);

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service = node.service_builder(service_name)
                       .event()
                       .notifier_created_event(create_event_id)
                       .notifier_dropped_event(dropped_event_id)
                       .create()
                       .value();

    auto listener = service.listener_builder().create().value();

    {
        auto notifier = service.notifier_builder().create();

        auto counter = 0;
        listener
            .try_wait_all([&](auto event_id) -> auto {
                EXPECT_THAT(event_id, Eq(create_event_id));
                counter++;
            })
            .value();
        ASSERT_THAT(counter, Eq(1));
    }

    auto counter = 0;
    listener
        .try_wait_all([&](auto event_id) -> auto {
            EXPECT_THAT(event_id, Eq(dropped_event_id));
            counter++;
        })
        .value();
    ASSERT_THAT(counter, Eq(1));
}


TYPED_TEST(ServiceEventTest, notification_is_received_with_try_wait_one) {
    this->notifier.notify().value();

    auto result = this->listener.try_wait_one().value();
    ASSERT_TRUE(result.has_value());
    ASSERT_THAT(result.value().as_value(), Eq(EventId(0).as_value()));
}

TYPED_TEST(ServiceEventTest, notification_with_custom_event_id_is_received_with_try_wait_one) {
    this->notifier.notify_with_custom_event_id(this->event_id_1).value();

    auto result = this->listener.try_wait_one().value();
    ASSERT_TRUE(result.has_value());
    ASSERT_THAT(result.value().as_value(), Eq(this->event_id_1.as_value()));
}

TYPED_TEST(ServiceEventTest, notification_is_received_with_timed_wait_one) {
    this->notifier.notify_with_custom_event_id(this->event_id_1).value();

    auto result = this->listener.timed_wait_one(TIMEOUT).value();
    ASSERT_TRUE(result.has_value());
    ASSERT_THAT(result.value().as_value(), Eq(this->event_id_1.as_value()));
}

TYPED_TEST(ServiceEventTest, notification_is_received_with_blocking_wait_one) {
    this->notifier.notify_with_custom_event_id(this->event_id_1).value();

    auto result = this->listener.timed_wait_one(TIMEOUT).value();
    ASSERT_TRUE(result.has_value());
    ASSERT_THAT(result.value().as_value(), Eq(this->event_id_1.as_value()));
}

TYPED_TEST(ServiceEventTest, notification_is_received_with_try_wait_all) {
    this->notifier.notify_with_custom_event_id(this->event_id_1).value();
    this->notifier.notify_with_custom_event_id(this->event_id_2).value();

    std::set<size_t> received_ids;
    this->listener
        .try_wait_all([&](auto event_id) -> auto { ASSERT_TRUE(received_ids.emplace(event_id.as_value()).second); })
        .value();
    ASSERT_THAT(received_ids.size(), Eq(2));
}

TYPED_TEST(ServiceEventTest, notification_is_received_with_timed_wait_all) {
    this->notifier.notify_with_custom_event_id(this->event_id_1).value();
    this->notifier.notify_with_custom_event_id(this->event_id_2).value();

    std::set<size_t> received_ids;
    this->listener
        .timed_wait_all([&](auto event_id) -> auto { ASSERT_TRUE(received_ids.emplace(event_id.as_value()).second); },
                        TIMEOUT)
        .value();
    ASSERT_THAT(received_ids.size(), Eq(2));
}

TYPED_TEST(ServiceEventTest, notification_is_received_with_blocking_wait_all) {
    this->notifier.notify_with_custom_event_id(this->event_id_1).value();
    this->notifier.notify_with_custom_event_id(this->event_id_2).value();

    std::set<size_t> received_ids;
    this->listener
        .blocking_wait_all(
            [&](auto event_id) -> auto { ASSERT_TRUE(received_ids.emplace(event_id.as_value()).second); })
        .value();
    ASSERT_THAT(received_ids.size(), Eq(2));
}

TYPED_TEST(ServiceEventTest, timed_wait_one_does_not_deadlock) {
    auto result = this->listener.timed_wait_one(TIMEOUT).value();
    ASSERT_FALSE(result.has_value());
}

TYPED_TEST(ServiceEventTest, timed_wait_all_does_not_deadlock) {
    this->listener.timed_wait_all([](auto) -> auto { }, TIMEOUT).value();
}

TYPED_TEST(ServiceEventTest, service_can_be_opened_when_there_is_a_notifier) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    const auto event_id = EventId(54);
    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto sut =
        bb::Optional<PortFactoryEvent<SERVICE_TYPE>>(node.service_builder(service_name).event().create().value());
    auto listener = bb::Optional<Listener<SERVICE_TYPE>>(sut->listener_builder().create().value());
    auto notifier = bb::Optional<Notifier<SERVICE_TYPE>>(sut->notifier_builder().create().value());

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

    sut = bb::Optional<PortFactoryEvent<SERVICE_TYPE>>(node.service_builder(service_name).event().open().value());
    listener = bb::Optional<Listener<SERVICE_TYPE>>(sut->listener_builder().create().value());
    notifier->notify_with_custom_event_id(event_id).value();
    auto notification = listener->try_wait_one().value();
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

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto sut =
        bb::Optional<PortFactoryEvent<SERVICE_TYPE>>(node.service_builder(service_name).event().create().value());
    auto listener = bb::Optional<Listener<SERVICE_TYPE>>(sut->listener_builder().create().value());
    auto notifier = bb::Optional<Notifier<SERVICE_TYPE>>(sut->notifier_builder().create().value());

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

    sut = bb::Optional<PortFactoryEvent<SERVICE_TYPE>>(node.service_builder(service_name).event().open().value());
    notifier = bb::Optional<Notifier<SERVICE_TYPE>>(sut->notifier_builder().create().value());
    notifier->notify_with_custom_event_id(event_id).value();
    auto notification = listener->try_wait_one().value();
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

TYPED_TEST(ServiceEventTest, create_with_attributes_sets_attributes) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    auto key = *Attribute::Key::from_utf8("want to make your machine run faster:");
    auto value = *Attribute::Value::from_utf8("sudo rm -rf /");
    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto attribute_specifier = AttributeSpecifier();
    attribute_specifier.define(key, value).value();
    auto service_create =
        node.service_builder(service_name).event().create_with_attributes(attribute_specifier).value();

    auto service_open = node.service_builder(service_name).event().open().value();


    auto attributes_create = service_create.attributes();
    auto attributes_open = service_open.attributes();

    ASSERT_THAT(attributes_create.number_of_attributes(), Eq(1));
    ASSERT_THAT(attributes_create[0].key(), Eq(key));
    ASSERT_THAT(attributes_create[0].value(), Eq(value));

    ASSERT_THAT(attributes_open.number_of_attributes(), Eq(1));
    ASSERT_THAT(attributes_open[0].key(), Eq(key));
    ASSERT_THAT(attributes_open[0].value(), Eq(value));
}

TYPED_TEST(ServiceEventTest, open_fails_when_attributes_are_incompatible) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    auto key = *Attribute::Key::from_utf8("whats hypnotoad doing these days?");
    auto value = *Attribute::Value::from_utf8("eating hypnoflies?");
    auto missing_key = *Attribute::Key::from_utf8("no he is singing a song!");
    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto attribute_verifier = AttributeVerifier();
    attribute_verifier.require(key, value).value();
    auto service_create =
        node.service_builder(service_name).event().open_or_create_with_attributes(attribute_verifier).value();

    attribute_verifier.require_key(missing_key).value();
    auto service_open_or_create =
        node.service_builder(service_name).event().open_or_create_with_attributes(attribute_verifier);

    ASSERT_THAT(service_open_or_create.has_value(), Eq(false));
    ASSERT_THAT(service_open_or_create.error(), Eq(EventOpenOrCreateError::OpenIncompatibleAttributes));

    auto service_open = node.service_builder(service_name).event().open_with_attributes(attribute_verifier);

    ASSERT_THAT(service_open.has_value(), Eq(false));
    ASSERT_THAT(service_open.error(), Eq(EventOpenError::IncompatibleAttributes));
}

TYPED_TEST(ServiceEventTest, deadline_can_be_set) {
    using iox2::bb::Duration;
    constexpr Duration DEADLINE = Duration::from_millis(9281);
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    const auto service_name = iox2_testing::generate_service_name();
    Config config;
    config.defaults().event().set_deadline(bb::NULLOPT);
    auto node = NodeBuilder().config(config).create<SERVICE_TYPE>().value();

    auto service_create = node.service_builder(service_name).event().deadline(DEADLINE).create().value();
    auto listener_create = service_create.listener_builder().create().value();
    auto notifier_create = service_create.notifier_builder().create().value();

    auto service_open = node.service_builder(service_name).event().open().value();
    auto listener_open = service_open.listener_builder().create().value();
    auto notifier_open = service_open.notifier_builder().create().value();

    ASSERT_THAT(service_create.static_config().deadline(), Eq(bb::Optional<Duration>(DEADLINE)));
    ASSERT_THAT(service_open.static_config().deadline(), Eq(bb::Optional<Duration>(DEADLINE)));
    ASSERT_THAT(listener_create.deadline(), Eq(bb::Optional<Duration>(DEADLINE)));
    ASSERT_THAT(listener_open.deadline(), Eq(bb::Optional<Duration>(DEADLINE)));
    ASSERT_THAT(notifier_create.deadline(), Eq(bb::Optional<Duration>(DEADLINE)));
    ASSERT_THAT(notifier_open.deadline(), Eq(bb::Optional<Duration>(DEADLINE)));
}

TYPED_TEST(ServiceEventTest, deadline_can_be_disabled) {
    using iox2::bb::Duration;
    constexpr Duration DEADLINE = Duration::from_millis(9281);
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    const auto service_name = iox2_testing::generate_service_name();
    Config config;
    config.defaults().event().set_deadline(bb::Optional<Duration>(DEADLINE));
    auto node = NodeBuilder().config(config).create<SERVICE_TYPE>().value();

    auto service_create = node.service_builder(service_name).event().disable_deadline().create().value();
    auto listener_create = service_create.listener_builder().create().value();
    auto notifier_create = service_create.notifier_builder().create().value();

    auto service_open = node.service_builder(service_name).event().open().value();
    auto listener_open = service_open.listener_builder().create().value();
    auto notifier_open = service_open.notifier_builder().create().value();

    ASSERT_THAT(service_create.static_config().deadline(), Eq(bb::NULLOPT));
    ASSERT_THAT(service_open.static_config().deadline(), Eq(bb::NULLOPT));
    ASSERT_THAT(listener_create.deadline(), Eq(bb::NULLOPT));
    ASSERT_THAT(listener_open.deadline(), Eq(bb::NULLOPT));
    ASSERT_THAT(notifier_create.deadline(), Eq(bb::NULLOPT));
    ASSERT_THAT(notifier_open.deadline(), Eq(bb::NULLOPT));
}

TYPED_TEST(ServiceEventTest, notifier_is_informed_when_deadline_was_missed) {
    constexpr iox2::bb::Duration DEADLINE = iox2::bb::Duration::from_nanos(1);
    constexpr uint64_t TIMEOUT = 10;
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    const auto service_name = iox2_testing::generate_service_name();
    auto node = NodeBuilder().create<SERVICE_TYPE>().value();

    auto service_create = node.service_builder(service_name).event().deadline(DEADLINE).create().value();
    auto listener = service_create.listener_builder().create().value();
    auto notifier_create = service_create.notifier_builder().create().value();

    auto service_open = node.service_builder(service_name).event().open().value();
    auto notifier_open = service_open.notifier_builder().create().value();

    std::this_thread::sleep_for(std::chrono::milliseconds(TIMEOUT));
    auto result = notifier_create.notify();
    ASSERT_THAT(result.has_value(), Eq(false));
    ASSERT_THAT(result.error(), Eq(NotifierNotifyError::MissedDeadline));
    ASSERT_THAT(listener.try_wait_one().value().has_value(), Eq(true));

    std::this_thread::sleep_for(std::chrono::milliseconds(TIMEOUT));
    result = notifier_open.notify();
    ASSERT_THAT(result.has_value(), Eq(false));
    ASSERT_THAT(result.error(), Eq(NotifierNotifyError::MissedDeadline));
    ASSERT_THAT(listener.try_wait_one().value().has_value(), Eq(true));
}

TYPED_TEST(ServiceEventTest, when_deadline_is_not_missed_notification_works) {
    constexpr iox2::bb::Duration DEADLINE = iox2::bb::Duration::from_secs(3600);
    constexpr uint64_t TIMEOUT = 10;
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    const auto service_name = iox2_testing::generate_service_name();
    auto node = NodeBuilder().create<SERVICE_TYPE>().value();

    auto service_create = node.service_builder(service_name).event().deadline(DEADLINE).create().value();
    auto listener = service_create.listener_builder().create().value();
    auto notifier_create = service_create.notifier_builder().create().value();

    auto service_open = node.service_builder(service_name).event().open().value();
    auto notifier_open = service_open.notifier_builder().create().value();

    std::this_thread::sleep_for(std::chrono::milliseconds(TIMEOUT));
    auto result = notifier_create.notify();
    ASSERT_THAT(result.has_value(), Eq(true));
    ASSERT_THAT(listener.try_wait_one().value().has_value(), Eq(true));

    std::this_thread::sleep_for(std::chrono::milliseconds(TIMEOUT));
    result = notifier_open.notify();
    ASSERT_THAT(result.has_value(), Eq(true));
    ASSERT_THAT(listener.try_wait_one().value().has_value(), Eq(true));
}

TYPED_TEST(ServiceEventTest, number_of_listener_notifier_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    const auto service_name = iox2_testing::generate_service_name();
    auto node = NodeBuilder().create<SERVICE_TYPE>().value();

    auto service = node.service_builder(service_name).event().create().value();

    ASSERT_THAT(service.dynamic_config().number_of_listeners(), Eq(0));
    ASSERT_THAT(service.dynamic_config().number_of_notifiers(), Eq(0));
    {
        auto listener = service.listener_builder().create().value();
        ASSERT_THAT(service.dynamic_config().number_of_listeners(), Eq(1));
        ASSERT_THAT(service.dynamic_config().number_of_notifiers(), Eq(0));

        auto notifier = service.notifier_builder().create().value();
        ASSERT_THAT(service.dynamic_config().number_of_listeners(), Eq(1));
        ASSERT_THAT(service.dynamic_config().number_of_notifiers(), Eq(1));
    }
    ASSERT_THAT(service.dynamic_config().number_of_listeners(), Eq(0));
    ASSERT_THAT(service.dynamic_config().number_of_notifiers(), Eq(0));
}

TYPED_TEST(ServiceEventTest, service_id_is_unique_per_service) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    const auto service_name_1 = iox2_testing::generate_service_name();
    const auto service_name_2 = iox2_testing::generate_service_name();
    auto node = NodeBuilder().create<SERVICE_TYPE>().value();

    auto service_1_create = node.service_builder(service_name_1).event().create().value();
    auto service_1_open = node.service_builder(service_name_1).event().open().value();
    auto service_2 = node.service_builder(service_name_2).event().create().value();

    ASSERT_THAT(service_1_create.service_id().c_str(), StrEq(service_1_open.service_id().c_str()));
    ASSERT_THAT(service_1_create.service_id().c_str(), Not(StrEq(service_2.service_id().c_str())));
}

//NOLINTBEGIN(readability-function-cognitive-complexity), false positive caused by ASSERT_THAT
TYPED_TEST(ServiceEventTest, list_service_nodes_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto node_name_1 = NodeName::create("Nala and The HypnoToad").value();
    const auto node_name_2 = NodeName::create("Can they be friends?").value();
    const auto service_name = iox2_testing::generate_service_name();

    auto node_1 = NodeBuilder().name(node_name_1).create<SERVICE_TYPE>().value();
    auto node_2 = NodeBuilder().name(node_name_2).create<SERVICE_TYPE>().value();

    auto sut_1 = node_1.service_builder(service_name).event().create().value();
    auto sut_2 = node_2.service_builder(service_name).event().open().value();

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

TYPED_TEST(ServiceEventTest, listing_all_notifiers_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    constexpr uint64_t NUMBER_OF_NOTIFIERS = 16;

    const auto service_name = iox2_testing::generate_service_name();
    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto sut = node.service_builder(service_name).event().max_notifiers(NUMBER_OF_NOTIFIERS).create().value();

    std::vector<iox2::Notifier<SERVICE_TYPE>> notifiers;
    notifiers.reserve(NUMBER_OF_NOTIFIERS);
    for (uint64_t idx = 0; idx < NUMBER_OF_NOTIFIERS; ++idx) {
        notifiers.push_back(sut.notifier_builder().create().value());
    }

    std::vector<UniqueNotifierId> notifier_ids;
    notifier_ids.reserve(NUMBER_OF_NOTIFIERS);
    sut.dynamic_config().list_notifiers([&](auto notifier_details_view) -> auto {
        notifier_ids.push_back(notifier_details_view.notifier_id());
        return CallbackProgression::Continue;
    });

    ASSERT_THAT(notifier_ids.size(), Eq(NUMBER_OF_NOTIFIERS));
    for (auto& notifier : notifiers) {
        auto iter = std::find(notifier_ids.begin(), notifier_ids.end(), notifier.id());
        ASSERT_THAT(iter, Ne(notifier_ids.end()));
    }
}

TYPED_TEST(ServiceEventTest, listing_all_notifiers_stops_on_request) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    constexpr uint64_t NUMBER_OF_NOTIFIERS = 13;

    const auto service_name = iox2_testing::generate_service_name();
    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto sut = node.service_builder(service_name).event().max_notifiers(NUMBER_OF_NOTIFIERS).create().value();

    std::vector<iox2::Notifier<SERVICE_TYPE>> notifiers;
    notifiers.reserve(NUMBER_OF_NOTIFIERS);
    for (uint64_t idx = 0; idx < NUMBER_OF_NOTIFIERS; ++idx) {
        notifiers.push_back(sut.notifier_builder().create().value());
    }

    auto counter = 0;
    sut.dynamic_config().list_notifiers([&](auto) -> auto {
        counter++;
        return CallbackProgression::Stop;
    });

    ASSERT_THAT(counter, Eq(1));
}

TYPED_TEST(ServiceEventTest, notifier_details_are_correct) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();
    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto sut = node.service_builder(service_name).event().create().value();

    iox2::Notifier<SERVICE_TYPE> notifier = sut.notifier_builder().create().value();

    auto counter = 0;
    sut.dynamic_config().list_notifiers([&](auto notifier_details_view) -> auto {
        counter++;
        EXPECT_TRUE(notifier_details_view.notifier_id() == notifier.id());
        EXPECT_TRUE(notifier_details_view.node_id() == node.id());
        return CallbackProgression::Continue;
    });

    ASSERT_THAT(counter, Eq(1));
}

TYPED_TEST(ServiceEventTest, listing_all_listeners_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    constexpr uint64_t NUMBER_OF_LISTENERS = 17;

    const auto service_name = iox2_testing::generate_service_name();
    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto sut = node.service_builder(service_name).event().max_listeners(NUMBER_OF_LISTENERS).create().value();

    std::vector<iox2::Listener<SERVICE_TYPE>> listeners;
    listeners.reserve(NUMBER_OF_LISTENERS);
    for (uint64_t idx = 0; idx < NUMBER_OF_LISTENERS; ++idx) {
        listeners.push_back(sut.listener_builder().create().value());
    }

    std::vector<UniqueListenerId> listener_ids;
    listener_ids.reserve(NUMBER_OF_LISTENERS);
    sut.dynamic_config().list_listeners([&](auto listener_details_view) -> auto {
        listener_ids.push_back(listener_details_view.listener_id());
        return CallbackProgression::Continue;
    });

    ASSERT_THAT(listener_ids.size(), Eq(NUMBER_OF_LISTENERS));
    for (auto& listener : listeners) {
        auto iter = std::find(listener_ids.begin(), listener_ids.end(), listener.id());
        ASSERT_THAT(iter, Ne(listener_ids.end()));
    }
}

TYPED_TEST(ServiceEventTest, listing_all_listeners_stops_on_request) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;
    constexpr uint64_t NUMBER_OF_LISTENERS = 13;

    const auto service_name = iox2_testing::generate_service_name();
    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto sut = node.service_builder(service_name).event().max_listeners(NUMBER_OF_LISTENERS).create().value();

    std::vector<iox2::Listener<SERVICE_TYPE>> listeners;
    listeners.reserve(NUMBER_OF_LISTENERS);
    for (uint64_t idx = 0; idx < NUMBER_OF_LISTENERS; ++idx) {
        listeners.push_back(sut.listener_builder().create().value());
    }

    auto counter = 0;
    sut.dynamic_config().list_listeners([&](auto) -> auto {
        counter++;
        return CallbackProgression::Stop;
    });

    ASSERT_THAT(counter, Eq(1));
}

TYPED_TEST(ServiceEventTest, listener_details_are_correct) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();
    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto sut = node.service_builder(service_name).event().create().value();

    iox2::Listener<SERVICE_TYPE> listener = sut.listener_builder().create().value();

    auto counter = 0;
    sut.dynamic_config().list_listeners([&](auto listener_details_view) -> auto {
        counter++;
        EXPECT_TRUE(listener_details_view.listener_id() == listener.id());
        EXPECT_TRUE(listener_details_view.node_id() == node.id());
        return CallbackProgression::Continue;
    });

    ASSERT_THAT(counter, Eq(1));
}

TYPED_TEST(ServiceEventTest, only_max_notifiers_can_be_created) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service = node.service_builder(service_name).event().max_notifiers(1).create().value();
    auto notifier = bb::Optional<Notifier<SERVICE_TYPE>>(service.notifier_builder().create().value());

    auto failing_sut = service.notifier_builder().create();
    ASSERT_FALSE(failing_sut.has_value());

    notifier.reset();

    auto sut = service.notifier_builder().create();
    ASSERT_TRUE(sut.has_value());
}

TYPED_TEST(ServiceEventTest, only_max_listeners_can_be_created) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().value();
    auto service = node.service_builder(service_name).event().max_listeners(1).create().value();
    auto listener = bb::Optional<Listener<SERVICE_TYPE>>(service.listener_builder().create().value());

    auto failing_sut = service.listener_builder().create();
    ASSERT_FALSE(failing_sut.has_value());

    listener.reset();

    auto sut = service.listener_builder().create();
    ASSERT_TRUE(sut.has_value());
}
} // namespace
