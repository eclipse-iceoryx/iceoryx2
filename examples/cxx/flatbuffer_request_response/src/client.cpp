// Copyright (c) 2026 Contributors to the Eclipse Foundation
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

#include <chrono>
#include <cstdint>
#include <flatbuffers/flatbuffers.h>
#include <thread>

#include "data_props_generated.h"
#include "iox2/iceoryx2.hpp"
#include "unbounded_data_generated.h"

// Explicitly sets the type name of our generated type so that the auto path-lookup
// works.
IOX2_DEFINE_TYPE_NAME(Example::UnboundedData, "UnboundedData");
IOX2_DEFINE_TYPE_NAME(Example::DataProps, "DataProps");

constexpr iox2::bb::Duration CYCLE_TIME = iox2::bb::Duration::from_secs(1);

auto main() -> int {
    using namespace iox2;
    using namespace Example;
    set_log_level_from_env_or(LogLevel::Info);

    // export IOX2_FLATBUFFER_SCHEMA_PATH=${pwd}/examples/cxx/flatbuffer_request_response/src
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

    auto service = node.service_builder(ServiceName::create("Flatbuffer/Request/Response").value())
                       .request_response<Flatbuffer<UnboundedData>, Flatbuffer<DataProps>>()
                       .request_user_header<uint64_t>()
                       .response_user_header<uint64_t>()
                       .open_or_create()
                       .value();

    auto client = service
                      .client_builder()
                      // We start with 1024 bytes. The more accurate the initial_reserved_memory
                      // estimate is, the fewer reallocations will be required. Reallocations occur
                      // only at the beginning of communication. Once the client's data segment
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

    auto request_counter = 0U;
    while (node.wait(CYCLE_TIME).has_value()) {
        request_counter += 1;
        auto request = client.loan_flatbuffer().value();
        auto& builder = request.flatbuffer_builder();

        // BEGIN: standard flatbuffer API
        auto title = builder.CreateString("Hello World!");

        std::vector<flatbuffers::Offset<Entry>> entries;
        for (uint64_t i = 0; i < (request_counter % 3); ++i) {                                      // NOLINT
            entries.emplace_back(CreateEntry(builder, static_cast<int32_t>(6 * i + 5), 6 * i + 7)); // NOLINT
        }

        auto entry_vec = builder.CreateVector(entries);
        auto unbounded_data = CreateUnboundedData(builder, title, entry_vec);
        // END: standard flatbuffer API

        // calls builder.Finish(root, nullptr) and sets the payload offset
        auto initialized_request = assume_init(std::move(request), unbounded_data);
        initialized_request.user_header_mut() = request_counter;

        std::cout << "send request " << request_counter << " ..." << std::endl;
        auto pending_response = send(std::move(initialized_request)).value();

        // we want to receive all responses so we wait here until there are no
        // more responses and the other side has dropped the active request object
        // and therefore dropped the connection
        while (pending_response.has_response() || pending_response.is_connected()) {
            auto response = pending_response.receive().value();
            if (response.has_value()) {
                std::cout << "  Receive response: " << response->payload_root()->received_entries_len() << std::endl;
            } else {
                std::this_thread::sleep_for(std::chrono::milliseconds(25)); // NOLINT
            }
        }
    }

    std::cout << "exit" << std::endl;

    return 0;
}
