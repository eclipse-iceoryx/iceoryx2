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

    uint64_t worst_case_memory_size = 1024; // NOLINT
    auto publisher = service.publisher_builder()
                         .max_slice_len(worst_case_memory_size)
                         .create()
                         .expect("successful publisher creation");

    auto counter = 1;

    while (node.wait(CYCLE_TIME).has_value()) {
        counter += 1;

        auto required_memory_size = (8 + counter) % 16; // NOLINT
        auto sample = publisher.loan_slice_uninit(required_memory_size).expect("acquire sample");
        sample.write_from_fn([&](auto byte_idx) { return (byte_idx + counter) % 255; }); // NOLINT

        auto initialized_sample = assume_init(std::move(sample));
        send(std::move(initialized_sample)).expect("send successful");

        std::cout << "Send sample " << counter << "..." << std::endl;
    }

    std::cout << "exit" << std::endl;

    return 0;
}
