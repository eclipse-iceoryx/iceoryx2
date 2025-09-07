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

#ifndef IOX2_EXAMPLES_CUSTOM_PUBLISHER_HPP
#define IOX2_EXAMPLES_CUSTOM_PUBLISHER_HPP

#include "iox2/iceoryx2.hpp"
#include "pubsub_event.hpp"
#include "transmission_data.hpp"

#include <utility>

constexpr uint64_t HISTORY_SIZE = 20;

// High-level publisher class that contains besides a publisher also a notifier and a listener.
// The notifier is used to send events like `PubSubEvent::SentSample` or `PubSubEvent::SentHistory`
// and the listener to wait for new subscribers.
class CustomPublisher : public iox2::FileDescriptorBased {
  public:
    CustomPublisher(const CustomPublisher&) = delete;
    CustomPublisher(CustomPublisher&&) = default;
    ~CustomPublisher() override {
        m_notifier
            .notify_with_custom_event_id(
                iox2::EventId(iox::from<PubSubEvent, size_t>(PubSubEvent::PublisherDisconnected)))
            .expect("");
    }

    static auto create(iox2::Node<iox2::ServiceType::Ipc>& node, const iox2::ServiceName& service_name)
        -> CustomPublisher {
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

        notifier
            .notify_with_custom_event_id(iox2::EventId(iox::from<PubSubEvent, size_t>(PubSubEvent::PublisherConnected)))
            .expect("");

        return CustomPublisher { std::move(publisher), std::move(listener), std::move(notifier) };
    }

    auto file_descriptor() const -> iox2::FileDescriptorView override {
        return m_listener.file_descriptor();
    }

    void handle_event() {
        for (auto event = m_listener.try_wait_one(); event.has_value() && event->has_value();
             event = m_listener.try_wait_one()) {
            switch (iox::from<size_t, PubSubEvent>(event.value()->as_value())) {
            case PubSubEvent::SubscriberConnected: {
                std::cout << "new subscriber connected - delivering history" << std::endl;
                m_publisher.update_connections().expect("");
                m_notifier
                    .notify_with_custom_event_id(
                        iox2::EventId(iox::from<PubSubEvent, size_t>(PubSubEvent::SentHistory)))
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

    void send(const uint64_t counter) {
        constexpr double SOME_NUMBER = 812.12;
        auto sample = m_publisher.loan_uninit().expect("");
        auto initialized_sample = sample.write_payload(TransmissionData {
            static_cast<int32_t>(counter), static_cast<int32_t>(counter), static_cast<double>(counter) * SOME_NUMBER });
        ::iox2::send(std::move(initialized_sample)).expect("");

        m_notifier.notify_with_custom_event_id(iox2::EventId(iox::from<PubSubEvent, size_t>(PubSubEvent::SentSample)))
            .expect("");
    }

    auto operator=(const CustomPublisher&) -> CustomPublisher& = delete;
    auto operator=(CustomPublisher&&) -> CustomPublisher& = default;

  private:
    CustomPublisher(iox2::Publisher<iox2::ServiceType::Ipc, TransmissionData, void>&& publisher,
                    iox2::Listener<iox2::ServiceType::Ipc>&& listener,
                    iox2::Notifier<iox2::ServiceType::Ipc>&& notifier)
        : m_publisher { std::move(publisher) }
        , m_listener { std::move(listener) }
        , m_notifier { std::move(notifier) } {
    }

    iox2::Publisher<iox2::ServiceType::Ipc, TransmissionData, void> m_publisher;
    iox2::Listener<iox2::ServiceType::Ipc> m_listener;
    iox2::Notifier<iox2::ServiceType::Ipc> m_notifier;
};

#endif
