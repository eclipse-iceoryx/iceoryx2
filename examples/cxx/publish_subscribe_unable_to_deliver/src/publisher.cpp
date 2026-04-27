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

#include "iox2/bb/duration.hpp"
#include "iox2/iceoryx2.hpp"
#include "iox2/unable_to_deliver_action.hpp"
#include "iox2/unable_to_deliver_handler.hpp"
#include "transmission_data.hpp"

#include <chrono>
#include <iomanip>
#include <iostream>
#include <thread>
#include <utility>

// NOLINTBEGIN(cppcoreguidelines-avoid-magic-numbers, readability-magic-numbers) fine for the example

constexpr iox2::bb::Duration CYCLE_TIME = iox2::bb::Duration::from_millis(500);

auto main() -> int {
    using namespace iox2;
    set_log_level_from_env_or(LogLevel::Info);
    auto node = NodeBuilder().create<ServiceType::Ipc>().value();

    auto service = node.service_builder(ServiceName::create("My/Funk/ServiceName").value())
                       .publish_subscribe<TransmissionData>()
                       .enable_safe_overflow(false)
                       .open_or_create()
                       .value();

    auto counter = 0;
    auto delivery_incident_counter = 0;

    UnableToDeliverHandler unable_to_deliver_handler = [&](UnableToDeliverInfo& info) -> auto {
        // only print the port IDs in the first iteration of the retry loop of each delivery incident
        if (info.retries() == 0) {
            delivery_incident_counter += 1;
            std::cout << std::endl << std::setw(2) << std::setfill('0');
            std::cout << "Sample delivery interruption count " << delivery_incident_counter;

            auto sender_port_id = info.sender_port_id();
            auto receiver_port_id = info.sender_port_id();
            std::cout << "    Could not deliver sample " << counter << " from publisher sender id 0x" << std::hex;
            for (uint32_t i = 0; i < sender_port_id.size(); ++i) {
                // NOTE: the + promotes the value to an integer, else it will be interpreted as char
                std::cout << +*sender_port_id.element_at(i);
            }
            std::cout << " to subscriber receiver id 0x" << std::hex;
            for (uint32_t i = 0; i < receiver_port_id.size(); ++i) {
                // NOTE: the + promotes the value to an integer, else it will be interpreted as char
                std::cout << +*receiver_port_id.element_at(i);
            }
            std::cout << std::dec << std::endl;
        }

        // there are multiple mitigation options available and to showcase these options,
        // the mitigation is selected based on the incident counter
        switch (delivery_incident_counter % 4) {
        case 0: {
            // use the built-in sleeping strategy and keep retrying to send the sample
            // for 10ms and then discard the sample for the receiver that caused the
            // incident but continue to try delivering the sample to all other receiver
            // to whom no attempt was taken to deliver the sample, yet;
            // return with an error if the sample was not delivered to all receivers
            if (info.elapsed_time() < bb::Duration::from_millis(10)) {
                return UnableToDeliverAction::Retry;
            } else {
                std::cout << "    Retried for 10ms! Discarding sample and failing" << std::endl;
                return UnableToDeliverAction::DiscardDataAndFail;
            }
        }
        case 1: {
            // instead of using the built-in sleeping strategy, the sleep time is defined
            // by the handler and the sample is discarded after a specified amount of retries
            // for the receiver that caused the incident but continue to try delivering
            // the sample to all other receiver to whom no attempt was taken to deliver
            // the sample, yet;
            // return with an error if the sample was not delivered to all receivers
            if (info.retries() < 10) {
                std::cout << "    Sleeping 100ms and retry" << std::endl;
                std::this_thread::sleep_for(std::chrono::milliseconds(100));
                return UnableToDeliverAction::Retry;
            } else {
                return UnableToDeliverAction::DiscardDataAndFail;
            }
        }
        case 2: {
            // just discard the sample for the receiver involved in the incident and
            // continue to try delivering the sample to all other receiver to whom no
            // attempt was taken to deliver the sample, yet
            std::cout << "    Discarding sample silently" << std::endl;
            return UnableToDeliverAction::DiscardData;
        }
        default: {
            // just discard the sample for the receiver involved in the incident and
            // continue to try delivering the sample to all other receiver to whom
            // no attempt was taken to deliver the sample, yet;
            // return with an error if the sample was not delivered to all receivers
            std::cout << "    Discarding sample and failing" << std::endl;
            return UnableToDeliverAction::DiscardDataAndFail;
        }
        }
    };

    auto publisher =
        service.publisher_builder().set_unable_to_deliver_handler(&unable_to_deliver_handler).create().value();

    while (node.wait(CYCLE_TIME).has_value()) {
        counter += 1;

        auto sample = publisher.loan_uninit().value();

        auto initialized_sample = sample.write_payload(TransmissionData { counter, counter * 3, counter * 812.12 });

        std::cout << "Sending sample " << counter << "..." << std::endl;
        if (!send(std::move(initialized_sample)).has_value()) {
            std::cout << "Failed to send sample" << std::endl;
        }
    }

    std::cout << "exit" << std::endl;

    return 0;
}

// NOLINTEND(cppcoreguidelines-avoid-magic-numbers, readability-magic-numbers)
