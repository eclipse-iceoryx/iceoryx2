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

constexpr iox2::bb::Duration CYCLE_TIME = iox2::bb::Duration::from_secs(1);

auto main() -> int {
    using namespace iox2;
    set_log_level_from_env_or(LogLevel::Info);
    auto node = NodeBuilder().create<ServiceType::Ipc>().value();

    auto service = node.service_builder(ServiceName::create("MyEventName").value()).event().open_or_create().value();

    auto listener = service.listener_builder().create().value();

    std::cout << "Listener ready to receive events!" << std::endl;

    while (node.wait(iox2::bb::Duration::zero()).has_value()) {
        auto number_of_notifications = listener.timed_wait(
            [](auto event) -> void {
                std::cout << "event was triggered with id: " << event.id() << " " << event.count() << "times"
                          << std::endl;
            },
            CYCLE_TIME);
    }

    std::cout << "exit" << std::endl;

    return 0;
}
