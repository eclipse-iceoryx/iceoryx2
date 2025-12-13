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

#include <cstdint>
#include <iostream>
#include <utility>

constexpr iox2::bb::Duration CYCLE_TIME = iox2::bb::Duration::from_secs(1);
constexpr uint8_t MAX_VALUE = 255;

auto main() -> int {
    using namespace iox2;
    set_log_level_from_env_or(LogLevel::Info);
    auto node = NodeBuilder().create<ServiceType::Ipc>().expect("successful node creation");

    auto service = node.service_builder(ServiceName::create("Service With Dynamic Data").expect("valid service name"))
                       .publish_subscribe<iox::Slice<uint8_t>>()
                       .open_or_create()
                       .expect("successful service creation/opening");

    // Since the payload type is uint8_t, this number is the same as the number of bytes in the payload.
    // For other types, number of bytes used by the payload will be max_slice_len * sizeof(Payload::ValueType)
    constexpr uint64_t INITIAL_SIZE_HINT = 16;
    auto publisher = service
                         .publisher_builder()
                         // We guess that the samples are at most 16 bytes in size.
                         // This is just a hint to the underlying allocator and is purely optional
                         // The better the guess is the less reallocations will be performed
                         .initial_max_slice_len(INITIAL_SIZE_HINT)
                         // The underlying sample size will be increased with a power of two strategy
                         // when [`Publisher::loan_slice()`] or [`Publisher::loan_slice_uninit()`] require more
                         // memory than available.
                         .allocation_strategy(AllocationStrategy::PowerOfTwo)
                         .create()
                         .expect("successful publisher creation");

    uint64_t counter = 0;

    while (node.wait(CYCLE_TIME).has_value()) {
        const uint64_t required_memory_size = (counter + 1) * (counter + 1); // NOLINT
        auto sample = publisher.loan_slice_uninit(required_memory_size).expect("acquire sample");
        auto initialized_sample = sample.write_from_fn(
            [&](auto byte_idx) { return static_cast<uint8_t>((byte_idx + counter) % MAX_VALUE); }); // NOLINT

        send(std::move(initialized_sample)).expect("send successful");

        std::cout << "Send sample " << counter << " with " << required_memory_size << " bytes..." << std::endl;

        counter++;
    }

    std::cout << "exit" << std::endl;

    return 0;
}
