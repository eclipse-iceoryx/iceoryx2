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
#include "iox2/callback_progression.hpp"
#include "iox2/config.hpp"
#include "iox2/log.hpp"
#include "iox2/service.hpp"
#include "iox2/service_type.hpp"

#include <iostream>

// NOLINTBEGIN
struct Args {
    IOX_CLI_DEFINITION(Args);
    IOX_CLI_OPTIONAL(
        iox::string<32>, domain, { "iox2_" }, 'd', "domain", "The name of the domain. Must be a valid file name.");
    IOX_CLI_SWITCH(debug, 'e', "debug", "Enable full debug log output");
};
// NOLINTEND

auto main(int argc, char** argv) -> int {
    using namespace iox2;
    set_log_level_from_env_or(LogLevel::Info);
    auto args = Args::parse(argc, argv, "Discovery of the domain example.");

    // create a new config based on the global config
    auto config = Config::global_config().to_owned();

    // The domain name becomes the prefix for all resources.
    // Therefore, different domain names never share the same resources.
    config.global().set_prefix(iox::FileName::create(args.domain()).expect("valid domain name"));

    Service<ServiceType::Ipc>::list(config.view(), [](auto service) {
        std::cout << service.static_details << std::endl;
        return CallbackProgression::Continue;
    }).expect("discover all available services");

    return 0;
}
