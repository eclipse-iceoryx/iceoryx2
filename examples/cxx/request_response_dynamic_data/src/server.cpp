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

constexpr iox::units::Duration CYCLE_TIME = iox::units::Duration::fromMilliseconds(100);

auto main() -> int {
    using namespace iox2;
    set_log_level_from_env_or(LogLevel::Info);
    auto node = NodeBuilder().create<ServiceType::Ipc>().expect("successful node creation");

    auto service = node.service_builder(ServiceName::create("My/Funk/ServiceName").expect("valid service name"))
                       .request_response<iox::Slice<uint8_t>, iox::Slice<uint8_t>>()
                       .open_or_create()
                       .expect("successful service creation/opening");

    constexpr uint64_t INITIAL_SIZE_HINT = 16;
    auto server = service
                      .server_builder()
                      // We guess that the samples are at most 16 bytes in size.
                      // This is just a hint to the underlying allocator and is purely optional.
                      // The better the guess is the less reallocations will be performed.
                      .initial_max_slice_len(INITIAL_SIZE_HINT)
                      // The underlying sample size will be increased with a power of two strategy
                      // when [`ActiveRequest::loan_slice()`] or [`ActiveRequest::loan_slice_uninit()`]
                      // requires more memory than available.
                      .allocation_strategy(AllocationStrategy::PowerOfTwo)
                      .create()
                      .expect("successful server creation");

    std::cout << "Server ready to receive requests!" << std::endl;

    auto counter = 1;

    while (node.wait(CYCLE_TIME).has_value()) {
        while (true) {
            auto active_request = server.receive().expect("receive successful");
            if (active_request.has_value()) {
                std::cout << "received request with " << active_request->payload().number_of_bytes() << " bytes ..."
                          << std::endl;

                auto required_memory_size = std::min(1000000, counter * counter); // NOLINT
                auto response = active_request->loan_slice_uninit(required_memory_size).expect("loan successful");
                auto initialized_response =
                    response.write_from_fn([&](auto byte_idx) { return (byte_idx + counter) % 255; }); // NOLINT
                std::cout << "send response with " << initialized_response.payload().number_of_bytes() << " bytes"
                          << std::endl;
                send(std::move(initialized_response)).expect("send successful");
            } else {
                break;
            }
        }

        counter += 1;
    }

    std::cout << "exit" << std::endl;

    return 0;
}
