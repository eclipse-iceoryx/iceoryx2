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

#include <iostream>

constexpr iox::units::Duration CYCLE_TIME = iox::units::Duration::fromSeconds(1);

auto main() -> int {
    using namespace iox2;
    set_log_level_from_env_or(LogLevel::Info);
    auto node = NodeBuilder().create<ServiceType::Ipc>().expect("successful node creation");

    auto service = node.service_builder(ServiceName::create("MyEventName").expect("valid service name"))
                       .event()
                       .open_or_create()
                       .expect("successful service creation/opening");
    auto max_event_id = service.static_config().event_id_max_value();

    auto notifier = service.notifier_builder().create().expect("successful notifier creation");

    auto counter = 0;
    while (node.wait(CYCLE_TIME).has_value()) {
        counter += 1;
        const auto event_id = EventId(counter % max_event_id);
        notifier.notify_with_custom_event_id(event_id).expect("notification");

        std::cout << "Trigger event with id " << event_id << "..." << std::endl;
    }

    std::cout << "exit" << std::endl;

    return 0;
}
