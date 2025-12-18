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

#include "complex_type.hpp"
#include "iox2/iceoryx2.hpp"

#include <cstdint>
#include <iostream>
#include <utility>

constexpr iox2::bb::Duration CYCLE_TIME = iox2::bb::Duration::from_secs(1);

auto main() -> int {
    using namespace iox2;
    set_log_level_from_env_or(LogLevel::Info);
    auto node = NodeBuilder().create<ServiceType::Ipc>().value();

    auto service = node.service_builder(ServiceName::create("CrossLanguageComplexTypes").value())
                       .publish_subscribe<ComplexType>()
                       .open_or_create()
                       .value();

    auto publisher = service.publisher_builder().create().value();

    auto counter = 0;
    while (node.wait(CYCLE_TIME).has_value()) {
        counter += 1;

        auto sample = publisher.loan_uninit().value();
        new (&sample.payload_mut()) ComplexType {};

        auto& payload = sample.payload_mut();
        payload.address_book.try_emplace_back(FullName {
            *bb::StaticString<256>::from_utf8("Lisa"),    // NOLINT
            *bb::StaticString<256>::from_utf8("The Log"), // NOLINT
        });
        payload.some_matrix.try_insert_at(0, 8, bb::StaticVector<double, 8>()); //NOLINT
        for (uint64_t idx = 0; idx < payload.some_matrix.size(); ++idx) {
            payload.some_matrix.unchecked_access()[idx].try_insert_at(0, 8, 0.0); //NOLINT
        }
        payload.some_matrix.unchecked_access()[2].unchecked_access()[5] = counter * 1.2123; //NOLINT

        auto initialized_sample = assume_init(std::move(sample));
        send(std::move(initialized_sample)).value();

        std::cout << "Send sample " << counter << "..." << std::endl;
    }

    std::cout << "exit" << std::endl;

    return 0;
}
