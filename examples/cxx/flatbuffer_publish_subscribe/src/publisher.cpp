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

#include <cstdint>
#include <cstdlib>
#include <flatbuffers/flatbuffers.h>
#include <iostream>
#include <utility>
#include <vector>

#include "iox2/iceoryx2.hpp"
#include "unbounded_data_generated.h"

constexpr iox2::bb::Duration CYCLE_TIME = iox2::bb::Duration::from_secs(1);

auto main() -> int {
    using namespace iox2;
    using namespace Example;

    set_log_level_from_env_or(LogLevel::Info);

    const auto* lookup_path = std::getenv("IOX2_FLATBUFFER_SCHEMA_PATH");
    if (lookup_path == nullptr) {
        std::cout << "Please define IOX2_FLATBUFFER_SCHEMA_PATH!" << std::endl;
        return -1;
    }

    auto config = Config();
    config.global().service().set_flatbuffer_schema_path(bb::Path::create(lookup_path).value());

    auto node = NodeBuilder()
                    // Use the config with the defined flatbuffer schema path to enable automatic flatbuffer
                    // schema file lookup.
                    .config(config)
                    .create<ServiceType::Ipc>()
                    .value();

    auto service = node.service_builder(ServiceName::create("My/Flatbuffer/Service").value())
                       .publish_subscribe<Flatbuffer<UnboundedData>>()
                       .flatbuffer_schema_path(bb::FilePath::create("unbounded_data.fbs").value())
                       .user_header<uint64_t>()
                       .open_or_create()
                       .value();

    auto publisher = service.publisher_builder()
                         .initial_reserved_memory(32)
                         .allocation_strategy(AllocationStrategy::PowerOfTwo)
                         .create()
                         .value();

    uint64_t counter = 0;
    while (node.wait(CYCLE_TIME).has_value()) {
        counter += 1;

        auto sample = publisher.loan_flatbuffer().value();
        auto& builder = sample.flatbuffer_builder();

        auto title = builder.CreateString("Hello World!");

        std::vector<flatbuffers::Offset<Entry>> entries;
        for (uint64_t i = 0; i < (counter % 15); ++i) {                       // NOLINT
            entries.emplace_back(CreateEntry(builder, 6 * i + 5, 6 * i + 7)); // NOLINT
        }

        auto entry_vec = builder.CreateVector(entries);

        auto unbounded_data = CreateUnboundedData(builder, title, entry_vec);
        auto initialized_sample = assume_init(std::move(sample), unbounded_data);

        initialized_sample.user_header_mut() = counter;

        send(std::move(initialized_sample)).has_value();

        std::cout << "Send sample " << counter << "..." << std::endl;
    }

    std::cout << "exit" << std::endl;

    return 0;
}
