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

// Explicitly sets the type name of our generated type so that the auto path-lookup
// works.
IOX2_DEFINE_TYPE_NAME(Example::UnboundedData, "UnboundedData");

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
                       // This method allows us to use a custom schema file path when no schema lookup path was
                       // defined or when a custom file is required (maybe outside of the lookup path).
                       // IOX2_DEFINE_TYPE_NAME must be called for the generated payload type.
                       // .flatbuffer_schema_path(bb::FilePath::create("unbounded_data.fbs").value())
                       .user_header<uint64_t>()
                       .open_or_create()
                       .value();

    auto publisher = service
                         .publisher_builder()
                         // We start with 1024 bytes. The more accurate the initial_reserved_memory
                         // estimate is, the fewer reallocations will be required. Reallocations occur
                         // only at the beginning of communication. Once the publisher's data segment
                         // has been resized appropriately, all subsequent samples will use that size.
                         .initial_reserved_memory(1024) // NOLINT
                         // By default, the allocation strategy is Static, which does not allow
                         // reallocations when initial_reserved_memory is exhausted. Set it to
                         // PowerOfTwo or BestFit to enable reallocations.
                         //
                         // The maximum number of reallocations is 256. BestFit allocates only the
                         // explicitly requested amount of memory, so this limit can be reached
                         // quickly. Increasing initial_reserved_memory reduces the number of
                         // reallocations.
                         .allocation_strategy(AllocationStrategy::PowerOfTwo)
                         .create()
                         .value();

    uint64_t counter = 0;
    while (node.wait(CYCLE_TIME).has_value()) {
        counter += 1;
        auto sample = publisher.loan_flatbuffer().value();
        auto& builder = sample.flatbuffer_builder();

        // BEGIN: standard flatbuffer API
        auto title = builder.CreateString("Hello World!");

        std::vector<flatbuffers::Offset<Entry>> entries;
        for (uint64_t i = 0; i < (counter % 15); ++i) {                                             // NOLINT
            entries.emplace_back(CreateEntry(builder, static_cast<int32_t>(6 * i + 5), 6 * i + 7)); // NOLINT
        }

        auto entry_vec = builder.CreateVector(entries);
        auto unbounded_data = CreateUnboundedData(builder, title, entry_vec);
        // END: standard flatbuffer API

        // calls builder.Finish(root, nullptr) and sets the payload offset
        auto initialized_sample = assume_init(std::move(sample), unbounded_data);
        initialized_sample.user_header_mut() = counter;

        send(std::move(initialized_sample)).has_value();
        std::cout << "Send sample " << counter << "..." << std::endl;
    }

    std::cout << "exit" << std::endl;

    return 0;
}
