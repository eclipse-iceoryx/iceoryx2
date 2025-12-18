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


#include "iox2/bb/static_string.hpp"
#include "iox2/iceoryx2.hpp"
#include "parse_args.hpp"

#include <iostream>
#include <sstream>

constexpr iox2::bb::Duration CYCLE_TIME = iox2::bb::Duration::from_secs(1);

auto main(int argc, char** argv) -> int {
    using namespace iox2;
    set_log_level_from_env_or(LogLevel::Info);

    check_for_help_from_args(argc, argv, []() -> auto {
        std::cout << "Notifier of the event multiplexing example." << std::endl;
        std::cout << std::endl;
        std::cout << "Use '-e' or '--event-id' to specify event ID that shall be used to trigger the service."
                  << std::endl;
        std::cout << "Use '-s' or '--service' to specify the name of the service." << std::endl;
    });

    // NOLINTNEXTLINE(cppcoreguidelines-avoid-magic-numbers,readability-magic-numbers) fine for the example
    const CliOption<256> option_service { "-s",
                                          "--service",
                                          iox2::bb::StaticString<256>::from_utf8_unchecked("fuu"),
                                          "Invalid parameter! The service must be passed after '-s' or '--service'" };

    // NOLINTNEXTLINE(cppcoreguidelines-avoid-magic-numbers,readability-magic-numbers) fine for the example
    const CliOption<32> option_event_id { "-e",
                                          "--event-id",
                                          iox2::bb::StaticString<32>::from_utf8_unchecked("0"),
                                          "Invalid parameter! The event-id must be passed after '-e' or '--event-id'" };

    auto service_name_arg = parse_from_args(argc, argv, option_service);
    auto event_id_string = parse_from_args(argc, argv, option_event_id);

    std::istringstream iss(std::string(event_id_string.unchecked_access().c_str()));
    uint64_t event_id_int { 0 };
    if (iss >> event_id_int) {
        // std::cout << "Converted number: " << num << std::endl;
    } else {
        std::cout << "Could not parse event ID: " << event_id_string << std::endl;
        exit(1);
    }

    auto event_id = EventId(event_id_int);

    auto service_name = ServiceName::create(service_name_arg.unchecked_access().c_str()).value();

    auto node = NodeBuilder().create<ServiceType::Ipc>().value();

    auto service = node.service_builder(service_name).event().open_or_create().value();

    auto notifier = service.notifier_builder().create().value();

    while (node.wait(CYCLE_TIME).has_value()) {
        notifier.notify_with_custom_event_id(event_id).value();

        std::cout << "[service: \"" << service_name.to_string().unchecked_access().c_str()
                  << "\"] Trigger event with id " << event_id << "..." << std::endl;
    }

    std::cout << "exit" << std::endl;

    return 0;
}
