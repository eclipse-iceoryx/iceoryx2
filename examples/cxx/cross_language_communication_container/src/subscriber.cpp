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

#include "iox2/container/static_string.hpp"
#include "iox2/container/static_vector.hpp"
#include "iox2/iceoryx2.hpp"

#include <cstdint>
#include <iostream>

constexpr iox2::bb::Duration CYCLE_TIME = iox2::bb::Duration::fromSeconds(1);

auto main() -> int {
    using namespace iox2;
    set_log_level_from_env_or(LogLevel::Info);
    auto node = NodeBuilder().create<ServiceType::Ipc>().expect("successful node creation");

    auto service = node.service_builder(ServiceName::create("CrossLanguageContainer").expect("valid service name"))
                       .publish_subscribe<iox2::container::StaticVector<uint64_t, 32>>() // NOLINT
                       .user_header<iox2::container::StaticString<64>>()                 // NOLINT
                       // add some QoS, disable safe overflow and the subscriber shall get the
                       // last 5 samples when connecting to the service
                       .history_size(5)               // NOLINT
                       .subscriber_max_buffer_size(5) // NOLINT
                       .enable_safe_overflow(false)
                       .open_or_create()
                       .expect("successful service creation/opening");

    auto subscriber = service.subscriber_builder().create().expect("successful subscriber creation");

    std::cout << "Subscriber ready to receive data!" << std::endl;

    while (node.wait(CYCLE_TIME).has_value()) {
        auto sample = subscriber.receive().expect("receive succeeds");
        while (sample.has_value()) {
            std::cout << "received: " << sample->payload() << ", user_header: " << sample->user_header() << std::endl;
            sample = subscriber.receive().expect("receive succeeds");
        }
    }

    std::cout << "exit" << std::endl;

    return 0;
}
