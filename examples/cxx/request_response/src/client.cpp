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

constexpr iox::units::Duration CYCLE_TIME = iox::units::Duration::fromSeconds(1);

auto main() -> int {
    using namespace iox2;
    set_log_level_from_env_or(LogLevel::Info);
    auto node = NodeBuilder().create<ServiceType::Ipc>().expect("successful node creation");

    auto service = node.service_builder(ServiceName::create("My/Funk/ServiceName").expect("valid service name"))
                       .request_response<uint64_t, TransmissionData>()
                       .open_or_create()
                       .expect("successful service creation/opening");

    auto client = service.client_builder().create().expect("successful client creation");

    auto request_counter = 0;
    auto response_counter = 0;

    // sending first request by using slower, inefficient copy API
    std::cout << "send request " << request_counter << " ..." << std::endl;
    auto pending_response = client.send_copy(request_counter).expect("send successful");

    while (node.wait(CYCLE_TIME).has_value()) {
        // acquire all responses to our request from our buffer that were sent by the servers
        while (true) {
            auto response = pending_response.receive().expect("receive successful");
            if (response.has_value()) {
                std::cout << "received response " << response_counter << ": " << response->payload() << std::endl;
                response_counter += 1;
            } else {
                break;
            }
        }

        request_counter += 1;
        // send all other requests by using zero copy API
        auto request = client.loan_uninit().expect("loan successful");
        auto initialized_request = request.write_payload(request_counter);

        pending_response = send(std::move(initialized_request)).expect("send successful");

        std::cout << "send request " << request_counter << " ..." << std::endl;
    }

    std::cout << "exit" << std::endl;

    return 0;
}
