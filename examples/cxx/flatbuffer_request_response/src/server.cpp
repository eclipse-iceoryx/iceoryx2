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

#include <cstdint>
#include <flatbuffers/flatbuffers.h>

#include "data_props_generated.h"
#include "iox2/iceoryx2.hpp"
#include "iox2/marker.hpp"
#include "unbounded_data_generated.h"

// Explicitly sets the type name of our generated type so that the auto path-lookup
// works.
IOX2_DEFINE_TYPE_NAME(Example::UnboundedData, "UnboundedData");
IOX2_DEFINE_TYPE_NAME(Example::DataProps, "DataProps");

constexpr iox2::bb::Duration CYCLE_TIME = iox2::bb::Duration::from_millis(100);

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

    auto server = service
                      .server_builder()
                      // We start with 1024 bytes. The more accurate the initial_reserved_memory
                      // estimate is, the fewer reallocations will be required. Reallocations occur
                      // only at the beginning of communication. Once the server's data segment
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

    std::cout << "Server ready to receive requests!" << std::endl;

    auto response_counter = 0;

    while (node.wait(CYCLE_TIME).has_value()) {
        auto active_request = server.receive().value();
        if (active_request.has_value()) {
            const auto* data = active_request->payload_root();

            std::cout << "title: " << data->title()->c_str() << std::endl;
            std::cout << "user header: " << active_request->user_header() << std::endl;

            const auto* entries = data->entries();
            for (uint32_t i = 0; i < entries->size(); ++i) {
                response_counter += 1;
                auto data1 = entries->Get(i)->data_1();
                auto data2 = entries->Get(i)->data_2();
                auto sum = data1 + data2;

                std::cout << "  Send response: " << data1 << "+" << data2 << " = " << sum << " to Entry " << i
                          << ": data_1=" << data1 << ", data_2=" << data2 << std::endl;
            }
            std::cout << std::endl;
        }
    }

    std::cout << "exit" << std::endl;

    return 0;
}
