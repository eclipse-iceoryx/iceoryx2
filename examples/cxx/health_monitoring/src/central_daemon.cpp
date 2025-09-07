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

constexpr iox::units::Duration CYCLE_TIME = iox::units::Duration::fromMilliseconds(100);
constexpr iox::units::Duration DEADLINE_SERVICE_1 = iox::units::Duration::fromMilliseconds(1500);
constexpr iox::units::Duration DEADLINE_SERVICE_2 = iox::units::Duration::fromMilliseconds(2000);

namespace {
void find_and_cleanup_dead_nodes();
}

auto main() -> int {
    set_log_level_from_env_or(LogLevel::Info);
    auto service_name_1 = ServiceName::create("service_1").expect("");
    auto service_name_2 = ServiceName::create("service_2").expect("");

    auto node = NodeBuilder()
                    .name(NodeName::create("central daemon").expect(""))
                    .create<ServiceType::Ipc>()
                    .expect("successful node creation");

    // The central daemon is responsible to create all services before hand and the other processes
    // just open the communication resources and start communicating.
    auto service_pubsub_1 = node.service_builder(service_name_1)
                                .publish_subscribe<uint64_t>()
                                // We use here open_or_create so that, in case of a crash of the central daemon, it can
                                // be restarted.
                                .open_or_create()
                                .expect("successful service creation/opening");

    auto service_event_1 = node.service_builder(service_name_1)
                               .event()
                               // Defines the maximum timespan between two notifications for this service. The user of a
                               // notifier that send a notification after the deadline was already reached, receives an
                               // MISSED_DEADLINE error after the notification was delivered.
                               .deadline(DEADLINE_SERVICE_1)
                               // Whenever a new notifier is created the PublisherConnected event is emitted. this makes
                               // sense since in this example a notifier is always created after a new publisher was
                               // created.
                               // The task of the notifier/event is it to inform and wake up other processes when
                               // certain system event have happened.
                               .notifier_created_event(iox::into<EventId>(PubSubEvent::PublisherConnected))
                               .notifier_dropped_event(iox::into<EventId>(PubSubEvent::PublisherDisconnected))
                               // This event is emitted when either the central daemon or a decentralized process
                               // detects a dead node and cleaned up all of its stale resources succesfully.
                               .notifier_dead_event(iox::into<EventId>(PubSubEvent::ProcessDied))
                               .open_or_create()
                               .expect("successful service creation/opening");

    auto service_pubsub_2 = node.service_builder(service_name_2)
                                .publish_subscribe<uint64_t>()
                                .open_or_create()
                                .expect("successful service creation/opening");

    auto service_event_2 = node.service_builder(service_name_2)
                               .event()
                               .deadline(DEADLINE_SERVICE_2)
                               .notifier_created_event(iox::into<EventId>(PubSubEvent::PublisherConnected))
                               .notifier_dropped_event(iox::into<EventId>(PubSubEvent::PublisherDisconnected))
                               .notifier_dead_event(iox::into<EventId>(PubSubEvent::ProcessDied))
                               .open_or_create()
                               .expect("successful service creation/opening");

    auto waitset = WaitSetBuilder().create<ServiceType::Ipc>().expect("");
    auto cycle_guard = waitset.attach_interval(CYCLE_TIME);

    std::cout << "Central daemon up and running." << std::endl;
    waitset
        // The only task of our central daemon is it to monitor all running nodes and cleanup their
        // resources if a process has died.
        //
        // Since we added the notifier_dead_event to the service, all listeners, that are waiting
        // on a service where one participant has died, will be woken up and they receive
        // the PubSubEvent::ProcessDied
        .wait_and_process([](auto) {
            find_and_cleanup_dead_nodes();
            return CallbackProgression::Continue;
        })
        .expect("");

    std::cout << "exit" << std::endl;

    return 0;
}

namespace {
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
