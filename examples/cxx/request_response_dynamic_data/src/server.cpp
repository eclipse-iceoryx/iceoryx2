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

constexpr iox2::bb::Duration CYCLE_TIME = iox2::bb::Duration::from_millis(100);
constexpr uint8_t MAX_VALUE = 255;

auto main() -> int {
    using namespace iox2;
    set_log_level_from_env_or(LogLevel::Info);
    auto node = NodeBuilder().create<ServiceType::Ipc>().value();

    auto service = node.service_builder(ServiceName::create("My/Funk/ServiceName").value())
                       .request_response<bb::Slice<uint8_t>, bb::Slice<uint8_t>>()
                       .open_or_create()
                       .value();

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
                      .value();

    std::cout << "Server ready to receive requests!" << std::endl;

    auto counter = 1U;

    while (node.wait(CYCLE_TIME).has_value()) {
        while (true) {
            auto active_request = server.receive().value();
            if (active_request.has_value()) {
                std::cout << "received request with " << active_request->payload().number_of_bytes() << " bytes ..."
                          << std::endl;

                uint64_t required_memory_size = std::min(1000000U, counter * counter); // NOLINT
                auto response = active_request->loan_slice_uninit(required_memory_size).value();
                auto initialized_response = response.write_from_fn(
                    [&](auto byte_idx) { return static_cast<uint8_t>((byte_idx + counter) % MAX_VALUE); }); // NOLINT
                std::cout << "send response with " << initialized_response.payload().number_of_bytes() << " bytes"
                          << std::endl;
                send(std::move(initialized_response)).value();
            } else {
                break;
            }
        }

        counter += 1;
    }

    std::cout << "exit" << std::endl;

    return 0;
}
