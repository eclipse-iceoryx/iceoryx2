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

#include <cstdint>
#include <iostream>

#include "custom_header.hpp"
#include "iox/duration.hpp"
#include "iox2/node.hpp"

constexpr iox::units::Duration CYCLE_TIME = iox::units::Duration::fromSeconds(1);

int main() {
    using namespace iox2;
    auto node = NodeBuilder().template create<ServiceType::Ipc>().expect("successful node creation");

    auto service = node.service_builder(ServiceName::create("My/Funk/ServiceName").expect("valid service name"))
                       .publish_subscribe<uint64_t>()
                       // define the CustomHeader as user_header which is stored in the
                       // beginning of every Sample
                       .user_header<CustomHeader>()
                       .open_or_create()
                       .expect("successful service creation/opening");

    auto publisher = service.publisher_builder().create().expect("successful publisher creation");

    auto counter = 0;
    while (node.wait(CYCLE_TIME) == NodeEvent::Tick) {
        counter += 1;
        auto sample = publisher.loan_uninit().expect("acquire sample");

        sample.user_header_mut().version = 123;
        sample.user_header_mut().timestamp = 80337 + counter;

        sample.write_payload(counter);

        send_sample(std::move(sample)).expect("send successful");

        std::cout << "Send sample " << counter << "..." << std::endl;
    }

    std::cout << "exit" << std::endl;

    return 0;
}
