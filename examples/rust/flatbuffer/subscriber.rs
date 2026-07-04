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

extern crate alloc;

use crate::unbounded_data_generated::example::UnboundedData;
use alloc::boxed::Box;
use core::time::Duration;
use iceoryx2::{prelude::*, service::marker::Flatbuffer};

#[path = "unbounded_data_generated.rs"]
#[allow(clippy::all)]
#[rustfmt::skip]
mod unbounded_data_generated;

const CYCLE_TIME: Duration = Duration::from_secs(1);

fn main() -> Result<(), Box<dyn core::error::Error>> {
    set_log_level_from_env_or(LogLevel::Info);

    // export IOX2_FLATBUFFER_SCHEMA_PATH=${pwd}/examples/rust/flatbuffer
    let lookup_path = std::env::var("IOX2_FLATBUFFER_SCHEMA_PATH")
        .inspect_err(|e| cerrln!("Please define IOX2_FLATBUFFER_SCHEMA_PATH! [{e:?}]"))?;

    let mut config = Config::default();
    config.global.service.flatbuffer_schema_path = Some(lookup_path.as_str().try_into()?);

    let node = NodeBuilder::new()
        .config(&config)
        .create::<ipc::Service>()?;

    let service = node
        .service_builder(&"My/Flatbuffer/Service".try_into()?)
        .publish_subscribe::<Flatbuffer<UnboundedData>>()
        .open_or_create()?;

    let subscriber = service.subscriber_builder().create()?;

    coutln!("Subscriber ready to receive data!");

    while node.wait(CYCLE_TIME).is_ok() {
        while let Some(sample) = subscriber.receive()? {
            coutln!("received: {:?}", *sample);
        }
    }

    coutln!("exit");

    Ok(())
}
