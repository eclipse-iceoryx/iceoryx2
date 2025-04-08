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

#include "iox2/attribute_verifier.hpp"
#include "iox2/log.hpp"
#include "iox2/node.hpp"
#include "iox2/service_name.hpp"
#include "iox2/service_type.hpp"

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

    incompatible_service = node.service_builder(ServiceName::create("My/Funk/ServiceName").expect("valid service name"))
                               .publish_subscribe<uint64_t>()
                               .open_with_attributes(
                                   // the opening of the service will fail since the key is not
                                   // defined.
                                   AttributeVerifier().require_key("camera_type"));

    return 0;
}
