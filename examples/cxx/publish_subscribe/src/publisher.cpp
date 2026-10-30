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

#include "iox2/bb/path.hpp"
#include "iox2/iceoryx2.hpp"
#include "iox2/port_name.hpp"
#include "transmission_data.hpp"

#include <iostream>
#include <utility>

constexpr iox2::bb::Duration CYCLE_TIME = iox2::bb::Duration::from_secs(1);

auto main() -> int {
    using namespace iox2;
    set_log_level_from_env_or(LogLevel::Info);

    auto config = Config::global_config().to_owned();
    config.global().set_prefix(iox2::bb::FileName::create("rector_").value());
    config.global().set_root_path(iox2::bb::Path::create("/tmp/rector").value());

    config.global().node().set_cleanup_dead_nodes_on_creation(false);
    config.global().node().set_cleanup_dead_nodes_on_destruction(false);
    config.global().service().set_cleanup_dead_nodes_on_open(false);

    auto node =
        NodeBuilder().name(NodeName::create("node-pub").value()).config(config).create<ServiceType::Ipc>().value();

    auto service = node.service_builder(ServiceName::create("My/Funk/ServiceName").value())
                       .publish_subscribe<TransmissionData>()
                       .history_size(1)
                       .subscriber_max_buffer_size(5)
                       .subscriber_max_borrowed_samples(5)
                       .open_or_create()
                       .value();

    auto publisher =
        service.publisher_builder().name(PortName::create("publisher").value()).max_loaned_samples(1).create().value();

    auto counter = 0;
    while (node.wait(CYCLE_TIME).has_value()) {
        counter += 1;

        auto sample = publisher.loan_uninit().value();

        auto initialized_sample =
            sample.write_payload(TransmissionData { counter, counter * 3, counter * 812.12 }); // NOLINT

        send(std::move(initialized_sample)).has_value();

        std::cout << "Send sample " << counter << "..." << std::endl;
    }

    std::cout << "exit" << std::endl;

    return 0;
}
