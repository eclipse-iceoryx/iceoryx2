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

constexpr iox::units::Duration CYCLE_TIME = iox::units::Duration::fromSeconds(1);

auto main() -> int {
    using namespace iox2;
    set_log_level_from_env_or(LogLevel::Info);
    auto node = NodeBuilder().create<ServiceType::Ipc>().expect("successful node creation");

    auto service = node.service_builder(ServiceName::create("Service/With/Properties").expect("valid service name"))
                       .publish_subscribe<uint64_t>()
                       .create_with_attributes(
                           // define a set of properties that are static for the lifetime
                           // of the service
                           AttributeSpecifier()
                               .define("dds_service_mapping", "my_funky_service_name")
                               .define("tcp_serialization_format", "cdr")
                               .define("someip_service_mapping", "1/2/3")
                               .define("camera_resolution", "1920x1080"))
                       .expect("successful service creation/opening");

    auto publisher = service.publisher_builder().create().expect("successful publisher creation");

    std::cout << "defined service attributes: " << service.attributes() << std::endl;

    while (node.wait(CYCLE_TIME).has_value()) {
        auto sample = publisher.loan().expect("acquire sample");
        sample.payload_mut() = 0;
        send(std::move(sample)).expect("send successful");
    }

    std::cout << "exit" << std::endl;

    return 0;
}
