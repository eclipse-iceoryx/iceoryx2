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

use core::time::Duration;
use iceoryx2::prelude::*;

const CYCLE_TIME: Duration = Duration::from_millis(750);

fn main() -> Result<(), Box<dyn core::error::Error>> {
    set_log_level_from_env_or(LogLevel::Info);

    let node = NodeBuilder::new()
        // The generic argument defines the service variant. Different variants can use
        // different mechanisms. For instance the upcoming `cuda::Service` would use GPU memory
        // or the `local::Service` would use mechanisms that are optimized for intra-process
        // communication.
        //
        // All services which are created via this `Node` use the same service variant.
        .create::<ipc::Service>()?;

    let service = node
        .service_builder(&"Service-Variants-Example".try_into()?)
        .publish_subscribe::<u64>()
        .open_or_create()?;

    let publisher = service.publisher_builder().create()?;

    let mut counter = 0u64;
    while node.wait(CYCLE_TIME).is_ok() {
        println!("send: {counter}");
        publisher.send_copy(counter)?;
        counter += 1;
    }

    println!("exit");

    Ok(())
}
