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

#include "iox2/bb/static_string.hpp"
#include "iox2/bb/static_vector.hpp"
#include "iox2/iceoryx2.hpp"

#include <cstdint>
#include <iostream>
#include <utility>

constexpr iox2::bb::Duration CYCLE_TIME = iox2::bb::Duration::from_secs(1);

auto main() -> int {
    using namespace iox2;
    set_log_level_from_env_or(LogLevel::Info);
    auto node = NodeBuilder().create<ServiceType::Ipc>().value();

    auto service = node.service_builder(ServiceName::create("CrossLanguageContainer").value())
                       .publish_subscribe<iox2::bb::StaticVector<uint64_t, 32>>() // NOLINT
                       .user_header<iox2::bb::StaticString<64>>()                 // NOLINT
                       // add some QoS, disable safe overflow and the subscriber shall get the
                       // last 5 samples when connecting to the service
                       .history_size(5)               // NOLINT
                       .subscriber_max_buffer_size(5) // NOLINT
                       .enable_safe_overflow(false)
                       .open_or_create()
                       .value();

    auto publisher = service.publisher_builder().create().value();

    uint64_t counter = 0;
    while (node.wait(CYCLE_TIME).has_value()) {
        counter += 1;

        auto sample = publisher.loan_uninit().value();
        sample.user_header_mut() =
            *bb::StaticString<64>::from_utf8("Why are Kermit and Miss Piggy no longer together?"); // NOLINT

        auto initialized_sample =
            sample.write_payload(*bb::StaticVector<uint64_t, 32>::from_value(2, counter)); // NOLINT

        send(std::move(initialized_sample)).value();

        std::cout << "Send sample " << counter << "..." << std::endl;
    }

    std::cout << "exit" << std::endl;

    return 0;
}
