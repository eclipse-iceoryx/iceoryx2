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

#include "iox2/iceoryx2.hpp"
#include "pubsub_event.hpp"

#include <iostream>

constexpr iox::units::Duration CYCLE_TIME = iox::units::Duration::fromMilliseconds(1000);

auto main() -> int {
    using namespace iox2;
    set_log_level_from_env_or(LogLevel::Info);
    auto service_name = ServiceName::create("service_1").expect("");
    auto node = NodeBuilder()
                    .name(NodeName::create("publisher 1").expect(""))
                    .create<ServiceType::Ipc>()
                    .expect("successful node creation");

    auto service = open_service(node, service_name);

    auto publisher = service.pubsub.publisher_builder().create().expect("");
    auto notifier = service.event
                        .notifier_builder()
                        // we only want to notify the other side explicitly when we have sent a sample
                        // so we can define it as default event id
                        .default_event_id(iox::into<EventId>(PubSubEvent::SentSample))
                        .create()
                        .expect("");
    auto counter = 0;

    auto waitset = WaitSetBuilder().create<ServiceType::Ipc>().expect("");

    // we only want to notify the other side explicitly when we have sent a sample
    // so we can define it as default event id
    auto cycle_guard = waitset.attach_interval(CYCLE_TIME);

    waitset
        .wait_and_process([&](auto) {
            std::cout << service_name.to_string().c_str() << ": Send sample " << counter << " ..." << std::endl;
            publisher.send_copy(counter).expect("");
            notifier.notify().expect("");
            counter += 1;
            return CallbackProgression::Continue;
        })
        .expect("");

    std::cout << "exit" << std::endl;

    return 0;
}
