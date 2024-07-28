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

#include "iox2/callback_progression.hpp"
#include "iox2/config.hpp"
#include "iox2/service.hpp"
#include "iox2/service_type.hpp"

#include <iostream>

auto main() -> int {
    using namespace iox2;

    Service<ServiceType::Ipc>::list(Config::global_config(), [](auto service) {
        std::cout << service.static_details.name() << std::endl;
        return CallbackProgression::Continue;
    }).expect("discover all available services");

    return 0;
}
