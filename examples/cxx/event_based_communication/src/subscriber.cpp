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

#include "custom_subscriber.hpp"
#include "iox2/iceoryx2.hpp"

constexpr iox::units::Duration DEADLINE = iox::units::Duration::fromSeconds(2);

using namespace iox2;

auto main() -> int {
    set_log_level_from_env_or(LogLevel::Info);
    auto node = NodeBuilder().create<ServiceType::Ipc>().expect("successful node creation");

    auto subscriber = CustomSubscriber::create(node, ServiceName::create("My/Funk/ServiceName").expect(""));

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
