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

#include "iox2/iceoryx2.hpp"
#include "request_header.hpp"
#include "transmission_data.hpp"

#include <algorithm>
#include <iterator>

constexpr iox2::bb::Duration TIMEOUT = iox2::bb::Duration::from_secs(2);

auto main() -> int {
    using namespace iox2;
    set_log_level_from_env_or(LogLevel::Info);
    auto node = NodeBuilder().create<ServiceType::Ipc>().value();

    auto server_event_service =
        node.service_builder(ServiceName::create("example/request_response_with_events/cxx_python/server").value())
            .event()
            .open_or_create()
            .value();
    auto server_listener = server_event_service.listener_builder().create().value();

    auto client_event_service =
        node.service_builder(ServiceName::create("example/request_response_with_events/cxx_python/client").value())
            .event()
            .open_or_create()
            .value();
    auto client_notifier = client_event_service.notifier_builder().create().value();

    auto service = node.service_builder(ServiceName::create("example/request_response_with_events/cxx_python").value())
                       .request_response<uint64_t, TransmissionData>()
                       .request_user_header<RequestResponseWithEventsHeader>()
                       .open_or_create()
                       .value();
    auto server = service.server_builder().create().value();

    std::cout << "Server ready to receive requests!" << std::endl;
    auto counter = 0;

    while (node.wait(bb::Duration::from_secs(0)).has_value()) {
        auto wait_result = server_listener.timed_wait([](auto) -> auto { }, TIMEOUT);
        if (!wait_result.has_value() && wait_result.error() == ListenerWaitError::InterruptSignal) {
            break;
        }
        if (wait_result.value() == 0) {
            std::cout << "Timeout while waiting for clients" << std::endl;
            continue;
        }

        while (true) {
            auto active_request = server.receive().value();
            if (!active_request.has_value()) {
                break;
            }
            std::cout << "received request: " << active_request->payload() << std::endl;
            auto response = TransmissionData { 5 + counter, 6 * counter, 7.77 }; // NOLINT
            std::cout << "  send response: " << response << std::endl;
            active_request->send_copy(response).value();

            // Notify after sending, while the request header is still alive.
            const auto& listener_id = active_request->user_header().listener_id;
            client_notifier.for_each_listener([&](auto monofier, auto details) -> auto {
                auto candidate_listener_id = details.listener_id();
                const auto& id_bytes = candidate_listener_id.bytes().value();
                if (std::equal(id_bytes.unchecked_access().begin(),
                               id_bytes.unchecked_access().end(),
                               std::begin(listener_id))) {
                    monofier.notify().value();
                    return CallbackProgression::Stop;
                }
                return CallbackProgression::Continue;
            });
        }
        counter += 1;
    }

    std::cout << "exit" << std::endl;
    return 0;
}
