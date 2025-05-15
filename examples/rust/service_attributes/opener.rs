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

fn main() -> Result<(), Box<dyn core::error::Error>> {
    set_log_level_from_env_or(LogLevel::Info);
    let node = NodeBuilder::new().create::<ipc::Service>()?;

    let service = node
        .service_builder(&"Service/With/Properties".try_into()?)
        .publish_subscribe::<u64>()
        .open_with_attributes(
            // define attributes that the service requires
            // if no attributes are defined the service accepts any attribute
            &AttributeVerifier::new()
                .require(&"camera_resolution".try_into()?, &"1920x1080".try_into()?)
                .require_key(&"dds_service_mapping".try_into()?),
        )?;

    let subscriber = service.subscriber_builder().create()?;

    println!("defined service attributes: {:?}", service.attributes());

    while node.wait(CYCLE_TIME).is_ok() {
        while let Some(sample) = subscriber.receive()? {
            println!("received: {:?}", *sample);
        }
    }

    println!("exit");

    Ok(())
}
