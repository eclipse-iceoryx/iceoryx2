// Copyright (c) 2025 Contributors to the Eclipse Foundation
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

#include <cstdint>
#include <iostream>

constexpr iox::units::Duration CYCLE_TIME = iox::units::Duration::fromMilliseconds(750);

auto main() -> int {
    using namespace iox2;
    set_log_level_from_env_or(LogLevel::Info);
    auto node = NodeBuilder()
                    // The generic argument defines the service variant. Different variants can use
                    // different mechanisms. For instance the upcoming `ServiceType::Cuda` would use GPU
                    // memory or the `local::Service` would use mechanisms that are optimized for
                    // intra-process communication.
                    //
                    // All services which are created via this `Node` use the same service variant.
                    .create<ServiceType::Ipc>()
                    .expect("successful node creation");

    auto service = node.service_builder(ServiceName::create("Service-Variants-Example").expect("valid service name"))
                       .publish_subscribe<uint64_t>()
                       .open_or_create()
                       .expect("successful service creation/opening");

    auto publisher = service.publisher_builder().create().expect("successful publisher creation");

    uint64_t counter = 0;
    while (node.wait(CYCLE_TIME).has_value()) {
        std::cout << "send: " << counter << std::endl;
        publisher.send_copy(counter).expect("sample was sent");
        counter += 1;
    }

    std::cout << "exit" << std::endl;

    return 0;
}
