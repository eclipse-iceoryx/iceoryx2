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
#include "iox2/entry_handle_mut.hpp"
#include "iox2/entry_value.hpp"
#include "iox2/log.hpp"
#include "iox2/node.hpp"
#include "iox2/service_name.hpp"
#include "iox2/service_type.hpp"

#include <iostream>
#include <utility>

constexpr iox::units::Duration CYCLE_TIME = iox::units::Duration::fromSeconds(1);

auto main() -> int {
    using namespace iox2;
    set_log_level_from_env_or(LogLevel::Info);
    auto node = NodeBuilder().create<ServiceType::Ipc>().expect("successful node creation");

    const double start_value = 1.1;
    auto service = node.service_builder(ServiceName::create("My/Funk/ServiceName").expect("valid service name"))
                       .blackboard_creator<uint64_t>()
                       .template add_with_default<uint64_t>(0)
                       .template add<double>(1, start_value)
                       .create()
                       .expect("successful service creation");
    std::cout << "Blackboard created." << std::endl;

    auto writer = service.writer_builder().create().expect("successful writer creation");
    auto entry_handle_mut_key_0 = writer.template entry<uint64_t>(0).expect("successful entry handle creation");
    auto entry_handle_mut_key_1 = writer.template entry<double>(1).expect("successful entry handle creation");

    auto counter = 0;
    while (node.wait(CYCLE_TIME).has_value()) {
        counter += 1;

        entry_handle_mut_key_0.update_with_copy(counter);
        std::cout << "Write new value for key 0: " << counter << "..." << std::endl;

        auto entry_value_uninit = loan_uninit(std::move(entry_handle_mut_key_1));
        auto value = start_value * counter;
        auto entry_value = write(std::move(entry_value_uninit), value);
        entry_handle_mut_key_1 = update(std::move(entry_value));
        std::cout << "Write new value for key 1: " << value << "...\n" << std::endl;
    }

    std::cout << "exit" << std::endl;

    return 0;
}
