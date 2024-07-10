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

#include "iox/duration.hpp"
#include "iox2/node.hpp"

constexpr iox::units::Duration CYCLE_TIME =
    iox::units::Duration::fromSeconds(1);

int main() {
    using namespace iox2;
    auto node = NodeBuilder().template create<ServiceType::Ipc>().expect(
        "successful node creation");

    auto service =
        node.service_builder(
                ServiceName::create("MyEventName").expect("valid service name"))
            .event()
            .open_or_create()
            .expect("successful service creation/opening");

    auto notifier = service.notifier_builder().create().expect(
        "successful notifier creation");

    auto counter = 0;
    while (node.wait(CYCLE_TIME) == NodeEvent::Tick) {
        counter += 1;
        notifier.notify_with_custom_event_id(EventId(counter))
            .expect("notification");

        std::cout << "Trigger event with id " << counter << "..." << std::endl;
    }

    std::cout << "exit" << std::endl;

    return 0;
}
