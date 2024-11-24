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

#include <iostream>

#include "iox/duration.hpp"
#include "iox/into.hpp"
#include "iox2/file_descriptor.hpp"
#include "iox2/listener.hpp"
#include "iox2/node.hpp"
#include "iox2/notifier.hpp"
#include "iox2/service_name.hpp"
#include "iox2/service_type.hpp"
#include "iox2/subscriber.hpp"
#include "iox2/waitset.hpp"
#include "pubsub_event.hpp"
#include "transmission_data.hpp"

constexpr iox::units::Duration DEADLINE = iox::units::Duration::fromSeconds(2);

using namespace iox2;

// High-level subscriber class that contains besides a subscriber also a notifier
// and a listener. The notifier is used to send events like
// `PubSubEvent::ReceivedSample` or to notify the publisher that a new subscriber
// connected.
// The listener waits for events originating from the publisher like
// `PubSubEvent::SentSample`.
class EventBasedSubscriber : public FileDescriptorBased {
  public:
    EventBasedSubscriber(const EventBasedSubscriber&) = delete;
    EventBasedSubscriber(EventBasedSubscriber&&) = default;
    ~EventBasedSubscriber() override;
    auto operator=(const EventBasedSubscriber&) -> EventBasedSubscriber& = delete;
    auto operator=(EventBasedSubscriber&&) -> EventBasedSubscriber& = default;

    static auto create(Node<ServiceType::Ipc>& node, const ServiceName& service_name) -> EventBasedSubscriber;
    auto file_descriptor() const -> FileDescriptorView override;
    void handle_event();
    auto receive() -> iox::optional<Sample<ServiceType::Ipc, TransmissionData, void>>;

  private:
    EventBasedSubscriber(Subscriber<ServiceType::Ipc, TransmissionData, void>&& subscriber,
                         Notifier<ServiceType::Ipc>&& notifier,
                         Listener<ServiceType::Ipc>&& listener);

    Subscriber<ServiceType::Ipc, TransmissionData, void> m_subscriber;
    Notifier<ServiceType::Ipc> m_notifier;
    Listener<ServiceType::Ipc> m_listener;
};

auto main() -> int {
    auto node = NodeBuilder().create<ServiceType::Ipc>().expect("successful node creation");

    auto subscriber = EventBasedSubscriber::create(node, ServiceName::create("My/Funk/ServiceName").expect(""));

    auto waitset = WaitSetBuilder().create<ServiceType::Ipc>().expect("");

    // The subscriber is attached as a deadline, meaning that we expect some activity
    // latest after the deadline has passed.
    auto subscriber_guard = waitset.attach_deadline(subscriber, DEADLINE).expect("");

    auto on_event = [&](WaitSetAttachmentId<ServiceType::Ipc> attachment_id) {
        // If we have received a new event on the subscriber we handle it.
        if (attachment_id.has_event_from(subscriber_guard)) {
            subscriber.handle_event();
            // If the subscriber did not receive an event until DEADLINE has
            // passed, we print out a warning.
        } else if (attachment_id.has_missed_deadline(subscriber_guard)) {
            std::cout << "Contract violation! The subscriber did not receive a message for " << DEADLINE << std::endl;
        }

        return CallbackProgression::Continue;
    };

    waitset.wait_and_process(on_event).expect("");

    std::cout << "exit" << std::endl;

    return 0;
}

EventBasedSubscriber::EventBasedSubscriber(Subscriber<ServiceType::Ipc, TransmissionData, void>&& subscriber,
                                           Notifier<ServiceType::Ipc>&& notifier,
                                           Listener<ServiceType::Ipc>&& listener)
    : m_subscriber { std::move(subscriber) }
    , m_notifier { std::move(notifier) }
    , m_listener { std::move(listener) } {
}

EventBasedSubscriber::~EventBasedSubscriber() {
    m_notifier.notify_with_custom_event_id(EventId(iox::from<PubSubEvent, size_t>(PubSubEvent::SubscriberDisconnected)))
        .expect("");
}


auto EventBasedSubscriber::create(Node<ServiceType::Ipc>& node,
                                  const ServiceName& service_name) -> EventBasedSubscriber {
    auto pubsub_service =
        node.service_builder(service_name).publish_subscribe<TransmissionData>().open_or_create().expect("");
    auto event_service = node.service_builder(service_name).event().open_or_create().expect("");

    auto listener = event_service.listener_builder().create().expect("");
    auto notifier = event_service.notifier_builder().create().expect("");
    auto subscriber = pubsub_service.subscriber_builder().create().expect("");

    notifier.notify_with_custom_event_id(EventId(iox::from<PubSubEvent, size_t>(PubSubEvent::SubscriberConnected)))
        .expect("");

    return EventBasedSubscriber { std::move(subscriber), std::move(notifier), std::move(listener) };
}

auto EventBasedSubscriber::file_descriptor() const -> FileDescriptorView {
    return m_listener.file_descriptor();
}

void EventBasedSubscriber::handle_event() {
    for (auto event = m_listener.try_wait_one(); event.has_value() && event->has_value();
         event = m_listener.try_wait_one()) {
        switch (iox::from<size_t, PubSubEvent>(event.value()->as_value())) {
        case PubSubEvent::SentHistory: {
            std::cout << "History delivered" << std::endl;
            for (auto sample = receive(); sample.has_value(); sample = receive()) {
                std::cout << "  history: " << sample->payload().x << std::endl;
            }
            break;
        }
        case PubSubEvent::SentSample: {
            for (auto sample = receive(); sample.has_value(); sample = receive()) {
                std::cout << "received: " << sample->payload().x << std::endl;
            }
            break;
        }
        case PubSubEvent::PublisherConnected: {
            std::cout << "new publisher connected" << std::endl;
            break;
        }
        case PubSubEvent::PublisherDisconnected: {
            std::cout << "publisher disconnected" << std::endl;
            break;
        }
        default: {
            break;
        }
        }
    }
}

auto EventBasedSubscriber::receive() -> iox::optional<Sample<ServiceType::Ipc, TransmissionData, void>> {
    auto sample = m_subscriber.receive().expect("");
    if (sample.has_value()) {
        m_notifier.notify_with_custom_event_id(EventId(iox::from<PubSubEvent, size_t>(PubSubEvent::ReceivedSample)))
            .expect("");
    }

    return sample;
}
