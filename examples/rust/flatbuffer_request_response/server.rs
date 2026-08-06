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

use core::time::Duration;
use iceoryx2::prelude::*;

use crate::{
    data_props_generated::example::DataProps, unbounded_data_generated::example::UnboundedData,
};

#[path = "unbounded_data_generated.rs"]
#[allow(clippy::all)]
#[rustfmt::skip]
mod unbounded_data_generated;

#[path = "data_props_generated.rs"]
#[allow(clippy::all)]
#[rustfmt::skip]
mod data_props_generated;

const CYCLE_TIME: Duration = Duration::from_millis(100);

fn main() -> Result<(), Box<dyn core::error::Error>> {
    set_log_level_from_env_or(LogLevel::Info);

    // export IOX2_FLATBUFFER_SCHEMA_PATH=${pwd}/examples/rust/flatbuffer_request_response
    let lookup_path = std::env::var("IOX2_FLATBUFFER_SCHEMA_PATH")
        .inspect_err(|e| cerrln!("Please define IOX2_FLATBUFFER_SCHEMA_PATH! [{e:?}]"))?;

    let mut config = Config::default();
    config.global.service.flatbuffer_schema_path = Some(lookup_path.as_str().try_into()?);

    let node = NodeBuilder::new()
        // Use the config with the defined flatbuffer schema path to enable automatic flatbuffer
        // schema file lookup.
        .config(&config)
        .create::<ipc::Service>()?;

    let service = node
        .service_builder(&"Flatbuffer/Request/Response".try_into()?)
        .request_response::<Flatbuffer<UnboundedData>, Flatbuffer<DataProps>>()
        .open_or_create()?;

    let _server = service.server_builder().create()?;

    coutln!("Server ready to receive requests!");

    while node.wait(CYCLE_TIME).is_ok() {}

    coutln!("exit");

    Ok(())
}
