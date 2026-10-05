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

constexpr iox2::bb::Duration TIMEOUT = iox2::bb::Duration::from_secs(2);

auto main() -> int {
    using namespace iox2;
    set_log_level_from_env_or(LogLevel::Info);
    auto node = NodeBuilder().create<ServiceType::Ipc>().value();
    auto service =
        node.service_builder(ServiceName::create("example/request_response_with_events/cxx_python/client").value())
            .event()
            .open_or_create()
            .value();
    auto listener = service.listener_builder().create().value();
    std::cout << "Idle listener ready!" << std::endl;

    while (node.wait(bb::Duration::from_secs(0)).has_value()) {
        auto wait_result = listener.timed_wait([](auto) -> auto { }, TIMEOUT);
        if (!wait_result.has_value() && wait_result.error() == ListenerWaitError::InterruptSignal) {
            break;
        }
        if (wait_result.value() != 0) {
            std::cout << "Unexpected notification for idle listener" << std::endl;
            return 1;
        }
    }

    std::cout << "exit" << std::endl;
    return 0;
}
