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

auto main() -> int {
    using namespace iox2;
    set_log_level_from_env_or(LogLevel::Info);
    auto node = NodeBuilder().create<ServiceType::Ipc>().value();
    // define a set of properties that are static for the lifetime
    // of the service
    auto attribute_specifier = AttributeSpecifier();
    attribute_specifier
        .define(*Attribute::Key::from_utf8("dds_service_mapping"),
                *Attribute::Value::from_utf8("my_funky_service_name"))
        .value();
    attribute_specifier
        .define(*Attribute::Key::from_utf8("tcp_serialization_format"), *Attribute::Value::from_utf8("cdr"))
        .value();
    attribute_specifier
        .define(*Attribute::Key::from_utf8("someip_service_mapping"), *Attribute::Value::from_utf8("1/2/3"))
        .value();
    attribute_specifier
        .define(*Attribute::Key::from_utf8("camera_resolution"), *Attribute::Value::from_utf8("1920x1080"))
        .value();

    auto service = node.service_builder(ServiceName::create("Service/With/Properties").value())
                       .publish_subscribe<uint64_t>()
                       .create_with_attributes(attribute_specifier)
                       .value();

    auto publisher = service.publisher_builder().create().value();

    std::cout << "defined service attributes: " << service.attributes() << std::endl;

    while (node.wait(CYCLE_TIME).has_value()) {
        auto sample = publisher.loan().value();
        sample.payload_mut() = 0;
        send(std::move(sample)).value();
    }

    std::cout << "exit" << std::endl;

    return 0;
}
