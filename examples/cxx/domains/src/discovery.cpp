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

auto main(int argc, char** argv) -> int {
    using namespace iox2;
    set_log_level_from_env_or(LogLevel::Info);

    check_for_help_from_args(argc, argv, []() -> auto {
        std::cout << "Discovery of the domain example." << std::endl;
        std::cout << std::endl;
        std::cout << "Use '-d' or '--domain' to specify the name of the domain." << std::endl;
    });

    // NOLINTNEXTLINE(cppcoreguidelines-avoid-magic-numbers,readability-magic-numbers) fine for the example
    const CliOption<32> option_domain { "-d",
                                        "--domain",
                                        iox2::bb::StaticString<32>::from_utf8_unchecked("iox2_"),
                                        "Invalid parameter! The domain must be passed after '-d' or '--domain'" };

    auto domain = parse_from_args(argc, argv, option_domain);

    // create a new config based on the global config
    auto config = Config::global_config().to_owned();

    // The domain name becomes the prefix for all resources.
    // Therefore, different domain names never share the same resources.
    config.global().set_prefix(iox2::bb::FileName::create(domain).value());


    std::cout << "Services running in domain \"" << domain.unchecked_access().c_str() << "\":" << std::endl;

    Service<ServiceType::Ipc>::list(config.view(), [](auto service) -> auto {
        std::cout << service.static_details << std::endl;
        return CallbackProgression::Continue;
    }).value();

    return 0;
}
