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

use core::time::Duration;
use iceoryx2::prelude::*;

const CYCLE_TIME: Duration = Duration::from_secs(1);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let node = NodeBuilder::new().create::<ipc::Service>()?;

    let service = node
        .service_builder(&"Service/With/Properties".try_into()?)
        .publish_subscribe::<u64>()
        .create_with_attributes(
            // define a set of properties that are static for the lifetime
            // of the service
            &AttributeSpecifier::new()
                .define("dds_service_mapping", "my_funky_service_name")
                .define("tcp_serialization_format", "cdr")
                .define("someip_service_mapping", "1/2/3")
                .define("camera_resolution", "1920x1080"),
        )?;

    let publisher = service.publisher_builder().create()?;

    println!("defined service attributes: {:?}", service.attributes());

    while let NodeEvent::Tick = node.wait(CYCLE_TIME) {
        let sample = publisher.loan_uninit()?;
        let sample = sample.write_payload(0);
        sample.send()?;
    }

    println!("exit");

    Ok(())
}
