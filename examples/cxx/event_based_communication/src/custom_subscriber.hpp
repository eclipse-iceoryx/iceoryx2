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

#ifndef IOX2_EXAMPLES_CUSTOM_SUBSCRIBER_HPP
#define IOX2_EXAMPLES_CUSTOM_SUBSCRIBER_HPP

#include "iox2/iceoryx2.hpp"
#include "pubsub_event.hpp"
#include "transmission_data.hpp"

constexpr uint64_t HISTORY_SIZE = 20;

// High-level subscriber class that contains besides a subscriber also a notifier
// and a listener. The notifier is used to send events like
// `PubSubEvent::ReceivedSample` or to notify the publisher that a new subscriber
// connected.
// The listener waits for events originating from the publisher like
// `PubSubEvent::SentSample`.
class CustomSubscriber : public iox2::FileDescriptorBased {
  public:
    CustomSubscriber(const CustomSubscriber&) = delete;
    CustomSubscriber(CustomSubscriber&&) = default;
    ~CustomSubscriber() override {
        m_notifier
            .notify_with_custom_event_id(
                iox2::EventId(iox::from<PubSubEvent, size_t>(PubSubEvent::SubscriberDisconnected)))
            .expect("");
    }

    auto operator=(const CustomSubscriber&) -> CustomSubscriber& = delete;
    auto operator=(CustomSubscriber&&) -> CustomSubscriber& = default;

    static auto create(iox2::Node<iox2::ServiceType::Ipc>& node, const iox2::ServiceName& service_name)
        -> CustomSubscriber {
        auto pubsub_service = node.service_builder(service_name)
                                  .publish_subscribe<TransmissionData>()
                                  .history_size(HISTORY_SIZE)
                                  .subscriber_max_buffer_size(HISTORY_SIZE)
                                  .open_or_create()
                                  .expect("");
        auto event_service = node.service_builder(service_name).event().open_or_create().expect("");

        auto listener = event_service.listener_builder().create().expect("");
        auto notifier = event_service.notifier_builder().create().expect("");
        auto subscriber = pubsub_service.subscriber_builder().create().expect("");

        notifier
            .notify_with_custom_event_id(
                iox2::EventId(iox::from<PubSubEvent, size_t>(PubSubEvent::SubscriberConnected)))
            .expect("");

        return CustomSubscriber { std::move(subscriber), std::move(notifier), std::move(listener) };
    }

    auto file_descriptor() const -> iox2::FileDescriptorView override {
        return m_listener.file_descriptor();
    }

    void handle_event() {
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

    auto receive() -> iox::optional<iox2::Sample<iox2::ServiceType::Ipc, TransmissionData, void>> {
        auto sample = m_subscriber.receive().expect("");
        if (sample.has_value()) {
            m_notifier
                .notify_with_custom_event_id(iox2::EventId(iox::from<PubSubEvent, size_t>(PubSubEvent::ReceivedSample)))
                .expect("");
        }

        return sample;
    }

  private:
    CustomSubscriber(iox2::Subscriber<iox2::ServiceType::Ipc, TransmissionData, void>&& subscriber,
                     iox2::Notifier<iox2::ServiceType::Ipc>&& notifier,
                     iox2::Listener<iox2::ServiceType::Ipc>&& listener)
        : m_subscriber { std::move(subscriber) }
        , m_notifier { std::move(notifier) }
        , m_listener { std::move(listener) } {
    }

    iox2::Subscriber<iox2::ServiceType::Ipc, TransmissionData, void> m_subscriber;
    iox2::Notifier<iox2::ServiceType::Ipc> m_notifier;
    iox2::Listener<iox2::ServiceType::Ipc> m_listener;
};

#endif
