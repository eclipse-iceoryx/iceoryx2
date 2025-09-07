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

#include <atomic>
#include <cstdint>
#include <iostream>
#include <mutex>

constexpr iox::units::Duration CYCLE_TIME = iox::units::Duration::fromSeconds(1);
namespace {
std::atomic<bool> keep_running = true; // NOLINT
std::mutex cout_mtx;                   // NOLINT

void background_thread_fn() {
    using namespace iox2;
    // Another node is created inside this thread to communicate with the main thread
    auto node = NodeBuilder()
                    // Optionally, a name can be provided to the node which helps identifying them
                    // later during debugging or introspection
                    .name(NodeName::create("threadnode").expect("valid node name"))
                    .create<ServiceType::Local>()
                    .expect("successful node creation");

    auto service = node.service_builder(ServiceName::create("Service-Variants-Example").expect("valid service name"))
                       .publish_subscribe<uint64_t>()
                       .open_or_create()
                       .expect("successful service creation/opening");

    auto subscriber = service.subscriber_builder().create().expect("successful subscriber creation");
    while (keep_running.load()) {
        std::this_thread::sleep_for(std::chrono::milliseconds(CYCLE_TIME.toMilliseconds()));
        auto sample = subscriber.receive().expect("sample received");
        while (sample.has_value()) {
            {
                const std::lock_guard<std::mutex> cout_guard(cout_mtx);
                std::cout << "[thread] received: " << sample->payload() << std::endl;
            }
            sample = subscriber.receive().expect("sample received");
        }
    }
}
} // namespace

auto main() -> int {
    using namespace iox2;
    set_log_level_from_env_or(LogLevel::Info);
    // When choosing `local::Service` the service does not use inter-process mechanisms
    // like shared memory or unix domain sockets but mechanisms like socketpairs and heap.
    //
    // Those services can communicate only within a single process.
    auto node = NodeBuilder()
                    // Optionally, a name can be provided to the node which helps identifying them
                    // later during debugging or introspection
                    .name(NodeName::create("mainnode").expect("valid node name"))
                    .create<ServiceType::Local>()
                    .expect("successful node creation");

    auto service = node.service_builder(ServiceName::create("Service-Variants-Example").expect("valid service name"))
                       .publish_subscribe<uint64_t>()
                       .open_or_create()
                       .expect("successful service creation/opening");

    auto publisher = service.publisher_builder().create().expect("successful publisher creation");
    auto background_thread = std::thread(background_thread_fn);

    uint64_t counter = 0;
    while (node.wait(CYCLE_TIME).has_value()) {
        {
            const std::lock_guard<std::mutex> lock(cout_mtx);
            std::cout << "send: " << counter << std::endl;
        }
        publisher.send_copy(counter).expect("send data");
        counter += 1;
    }

    keep_running.store(false);
    background_thread.join();

    std::cout << "exit" << std::endl;

    return 0;
}
