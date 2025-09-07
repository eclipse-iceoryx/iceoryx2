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

constexpr iox::units::Duration CYCLE_TIME = iox::units::Duration::fromSeconds(1);

auto main() -> int {
    using namespace iox2;
    set_log_level_from_env_or(LogLevel::Info);
    auto node = NodeBuilder().create<ServiceType::Ipc>().expect("successful node creation");

    auto service = node.service_builder(ServiceName::create("My/Funk/ServiceName").expect("valid service name"))
                       .request_response<iox::Slice<uint8_t>, iox::Slice<uint8_t>>()
                       .open_or_create()
                       .expect("successful service creation/opening");

    constexpr uint64_t INITIAL_SIZE_HINT = 16;
    auto client = service
                      .client_builder()
                      // We guess that the samples are at most 16 bytes in size.
                      // This is just a hint to the underlying allocator and is purely optional.
                      // The better the guess is the less reallocations will be performed.
                      .initial_max_slice_len(INITIAL_SIZE_HINT)
                      // The underlying sample size will be increased with a power of two strategy
                      // when [`Client::loan_slice()`] or [`Client::loan_slice_uninit()`] requires more
                      // memory than available.
                      .allocation_strategy(AllocationStrategy::PowerOfTwo)
                      .create()
                      .expect("successful client creation");

    auto counter = 1;

    while (true) {
        auto required_memory_size = std::min(1000000, counter * counter); // NOLINT
        auto request = client.loan_slice_uninit(required_memory_size).expect("loan successful");
        auto initialized_request =
            request.write_from_fn([&](auto byte_idx) { return (byte_idx + counter) % 255; }); // NOLINT
        auto pending_response = send(std::move(initialized_request)).expect("send successful");
        std::cout << "send request " << counter << " with " << required_memory_size << " bytes ..." << std::endl;

        if (!node.wait(CYCLE_TIME).has_value()) {
            break;
        }

        // acquire all responses to our request from our buffer that were sent by the servers
        while (true) {
            auto response = pending_response.receive().expect("receive successful");
            if (response.has_value()) {
                std::cout << "received response with " << response->payload().number_of_bytes() << " bytes"
                          << std::endl;
            } else {
                break;
            }
        }

        counter += 1;
    }

    std::cout << "exit" << std::endl;

    return 0;
}
