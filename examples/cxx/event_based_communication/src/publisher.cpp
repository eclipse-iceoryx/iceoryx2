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

#include "custom_publisher.hpp"
#include "iox2/iceoryx2.hpp"

#include <iostream>

using namespace iox2;

constexpr iox::units::Duration CYCLE_TIME = iox::units::Duration::fromSeconds(1);

auto main() -> int {
    set_log_level_from_env_or(LogLevel::Info);
    auto node = NodeBuilder().create<ServiceType::Ipc>().expect("successful node creation");
    auto publisher = CustomPublisher::create(node, ServiceName::create("My/Funk/ServiceName").expect(""));

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

    std::cout << "exit" << std::endl;

    return 0;
}
