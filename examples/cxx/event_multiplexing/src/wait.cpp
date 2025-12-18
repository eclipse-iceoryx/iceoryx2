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

#include <iostream>
#include <map>

#include "iox2/bb/static_string.hpp"
#include "iox2/bb/static_vector.hpp"
#include "iox2/iceoryx2.hpp"
#include "parse_args.hpp"

struct ServiceNameListenerPair {
    iox2::ServiceName service_name;
    iox2::Listener<iox2::ServiceType::Ipc> listener;
};

auto main(int argc, char** argv) -> int {
    using namespace iox2;
    set_log_level_from_env_or(LogLevel::Info);

    check_for_help_from_args(argc, argv, []() -> auto {
        std::cout << "Notifier of the event multiplexing example." << std::endl;
        std::cout << std::endl;
        std::cout << "Use '-s' or '--service1' to specify the name of the service 1." << std::endl;
        std::cout << "Use '-t' or '--service2' to specify the name of the service 2." << std::endl;
    });

    // NOLINTNEXTLINE(cppcoreguidelines-avoid-magic-numbers,readability-magic-numbers) fine for the example
    const CliOption<256> option_service_1 {
        "-s",
        "--service1",
        iox2::bb::StaticString<256>::from_utf8_unchecked("fuu"),
        "Invalid parameter! The service must be passed after '-s' or '--service2'"
    };
    // NOLINTNEXTLINE(cppcoreguidelines-avoid-magic-numbers,readability-magic-numbers) fine for the example
    const CliOption<256> option_service_2 {
        "-t",
        "--service2",
        iox2::bb::StaticString<256>::from_utf8_unchecked("bar"),
        "Invalid parameter! The service must be passed after '-t' or '--service2'"
    };

    auto service_name_arg_1 = parse_from_args(argc, argv, option_service_1);
    auto service_name_arg_2 = parse_from_args(argc, argv, option_service_2);

    auto service_name_1 = ServiceName::create(service_name_arg_1.unchecked_access().c_str()).value();
    auto service_name_2 = ServiceName::create(service_name_arg_2.unchecked_access().c_str()).value();

    // create node and services
    auto node = NodeBuilder().create<ServiceType::Ipc>().value();

    auto service_1 = node.service_builder(service_name_1).event().open_or_create().value();
    auto service_2 = node.service_builder(service_name_2).event().open_or_create().value();
    auto listener_1 = service_1.listener_builder().create().value();
    auto listener_2 = service_2.listener_builder().create().value();

    // create the waitset and attach the listeners to it
    auto waitset = WaitSetBuilder().create<ServiceType::Ipc>().value();
    // NOLINTNEXTLINE(misc-const-correctness) false positive
    iox2::bb::StaticVector<WaitSetGuard<ServiceType::Ipc>, 2> guards;

    guards.try_emplace_back(waitset.attach_notification(listener_1).value());
    guards.try_emplace_back(waitset.attach_notification(listener_2).value());

    // NOLINTNEXTLINE(misc-const-correctness) false positive
    std::map<WaitSetAttachmentId<ServiceType::Ipc>, ServiceNameListenerPair> listeners;

    listeners.emplace(WaitSetAttachmentId<ServiceType::Ipc>::from_guard(guards.unchecked_access()[0]),
                      ServiceNameListenerPair { service_name_1, std::move(listener_1) });
    listeners.emplace(WaitSetAttachmentId<ServiceType::Ipc>::from_guard(guards.unchecked_access()[1]),
                      ServiceNameListenerPair { service_name_2, std::move(listener_2) });

    // the callback that is called when a listener has received an event
    auto on_event = [&](WaitSetAttachmentId<ServiceType::Ipc> attachment_id) -> auto {
        auto entry = listeners.find(attachment_id);
        if (entry != listeners.end()) {
            std::cout << "Received trigger from \"" << entry->second.service_name.to_string().unchecked_access().c_str()
                      << "\"" << std::endl;

            auto& listener = entry->second.listener;
            // IMPORTANT:
            // We need to collect all notifications since the WaitSet will wake us up as long as
            // there is something to read. If we skip this step completely we will end up in a
            // busy loop.
            listener.try_wait_all([](auto event_id) -> auto { std::cout << " " << event_id; }).value();
            std::cout << std::endl;
        }

        return iox2::CallbackProgression::Continue;
    };

    std::cout << "Waiting on the following services: " << service_name_1.to_string().unchecked_access().c_str() << ", "
              << service_name_2.to_string().unchecked_access().c_str() << std::endl;

    // loops until the user has pressed CTRL+c, the application has received a SIGTERM or SIGINT
    // signal or the user has called explicitly `waitset.stop()` in the `on_event` callback. We
    // didn't add this to the example so feel free to play around with it.
    waitset.wait_and_process(on_event).value();

    std::cout << "exit" << std::endl;

    return 0;
}
