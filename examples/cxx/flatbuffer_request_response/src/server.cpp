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

    auto server = service.server_builder().create().value();

    std::cout << "Server ready to receive requests!" << std::endl;

    auto counter = 0;

    while (node.wait(CYCLE_TIME).has_value()) {
        //    while (true) {
        //        auto active_request = server.receive().value();
        //        if (active_request.has_value()) {
        //            std::cout << "received request: " << active_request->payload() << std::endl;

        //            auto response = TransmissionData { 5 + counter, 6 * counter, 7.77 }; // NOLINT
        //            std::cout << "send response: " << response << std::endl;
        //            // send first response by using the slower, non-zero-copy API
        //            active_request->send_copy(response).value();

        //            // use zero copy API, send out some responses to demonstrate the streaming API
        //            for (auto iter = 0; iter < static_cast<int32_t>(active_request->payload()) % 2; iter++) {
        //                auto response = active_request->loan_uninit().value();
        //                auto initialized_response = response.write_payload(
        //                    TransmissionData { counter * (iter + 1), counter + iter, counter * 0.1234 }); // NOLINT
        //                std::cout << "send response: " << initialized_response.payload() << std::endl;
        //                send(std::move(initialized_response)).value();
        //            }
        //        } else {
        //            break;
        //        }
        //        // when an active_request goes out of scope it marks the connection so
        //        // that the corresponding pending response sees that no more
        //        // responses are arriving
        //    }

        counter += 1;
    }

    std::cout << "exit" << std::endl;

    return 0;
}
