// Copyright (c) 2025 Contributors to the Eclipse Foundation
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
#include "iox2/log.hpp"
#include "iox2/node.hpp"
#include "iox2/service_name.hpp"
#include "iox2/service_type.hpp"

#include <iostream>

constexpr iox::units::Duration CYCLE_TIME = iox::units::Duration::fromSeconds(1);

auto main() -> int {
    using namespace iox2;
    set_log_level_from_env_or(LogLevel::Info);
    auto node = NodeBuilder().create<ServiceType::Ipc>().expect("successful node creation");

    auto service = node.service_builder(ServiceName::create("My/Funk/ServiceName").expect("valid service name"))
                       .blackboard_opener<uint64_t>()
                       .open()
                       .expect("successful service opening");

    auto reader = service.reader_builder().create().expect("successful reader creation");
    auto entry_handle_key_0 = reader.template entry<uint64_t>(0).expect("successful entry handle creation");
    auto entry_handle_key_1 = reader.template entry<double>(1).expect("successful entry handle creation");

    while (node.wait(CYCLE_TIME).has_value()) {
        std::cout << "Read value " << entry_handle_key_0.get() << " for key 0..." << std::endl;
        std::cout << "Read value " << entry_handle_key_1.get() << " for key 1...\n" << std::endl;
    }

    std::cout << "exit" << std::endl;

    return 0;
}
