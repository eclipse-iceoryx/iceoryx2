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
#include <map>

#include "iox/cli_definition.hpp"
#include "iox/duration.hpp"
#include "iox/string.hpp"
#include "iox/vector.hpp"
#include "iox2/node.hpp"
#include "iox2/service_name.hpp"
#include "iox2/service_type.hpp"
#include "iox2/waitset.hpp"

constexpr iox::units::Duration CYCLE_TIME = iox::units::Duration::fromSeconds(1);

// NOLINTBEGIN
struct Args {
    IOX_CLI_DEFINITION(Args);
    IOX_CLI_OPTIONAL(iox::string<64>, service1, { "fuu" }, 's', "service1", "The name of service 1.");
    IOX_CLI_OPTIONAL(iox::string<64>, service2, { "bar" }, 't', "service2", "The name of service 2.");
    IOX_CLI_OPTIONAL(uint64_t, event_id, 0, 'e', "event_id", "The event id that shall be used to trigger the service.");
};
// NOLINTEND

struct ServiceNameListenerPair {
    iox2::ServiceName service_name;
    iox2::Listener<iox2::ServiceType::Ipc> listener;
};

auto main(int argc, char** argv) -> int {
    using namespace iox2;
    auto args = Args::parse(argc, argv, "Notifier of the event multiplexing example.");

    auto event_id = EventId(args.event_id());
    auto service_name_1 = ServiceName::create(args.service1().c_str()).expect("valid service name");
    auto service_name_2 = ServiceName::create(args.service2().c_str()).expect("valid service name");

    // create node and services
    auto node = NodeBuilder().create<ServiceType::Ipc>().expect("successful node creation");

    auto service_1 =
        node.service_builder(service_name_1).event().open_or_create().expect("successful service creation/opening");
    auto service_2 =
        node.service_builder(service_name_2).event().open_or_create().expect("successful service creation/opening");
    auto listener_1 = service_1.listener_builder().create().expect("successful listener creation");
    auto listener_2 = service_2.listener_builder().create().expect("successful listener creation");

    // create the waitset and attach the listeners to it
    auto waitset = WaitSetBuilder().create<ServiceType::Ipc>().expect("");
    // NOLINTNEXTLINE(misc-const-correctness) false positive
    iox::vector<WaitSetGuard<ServiceType::Ipc>, 2> guards;

    guards.emplace_back(waitset.attach_notification(listener_1).expect(""));
    guards.emplace_back(waitset.attach_notification(listener_2).expect(""));

    // NOLINTNEXTLINE(misc-const-correctness) false positive
    std::map<WaitSetAttachmentId<ServiceType::Ipc>, ServiceNameListenerPair> listeners;

    listeners.emplace(WaitSetAttachmentId<ServiceType::Ipc>::from_guard(guards[0]),
                      ServiceNameListenerPair { service_name_1, std::move(listener_1) });
    listeners.emplace(WaitSetAttachmentId<ServiceType::Ipc>::from_guard(guards[1]),
                      ServiceNameListenerPair { service_name_2, std::move(listener_2) });

    // the callback that is called when a listener has received an event
    auto on_event = [&](WaitSetAttachmentId<ServiceType::Ipc> attachment_id) {
        if (auto entry = listeners.find(attachment_id); entry != listeners.end()) {
            std::cout << "Received trigger from \"" << entry->second.service_name.to_string().c_str() << "\""
                      << std::endl;

            auto& listener = entry->second.listener;
            // IMPORTANT:
            // We need to collect all notifications since the WaitSet will wake us up as long as
            // there is something to read. If we skip this step completely we will end up in a
            // busy loop.
            listener.try_wait_all([](auto event_id) { std::cout << " " << event_id; }).expect("");
            std::cout << std::endl;
        }

        return iox2::CallbackProgression::Continue;
    };

    std::cout << "Waiting on the following services: " << service_name_1.to_string().c_str() << ", "
              << service_name_2.to_string().c_str() << std::endl;

    // loops until the user has pressed CTRL+c, the application has received a SIGTERM or SIGINT
    // signal or the user has called explicitly `waitset.stop()` in the `on_event` callback. We
    // didn't add this to the example so feel free to play around with it.
    waitset.wait_and_process(on_event).expect("");

    std::cout << "exit" << std::endl;

    return 0;
}
