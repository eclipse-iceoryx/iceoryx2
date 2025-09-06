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

#include "iox/cli_definition.hpp"
#include "iox2/iceoryx2.hpp"
#include "transmission_data.hpp"

#include <iostream>
#include <utility>

// NOLINTBEGIN
struct Args {
    IOX_CLI_DEFINITION(Args);
    IOX_CLI_OPTIONAL(
        iox::string<32>, domain, { "iox2_" }, 'd', "domain", "The name of the domain. Must be a valid file name.");
    IOX_CLI_OPTIONAL(iox::string<256>, service, { "my_funky_service" }, 's', "service", "The name of the service.");
    IOX_CLI_SWITCH(debug, 'e', "debug", "Enable full debug log output");
};
// NOLINTEND

constexpr iox::units::Duration CYCLE_TIME = iox::units::Duration::fromSeconds(1);

auto main(int argc, char** argv) -> int {
    using namespace iox2;
    set_log_level_from_env_or(LogLevel::Info);
    auto args = Args::parse(argc, argv, "Publisher of the domain example.");

    // create a new config based on the global config
    auto config = Config::global_config().to_owned();

    // The domain name becomes the prefix for all resources.
    // Therefore, different domain names never share the same resources.
    config.global().set_prefix(iox::FileName::create(args.domain()).expect("valid domain name"));

    auto node = NodeBuilder()
                    // use the custom config when creating the custom node
                    // every service constructed by the node will use this config
                    .config(config)
                    .create<ServiceType::Ipc>()
                    .expect("successful node creation");

    auto service = node.service_builder(ServiceName::create(args.service().c_str()).expect("valid service name"))
                       .publish_subscribe<TransmissionData>()
                       .open_or_create()
                       .expect("successful service creation/opening");

    auto publisher = service.publisher_builder().create().expect("successful publisher creation");

    auto counter = 0;
    while (node.wait(CYCLE_TIME).has_value()) {
        counter += 1;

        auto sample = publisher.loan_uninit().expect("acquire sample");

        auto initialized_sample =
            sample.write_payload(TransmissionData { counter, counter * 3, counter * 812.12 }); // NOLINT

        send(std::move(initialized_sample)).expect("send successful");

        std::cout << "[domain: \"" << args.domain() << "\", service: \"" << args.service() << "] Send sample "
                  << counter << "..." << std::endl;
    }

    std::cout << "exit" << std::endl;

    return 0;
}
