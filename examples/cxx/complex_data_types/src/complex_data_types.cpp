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

#include "iox2/bb/static_string.hpp"
#include "iox2/bb/static_vector.hpp"
#include "iox2/iceoryx2.hpp"

#include <cstdint>
#include <iostream>
#include <utility>

struct ComplexData {
    iox2::bb::StaticString<4> name;           // NOLINT
    iox2::bb::StaticVector<uint64_t, 4> data; // NOLINT
};

struct ComplexDataType {
    uint64_t plain_old_data;
    iox2::bb::StaticString<8> text;                                  // NOLINT
    iox2::bb::StaticVector<uint64_t, 4> vec_of_data;                 // NOLINT
    iox2::bb::StaticVector<ComplexData, 404857> vec_of_complex_data; // NOLINT
};

constexpr iox2::bb::Duration CYCLE_TIME = iox2::bb::Duration::from_secs(1);

auto main() -> int {
    using namespace iox2;
    set_log_level_from_env_or(LogLevel::Info);
    auto node = NodeBuilder().create<ServiceType::Ipc>().value();

    auto service = node.service_builder(ServiceName::create("My/Funk/ServiceName").value())
                       .publish_subscribe<ComplexDataType>()
                       .max_publishers(16)  // NOLINT
                       .max_subscribers(16) // NOLINT
                       .open_or_create()
                       .value();

    auto publisher = service.publisher_builder().create().value();
    auto subscriber = service.subscriber_builder().create().value();

    uint64_t counter = 0;
    while (node.wait(CYCLE_TIME).has_value()) {
        counter += 1;
        auto sample = publisher.loan_uninit().value();
        new (&sample.payload_mut()) ComplexDataType {};

        auto& payload = sample.payload_mut();
        payload.plain_old_data = counter;
        payload.text = *iox2::bb::StaticString<8>::from_utf8("hello"); // NOLINT
        payload.vec_of_data.try_push_back(counter);
        payload.vec_of_complex_data.try_push_back(
            ComplexData { *iox2::bb::StaticString<4>::from_utf8("bla"),
                          iox2::bb::StaticVector<uint64_t, 4>::from_value<2>(counter) });

        auto initialized_sample = assume_init(std::move(sample));
        send(std::move(initialized_sample)).value();

        std::cout << counter << " :: send" << std::endl;

        auto recv_sample = subscriber.receive().value();
        while (recv_sample.has_value()) {
            std::cout << "received: " << recv_sample->payload().text.unchecked_access().c_str() << std::endl;
            recv_sample = subscriber.receive().value();
        }
    }

    std::cout << "exit" << std::endl;

    return 0;
}
