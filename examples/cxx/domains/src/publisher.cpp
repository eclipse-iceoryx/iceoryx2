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

#include "iox2/container/static_string.hpp"
#include "iox2/iceoryx2.hpp"
#include "iox2/legacy/cli_definition.hpp"
#include "transmission_data.hpp"

#include <iostream>
#include <utility>

// NOLINTBEGIN
struct Args {
    IOX2_CLI_DEFINITION(Args);
    IOX2_CLI_OPTIONAL(iox2::legacy::string<32>,
                      domain,
                      { "iox2_" },
                      'd',
                      "domain",
                      "The name of the domain. Must be a valid file name.");
    IOX2_CLI_OPTIONAL(
        iox2::legacy::string<256>, service, { "my_funky_service" }, 's', "service", "The name of the service.");
    IOX2_CLI_SWITCH(debug, 'e', "debug", "Enable full debug log output");
};
// NOLINTEND

constexpr iox2::bb::Duration CYCLE_TIME = iox2::bb::Duration::from_secs(1);

auto main(int argc, char** argv) -> int {
    using namespace iox2;
    set_log_level_from_env_or(LogLevel::Info);
    auto args = Args::parse(argc, argv, "Publisher of the domain example.");

    // create a new config based on the global config
    auto config = Config::global_config().to_owned();

    // The domain name becomes the prefix for all resources.
    // Therefore, different domain names never share the same resources.
    // TODO: adapt Args
    auto domain = *container::StaticString<32>::from_utf8_null_terminated_unchecked(args.domain().c_str());
    config.global().set_prefix(iox2::bb::FileName::create(domain).expect("valid domain name"));

    auto node = NodeBuilder()
                    // use the custom config when creating the custom node
                    // every service constructed by the node will use this config
                    .config(config)
                    .create<ServiceType::Ipc>()
                    .value();

    auto service = node.service_builder(ServiceName::create(args.service().c_str()).value())
                       .publish_subscribe<TransmissionData>()
                       .open_or_create()
                       .value();

    auto publisher = service.publisher_builder().create().value();

    auto counter = 0;
    while (node.wait(CYCLE_TIME).has_value()) {
        counter += 1;

        auto sample = publisher.loan_uninit().value();

        auto initialized_sample =
            sample.write_payload(TransmissionData { counter, counter * 3, counter * 812.12 }); // NOLINT

        send(std::move(initialized_sample)).value();

        std::cout << "[domain: \"" << args.domain() << "\", service: \"" << args.service() << "] Send sample "
                  << counter << "..." << std::endl;
    }

    std::cout << "exit" << std::endl;

    return 0;
}
