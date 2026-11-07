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
#include <cstdint>
#include <iterator>
#include <utility>

constexpr iox2::bb::Duration CYCLE_TIME = iox2::bb::Duration::from_secs(1);
constexpr iox2::bb::Duration TIMEOUT = iox2::bb::Duration::from_millis(100);

auto main() -> int {
    using namespace iox2;
    set_log_level_from_env_or(LogLevel::Info);
    auto node = NodeBuilder().create<ServiceType::Ipc>().value();

    auto server_event_service =
        node.service_builder(ServiceName::create("example/request_response_with_events/cxx_python/server").value())
            .event()
            .open_or_create()
            .value();
    auto server_notifier = server_event_service.notifier_builder().create().value();

    auto client_event_service =
        node.service_builder(ServiceName::create("example/request_response_with_events/cxx_python/client").value())
            .event()
            .open_or_create()
            .value();
    auto client_listener = client_event_service.listener_builder().create().value();
    auto listener_id = client_listener.id();

    auto service = node.service_builder(ServiceName::create("example/request_response_with_events/cxx_python").value())
                       .request_response<uint64_t, TransmissionData>()
                       .request_user_header<RequestResponseWithEventsHeader>()
                       .open_or_create()
                       .value();
    auto client = service.client_builder().create().value();

    auto request_counter = uint64_t { 0 };
    auto response_counter = uint64_t { 0 };
    std::cout << "Client ready to send requests!" << std::endl;

    while (node.wait(CYCLE_TIME).has_value()) {
        std::cout << "send request " << request_counter << " ..." << std::endl;
        // Preserve the counter and pass an explicit rvalue, including on MSVC.
        auto request_payload = request_counter;
        // NOLINTNEXTLINE(hicpp-move-const-arg,performance-move-const-arg) write_payload requires an rvalue.
        auto request = client.loan_uninit().value().write_payload(std::move(request_payload));
        const auto& id_bytes = listener_id.bytes().value();
        std::copy(id_bytes.unchecked_access().begin(),
                  id_bytes.unchecked_access().end(),
                  std::begin(request.user_header_mut().listener_id));
        // Keep the pending response alive until the notification and responses arrive.
        auto pending_response = send(std::move(request)).value();
        server_notifier.notify().value();

        auto wait_result = client_listener.timed_wait([](auto) -> auto { }, TIMEOUT);
        if (!wait_result.has_value() && wait_result.error() == ListenerWaitError::InterruptSignal) {
            break;
        }
        auto events_received = wait_result.value();
        if (events_received == 0) {
            std::cout << "Timeout while waiting for response from server" << std::endl;
            continue;
        }
        std::cout << "  number of received notifications from server: " << events_received << std::endl;

        while (true) {
            auto response = pending_response.receive().value();
            if (!response.has_value()) {
                break;
            }
            std::cout << "  received response " << response_counter << " for request " << request_counter << ": "
                      << response->payload() << std::endl;
            response_counter += 1;
        }
        request_counter += 1;
    }

    std::cout << "exit" << std::endl;
    return 0;
}
