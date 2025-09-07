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

#include "iox2/iceoryx2.hpp"
#include "pubsub_event.hpp"

using namespace iox2;

constexpr iox::units::Duration REACTION_BUFFER = iox::units::Duration::fromMilliseconds(100);
constexpr iox::units::Duration CYCLE_TIME_1 = iox::units::Duration::fromMilliseconds(1000) + REACTION_BUFFER;
constexpr iox::units::Duration CYCLE_TIME_2 = iox::units::Duration::fromMilliseconds(1500) + REACTION_BUFFER;

namespace {
void find_and_cleanup_dead_nodes();
void handle_incoming_events(Listener<ServiceType::Ipc>& listener,
                            const Subscriber<ServiceType::Ipc, uint64_t, void>& subscriber,
                            const ServiceName& service_name);
} // namespace

auto main() -> int {
    set_log_level_from_env_or(LogLevel::Info);
    auto service_name_1 = ServiceName::create("service_1").expect("");
    auto service_name_2 = ServiceName::create("service_2").expect("");

    auto node = NodeBuilder()
                    .name(NodeName::create("subscruber").expect(""))
                    .create<ServiceType::Ipc>()
                    .expect("successful node creation");

    // open a pubsub and an event service with the same name
    auto service_1 = open_service(node, service_name_1);
    auto service_2 = open_service(node, service_name_2);

    auto subscriber_1 = service_1.pubsub.subscriber_builder().create().expect("");
    auto subscriber_2 = service_2.pubsub.subscriber_builder().create().expect("");
    auto listener_1 = service_1.event.listener_builder().create().expect("");
    auto listener_2 = service_2.event.listener_builder().create().expect("");

    auto waitset = WaitSetBuilder().create<ServiceType::Ipc>().expect("");

    // If the service has defined a deadline we will use it, otherwise
    // we expect that the listener receive a message sent event after at most CYCLE_TIME_X
    auto deadline_1 = listener_1.deadline().value_or(CYCLE_TIME_1);
    auto deadline_2 = listener_2.deadline().value_or(CYCLE_TIME_2);
    auto listener_1_guard = waitset.attach_deadline(listener_1, deadline_1).expect("");
    auto listener_2_guard = waitset.attach_deadline(listener_2, deadline_2).expect("");

    auto missed_deadline = [](const ServiceName& service_name, const iox::units::Duration& cycle_time) {
        std::cout << service_name.to_string().c_str() << ": voilated contract and did not send a message after "
                  << cycle_time << std::endl;
    };

    auto on_event = [&](const WaitSetAttachmentId<ServiceType::Ipc>& attachment_id) {
        if (attachment_id.has_missed_deadline(listener_1_guard)) {
            missed_deadline(service_name_1, deadline_1);
            // one cause of a deadline it can be a dead node. usually our "central_daemon" would
            // take care of monitoring but when the node and the central daemon crashed we take
            // over here and check for dead nodes
            find_and_cleanup_dead_nodes();
        }

        if (attachment_id.has_missed_deadline(listener_2_guard)) {
            missed_deadline(service_name_2, deadline_2);
            find_and_cleanup_dead_nodes();
        }

        if (attachment_id.has_event_from(listener_1_guard)) {
            // in this function we either print out the received sample or the event that has
            // occurred like, publisher connected/disconnected or a process was identified as dead
            handle_incoming_events(listener_1, subscriber_1, service_name_1);
        }

        if (attachment_id.has_event_from(listener_2_guard)) {
            handle_incoming_events(listener_2, subscriber_2, service_name_2);
        }

        return CallbackProgression::Continue;
    };

    waitset.wait_and_process(on_event).expect("");

    std::cout << "exit" << std::endl;

    return 0;
}

namespace {
void handle_incoming_events(Listener<ServiceType::Ipc>& listener,
                            const Subscriber<ServiceType::Ipc, uint64_t, void>& subscriber,
                            const ServiceName& service_name) {
    listener
        .try_wait_all([&](auto event_id) {
            if (event_id == iox::into<EventId>(PubSubEvent::ProcessDied)) {
                std::cout << service_name.to_string().c_str() << ": process died!" << std::endl;
            } else if (event_id == iox::into<EventId>(PubSubEvent::PublisherConnected)) {
                std::cout << service_name.to_string().c_str() << ": publisher connected!" << std::endl;
            } else if (event_id == iox::into<EventId>(PubSubEvent::PublisherDisconnected)) {
                std::cout << service_name.to_string().c_str() << ": publisher disconnected!" << std::endl;
            } else if (event_id == iox::into<EventId>(PubSubEvent::SentSample)) {
                subscriber.receive().expect("").and_then([&](auto& sample) {
                    std::cout << service_name.to_string().c_str() << ": Received sample " << *sample << " ..."
                              << std::endl;
                });
            }
        })
        .expect("");
}

void find_and_cleanup_dead_nodes() {
    Node<ServiceType::Ipc>::list(Config::global_config(), [](auto node_state) {
        node_state.dead([](auto view) {
            std::cout << "detected dead node: ";
            view.details().and_then([](const auto& details) { std::cout << details.name().to_string().c_str(); });
            std::cout << std::endl;
            view.remove_stale_resources().expect("");
        });
        return CallbackProgression::Continue;
    }).expect("");
}
} // namespace
