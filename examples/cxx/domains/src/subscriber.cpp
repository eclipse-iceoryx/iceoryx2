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
#include "transmission_data.hpp"

#include <iostream>

constexpr iox2::bb::Duration CYCLE_TIME = iox2::bb::Duration::from_secs(1);

auto main(int argc, char** argv) -> int {
    using namespace iox2;
    set_log_level_from_env_or(LogLevel::Info);

    check_for_help_from_args(argc, argv, []() -> auto {
        std::cout << "Subscriber of the domain example." << std::endl;
        std::cout << std::endl;
        std::cout << "Use '-d' or '--domain' to specify the name of the domain." << std::endl;
        std::cout << "Use '-s' or '--service' to specify the name of the service." << std::endl;
    });

    // NOLINTNEXTLINE(cppcoreguidelines-avoid-magic-numbers,readability-magic-numbers) fine for the example
    const CliOption<32> option_domain { "-d",
                                        "--domain",
                                        iox2::bb::StaticString<32>::from_utf8_unchecked("iox2_"),
                                        "Invalid parameter! The domain must be passed after '-d' or '--domain'" };
    // NOLINTNEXTLINE(cppcoreguidelines-avoid-magic-numbers,readability-magic-numbers) fine for the example
    const CliOption<256> option_service { "-s",
                                          "--service",
                                          iox2::bb::StaticString<256>::from_utf8_unchecked("my_funky_service"),
                                          "Invalid parameter! The service must be passed after '-s' or '--service'" };

    auto domain = parse_from_args(argc, argv, option_domain);
    auto service_name = parse_from_args(argc, argv, option_service);

    // create a new config based on the global config
    auto config = Config::global_config().to_owned();

    // The domain name becomes the prefix for all resources.
    // Therefore, different domain names never share the same resources.
    config.global().set_prefix(iox2::bb::FileName::create(domain).value());

    auto node = NodeBuilder()
                    // use the custom config when creating the custom node
                    // every service constructed by the node will use this config
                    .config(config)
                    .create<ServiceType::Ipc>()
                    .value();

    auto service = node.service_builder(ServiceName::create(service_name.unchecked_access().c_str()).value())
                       .publish_subscribe<TransmissionData>()
                       .open_or_create()
                       .value();

    auto subscriber = service.subscriber_builder().create().value();

    std::cout << "subscribed to: [domain: \"" << domain.unchecked_access().c_str() << "\", service: \""
              << service_name.unchecked_access().c_str() << "\"]" << std::endl;
    while (node.wait(CYCLE_TIME).has_value()) {
        auto sample = subscriber.receive().value();
        while (sample.has_value()) {
            std::cout << "received: " << sample->payload() << std::endl;
            sample = subscriber.receive().value();
        }
    }

    std::cout << "exit" << std::endl;

    return 0;
}
