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

#include "iox2/publisher.hpp"
#include "iox/duration.hpp"
#include "iox2/file_descriptor.hpp"
#include "iox2/listener.hpp"
#include "iox2/node.hpp"
#include "iox2/notifier.hpp"
#include "iox2/sample_mut.hpp"
#include "iox2/service_name.hpp"
#include "iox2/service_type.hpp"
#include "iox2/waitset.hpp"
#include "pubsub_event.hpp"
#include "transmission_data.hpp"

#include <iostream>
#include <utility>

using namespace iox2;

constexpr iox::units::Duration CYCLE_TIME = iox::units::Duration::fromSeconds(1);
constexpr uint64_t HISTORY_SIZE = 20;

// High-level publisher class that contains besides a publisher also a notifier and a listener.
// The notifier is used to send events like `PubSubEvent::SentSample` or `PubSubEvent::SentHistory`
// and the listener to wait for new subscribers.
class EventBasedPublisher : public FileDescriptorBased {
  public:
    EventBasedPublisher(const EventBasedPublisher&) = delete;
    EventBasedPublisher(EventBasedPublisher&&) = default;
    ~EventBasedPublisher() override;

    auto operator=(const EventBasedPublisher&) -> EventBasedPublisher& = delete;
    auto operator=(EventBasedPublisher&&) -> EventBasedPublisher& = default;

    static auto create(Node<ServiceType::Ipc>& node, const ServiceName& service_name) -> EventBasedPublisher;
    void handle_event();
    void send(uint64_t counter);
    auto file_descriptor() const -> FileDescriptorView override;

  private:
    EventBasedPublisher(Publisher<ServiceType::Ipc, TransmissionData, void>&& publisher,
                        Listener<ServiceType::Ipc>&& listener,
                        Notifier<ServiceType::Ipc>&& notifier);

    Publisher<ServiceType::Ipc, TransmissionData, void> m_publisher;
    Listener<ServiceType::Ipc> m_listener;
    Notifier<ServiceType::Ipc> m_notifier;
};

auto main() -> int {
    auto node = NodeBuilder().create<ServiceType::Ipc>().expect("successful node creation");
    auto publisher = EventBasedPublisher::create(node, ServiceName::create("My/Funk/ServiceName").expect(""));

    auto waitset = WaitSetBuilder().create<ServiceType::Ipc>().expect("");
    // Whenever our publisher receives an event we get notified.
    auto publisher_guard = waitset.attach_notification(publisher).expect("");
    // Attach an interval so that we wake up and can publish a new message
    auto cyclic_trigger_guard = waitset.attach_interval(CYCLE_TIME).expect("");

    uint64_t counter = 0;

    // Event callback that is called whenever the WaitSet received an event.
    auto on_event = [&](WaitSetAttachmentId<ServiceType::Ipc> attachment_id) -> CallbackProgression {
        // when the cyclic trigger guard gets notified we send out a new message
        if (attachment_id.has_event_from(cyclic_trigger_guard)) {
            std::cout << "send message: " << counter << std::endl;
            publisher.send(counter);
            counter += 1;
            // when something else happens on the publisher we handle the events
        } else if (attachment_id.has_event_from(publisher_guard)) {
            publisher.handle_event();
        }
        return CallbackProgression::Continue;
    };

    // Start the event loop. It will run until `CallbackProgression::Stop` is returned by the
    // event callback or an interrupt/termination signal was received.
    waitset.wait_and_process(on_event).expect("");

    std::cout << "exit ..." << std::endl;

    return 0;
}

EventBasedPublisher::EventBasedPublisher(Publisher<ServiceType::Ipc, TransmissionData, void>&& publisher,
                                         Listener<ServiceType::Ipc>&& listener,
                                         Notifier<ServiceType::Ipc>&& notifier)
    : m_publisher { std::move(publisher) }
    , m_listener { std::move(listener) }
    , m_notifier { std::move(notifier) } {
}

EventBasedPublisher::~EventBasedPublisher() {
    m_notifier.notify_with_custom_event_id(EventId(iox::from<PubSubEvent, size_t>(PubSubEvent::PublisherDisconnected)))
        .expect("");
}

auto EventBasedPublisher::create(Node<ServiceType::Ipc>& node, const ServiceName& service_name) -> EventBasedPublisher {
    auto pubsub_service = node.service_builder(service_name)
                              .publish_subscribe<TransmissionData>()
                              .history_size(HISTORY_SIZE)
                              .subscriber_max_buffer_size(HISTORY_SIZE)
                              .open_or_create()
                              .expect("");
    auto event_service = node.service_builder(service_name).event().open_or_create().expect("");

    auto notifier = event_service.notifier_builder().create().expect("");
    auto listener = event_service.listener_builder().create().expect("");
    auto publisher = pubsub_service.publisher_builder().create().expect("");

    notifier.notify_with_custom_event_id(EventId(iox::from<PubSubEvent, size_t>(PubSubEvent::PublisherConnected)))
        .expect("");

    return EventBasedPublisher { std::move(publisher), std::move(listener), std::move(notifier) };
}

auto EventBasedPublisher::file_descriptor() const -> FileDescriptorView {
    return m_listener.file_descriptor();
}

void EventBasedPublisher::handle_event() {
    for (auto event = m_listener.try_wait_one(); event.has_value() && event->has_value();
         event = m_listener.try_wait_one()) {
        switch (iox::from<size_t, PubSubEvent>(event.value()->as_value())) {
        case PubSubEvent::SubscriberConnected: {
            std::cout << "new subscriber connected - delivering history" << std::endl;
            m_publisher.update_connections().expect("");
            m_notifier.notify_with_custom_event_id(EventId(iox::from<PubSubEvent, size_t>(PubSubEvent::SentHistory)))
                .expect("");
            break;
        }
        case PubSubEvent::SubscriberDisconnected: {
            std::cout << "subscriber disconnected" << std::endl;
            break;
        }
        case PubSubEvent::ReceivedSample: {
            std::cout << "subscriber has consumed sample" << std::endl;
            break;
        }
        default: {
            break;
        }
        }
    }
}

void EventBasedPublisher::send(const uint64_t counter) {
    constexpr double SOME_NUMBER = 812.12;
    auto sample = m_publisher.loan_uninit().expect("");
    sample.write_payload(TransmissionData {
        static_cast<int32_t>(counter), static_cast<int32_t>(counter), static_cast<double>(counter) * SOME_NUMBER });
    auto initialized_sample = assume_init(std::move(sample));
    ::iox2::send(std::move(initialized_sample)).expect("");

    m_notifier.notify_with_custom_event_id(EventId(iox::from<PubSubEvent, size_t>(PubSubEvent::SentSample))).expect("");
}
