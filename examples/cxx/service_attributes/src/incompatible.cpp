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

auto main() -> int {
    using namespace iox2;
    set_log_level_from_env_or(LogLevel::Info);
    auto node = NodeBuilder().create<ServiceType::Ipc>().expect("successful node creation");

    auto incompatible_service =
        node.service_builder(ServiceName::create("Service/With/Properties").expect("valid service name"))
            .publish_subscribe<uint64_t>()
            .open_with_attributes(
                // the opening of the service will fail since the
                // `camera_resolution` attribute is `1920x1080` and not
                // `3840x2160`
                AttributeVerifier().require("camera_resolution", "3840x2160"));
    if (incompatible_service.has_error()) {
        std::cout << "camera_resolution: 3840x2160 -> not available" << std::endl;
    }

    incompatible_service = node.service_builder(ServiceName::create("My/Funk/ServiceName").expect("valid service name"))
                               .publish_subscribe<uint64_t>()
                               .open_with_attributes(
                                   // the opening of the service will fail since the key is not
                                   // defined.
                                   AttributeVerifier().require_key("camera_type"));
    if (incompatible_service.has_error()) {
        std::cout << "camera_type -> not available" << std::endl;
    }

    return 0;
}
