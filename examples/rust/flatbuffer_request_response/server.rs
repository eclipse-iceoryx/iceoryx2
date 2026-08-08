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
    data_props_generated::example::{DataProps, DataPropsArgs},
    unbounded_data_generated::example::UnboundedData,
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
        // Those methods allows us to use a custom schema file path when no schema lookup path was
        // defined or when a custom file is required (maybe outside of the lookup path).
        //
        // .request_flatbuffer_schema_path(&"unbounded_data.fbs".try_into()?)
        // .response_flatbuffer_schema_path(&"data_props.fbs".try_into()?)
        .request_user_header::<u64>()
        .response_user_header::<u64>()
        .open_or_create()?;

    let server = service
        .server_builder()
        // We start with 1024 bytes. The more accurate the initial_reserved_memory
        // estimate is, the fewer reallocations will be required. Reallocations occur
        // only at the beginning of communication. Once the server's data segment
        // has been resized appropriately, all subsequent samples will use that size.
        .initial_reserved_memory(1024)
        // By default, the allocation strategy is Static, which does not allow
        // reallocations when initial_reserved_memory is exhausted. Set it to
        // PowerOfTwo or BestFit to enable reallocations.
        //
        // The maximum number of reallocations is 256. BestFit allocates only the
        // explicitly requested amount of memory, so this limit can be reached
        // quickly. Increasing initial_reserved_memory reduces the number of
        // reallocations.
        .allocation_strategy(AllocationStrategy::PowerOfTwo)
        .create()?;

    coutln!("Server ready to receive requests!");

    let mut response_counter = 0;
    while node.wait(CYCLE_TIME).is_ok() {
        while let Some(active_request) = server.receive()? {
            let data = active_request.payload_root()?;

            coutln!("title: {}", data.title().unwrap_or_default());
            coutln!("user header: {}", active_request.user_header());

            if let Some(entries) = data.entries() {
                for (index, entry) in entries.iter().enumerate() {
                    // send a response for every entry we have received
                    response_counter += 1;
                    let mut response = active_request.loan_flatbuffer()?;
                    let builder = response.flatbuffer_builder();

                    let received_entries_len = entry.data_1().max(0) as u64 + entry.data_2();
                    let data_props = DataProps::create(
                        builder,
                        &DataPropsArgs {
                            received_entries_len,
                        },
                    );

                    let mut response = response.assume_init(data_props);
                    *response.user_header_mut() = response_counter;

                    coutln!(
                        "  Send response: {}+{} = {received_entries_len} to: Entry {index}: data_1={}, data_2={}",
                        entry.data_1(),
                        entry.data_2(),
                        entry.data_1(),
                        entry.data_2()
                    );

                    response.send()?;
                }
            }
            coutln!("");
        }
    }

    coutln!("exit");

    Ok(())
}
