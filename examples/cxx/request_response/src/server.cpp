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

#include "iox2/iceoryx2.hpp"
#include "transmission_data.hpp"

constexpr iox::units::Duration CYCLE_TIME = iox::units::Duration::fromMilliseconds(100);

auto main() -> int {
    using namespace iox2;
    set_log_level_from_env_or(LogLevel::Info);
    auto node = NodeBuilder().create<ServiceType::Ipc>().expect("successful node creation");

    auto service = node.service_builder(ServiceName::create("My/Funk/ServiceName").expect("valid service name"))
                       .request_response<uint64_t, TransmissionData>()
                       .open_or_create()
                       .expect("successful service creation/opening");

    auto server = service.server_builder().create().expect("successful server creation");

    std::cout << "Server ready to receive requests!" << std::endl;

    auto counter = 0;

    while (node.wait(CYCLE_TIME).has_value()) {
        while (true) {
            auto active_request = server.receive().expect("receive successful");
            if (active_request.has_value()) {
                std::cout << "received request: " << active_request->payload() << std::endl;

                auto response = TransmissionData { 5 + counter, 6 * counter, 7.77 }; // NOLINT
                std::cout << "send response: " << response << std::endl;
                // send first response by using the slower, non-zero-copy API
                active_request->send_copy(response).expect("send successful");

                // use zero copy API, send out some responses to demonstrate the streaming API
                for (auto iter = 0; iter < static_cast<int32_t>(active_request->payload()) % 2; iter++) {
                    auto response = active_request->loan_uninit().expect("loan successful");
                    auto initialized_response = response.write_payload(
                        TransmissionData { counter * (iter + 1), counter + iter, counter * 0.1234 }); // NOLINT
                    std::cout << "send response: " << *initialized_response << std::endl;
                    send(std::move(initialized_response)).expect("send successful");
                }
            } else {
                break;
            }
            // when an active_request goes out of scope it marks the connection so
            // that the corresponding pending response sees that no more
            // responses are arriving
        }

        counter += 1;
    }

    std::cout << "exit" << std::endl;

    return 0;
}
