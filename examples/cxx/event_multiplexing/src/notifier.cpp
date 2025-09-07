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

#include <iostream>

#include "iox/cli_definition.hpp"
#include "iox2/iceoryx2.hpp"

// NOLINTBEGIN
struct Args {
    IOX_CLI_DEFINITION(Args);
    IOX_CLI_OPTIONAL(iox::string<64>, service, { "fuu" }, 's', "service", "The name of the service.");
    IOX_CLI_OPTIONAL(uint64_t, event_id, 0, 'e', "event-id", "The event id that shall be used to trigger the service.");
};
// NOLINTEND

constexpr iox::units::Duration CYCLE_TIME = iox::units::Duration::fromSeconds(1);

auto main(int argc, char** argv) -> int {
    using namespace iox2;
    set_log_level_from_env_or(LogLevel::Info);
    auto args = Args::parse(argc, argv, "Notifier of the event multiplexing example.");

    auto event_id = EventId(args.event_id());
    auto service_name = ServiceName::create(args.service().c_str()).expect("valid service name");

    auto node = NodeBuilder().create<ServiceType::Ipc>().expect("successful node creation");

    auto service =
        node.service_builder(service_name).event().open_or_create().expect("successful service creation/opening");

    auto notifier = service.notifier_builder().create().expect("successful notifier creation");

    while (node.wait(CYCLE_TIME).has_value()) {
        notifier.notify_with_custom_event_id(event_id).expect("notification");

        std::cout << "[service: \"" << service_name.to_string().c_str() << "\"] Trigger event with id " << event_id
                  << "..." << std::endl;
    }

    std::cout << "exit" << std::endl;

    return 0;
}
