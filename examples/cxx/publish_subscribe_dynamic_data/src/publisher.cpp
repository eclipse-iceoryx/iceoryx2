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

#include "iox/duration.hpp"
#include "iox/slice.hpp"
#include "iox2/node.hpp"
#include "iox2/sample_mut.hpp"
#include "iox2/service_name.hpp"
#include "iox2/service_type.hpp"

#include <cstdint>
#include <iostream>
#include <utility>

constexpr iox::units::Duration CYCLE_TIME = iox::units::Duration::fromSeconds(1);

auto main() -> int {
    using namespace iox2;
    auto node = NodeBuilder().create<ServiceType::Ipc>().expect("successful node creation");

    auto service = node.service_builder(ServiceName::create("Service With Dynamic Data").expect("valid service name"))
                       .publish_subscribe<iox::Slice<uint8_t>>()
                       .open_or_create()
                       .expect("successful service creation/opening");

    // Since the payload type is uint8_t, this number is the same as the number of bytes in the payload.
    // For other types, number of bytes used by the payload will be max_slice_len * sizeof(Payload::ValueType)
    const uint64_t maximum_elements = 1024; // NOLINT
    auto publisher =
        service.publisher_builder().max_slice_len(maximum_elements).create().expect("successful publisher creation");

    auto counter = 0;

    while (node.wait(CYCLE_TIME).has_value()) {
        const auto required_memory_size = (counter % 16) + 1; // NOLINT
        auto sample = publisher.loan_slice_uninit(required_memory_size).expect("acquire sample");
        sample.write_from_fn([&](auto byte_idx) { return (byte_idx + counter) % 255; }); // NOLINT

        auto initialized_sample = assume_init(std::move(sample));
        send(std::move(initialized_sample)).expect("send successful");

        std::cout << "Send sample " << counter << " with " << required_memory_size << " bytes..." << std::endl;

        counter++;
    }

    std::cout << "exit" << std::endl;

    return 0;
}
