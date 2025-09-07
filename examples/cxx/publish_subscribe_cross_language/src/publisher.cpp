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
#include "message_data.hpp"

#include <iostream>
#include <utility>

constexpr iox::units::Duration CYCLE_TIME = iox::units::Duration::fromSeconds(1);

auto main() -> int {
    using namespace iox2;
    set_log_level_from_env_or(LogLevel::Info);
    auto node = NodeBuilder().create<ServiceType::Ipc>().expect("successful node creation");

    auto service = node.service_builder(ServiceName::create("My/Funk/ServiceName").expect("valid service name"))
                       .publish_subscribe<TransmissionData>()
                       .user_header<CustomHeader>()
                       .open_or_create()
                       .expect("successful service creation/opening");

    auto publisher = service.publisher_builder().create().expect("successful publisher creation");

    auto counter = 0;
    while (node.wait(CYCLE_TIME).has_value()) {
        counter += 1;

        auto sample = publisher.loan_uninit().expect("acquire sample");
        sample.user_header_mut().version = 123;               // NOLINT
        sample.user_header_mut().timestamp = 80337 + counter; // NOLINT

        auto initialized_sample =
            sample.write_payload(TransmissionData { counter, counter * 3, counter * 812.12 }); // NOLINT

        send(std::move(initialized_sample)).expect("send successful");

        std::cout << "Send sample " << counter << "..." << std::endl;
    }

    std::cout << "exit" << std::endl;

    return 0;
}
