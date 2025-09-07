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
#include <thread>

constexpr iox::units::Duration CYCLE_TIME = iox::units::Duration::fromSeconds(1);

auto main() -> int {
    using namespace iox2;
    set_log_level_from_env_or(LogLevel::Info);
    auto node = NodeBuilder()
                    // In contrast to Rust, all service variants in C++ have threadsafe ports
                    // but at the cost of an additional mutex lock/unlock call.
                    .create<ServiceType::Ipc>()
                    .expect("successful node creation");

    auto service = node.service_builder(ServiceName::create("Service-Variants-Example").expect("valid service name"))
                       .publish_subscribe<uint64_t>()
                       .open_or_create()
                       .expect("service created");

    std::mutex cout_mtx;
    auto keep_running = std::atomic<bool>(true);
    auto subscriber = service.subscriber_builder().create().expect("subscriber created");

    // All ports (like Subscriber, Publisher, Client, Server, ...) are threadsafe
    // so they can be shared between threads.
    auto background_thread = std::thread([&] {
        while (keep_running.load()) {
            std::this_thread::sleep_for(std::chrono::milliseconds(CYCLE_TIME.toMilliseconds()));
            auto sample = subscriber.receive().expect("sample received");
            if (sample.has_value()) {
                const std::lock_guard<std::mutex> cout_guard(cout_mtx);
                std::cout << "[thread] received: " << sample->payload() << std::endl;
            }
        }
    });

    while (node.wait(CYCLE_TIME).has_value()) {
        auto sample = subscriber.receive().expect("sample received");
        if (sample.has_value()) {
            const std::lock_guard<std::mutex> cout_guard(cout_mtx);
            std::cout << "[main] received: " << sample->payload() << std::endl;
        }
    }

    keep_running.store(false);
    background_thread.join();
    return 0;
}
