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
use iceoryx2::{prelude::*, service::static_config::StaticConfig};
use iceoryx2_services_discovery::service_discovery::service_name;

const CYCLE_TIME: Duration = Duration::from_secs(1);

fn main() -> Result<(), Box<dyn core::error::Error>> {
    set_log_level_from_env_or(LogLevel::Info);
    let node = NodeBuilder::new().create::<ipc::Service>()?;

    let service = node
        .service_builder(service_name().try_into()?)
        .request_response::<(), [StaticConfig]>()
        .open_or_create()?;

    let client = service.client_builder().create()?;

    let pending_response = client.send_copy(())?;

    // IMPORTANT: We need to wait for the request to be sent before we can receive responses.
    while !node.wait(CYCLE_TIME).is_ok() {}

    while let Some(response) = pending_response.receive()? {
        for service in response.payload().iter() {
            println!("Service ID: {:?}", service.service_id().as_str());
            println!("Service Name: {:?}", service.name().as_str());
            println!();
        }
    }

    println!("exit");

    Ok(())
}
