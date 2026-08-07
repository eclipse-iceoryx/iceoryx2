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
    data_props_generated::example::DataProps,
    unbounded_data_generated::example::{Entry, EntryArgs, UnboundedData, UnboundedDataArgs},
};

#[path = "unbounded_data_generated.rs"]
#[allow(clippy::all)]
#[rustfmt::skip]
mod unbounded_data_generated;

#[path = "data_props_generated.rs"]
#[allow(clippy::all)]
#[rustfmt::skip]
mod data_props_generated;

const CYCLE_TIME: Duration = Duration::from_secs(1);

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
        // This method allows us to use a custom schema file path when no schema lookup path was
        // defined or when a custom file is required (maybe outside of the lookup path).
        //
        // .request_flatbuffer_schema_path(&"unbounded_data.fbs".try_into()?)
        .request_user_header::<u64>()
        .open_or_create()?;

    let client = service
        .client_builder()
        // We start with 1024 bytes. The more accurate the initial_reserved_memory
        // estimate is, the fewer reallocations will be required. Reallocations occur
        // only at the beginning of communication. Once the publisher's data segment
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

    let mut request_counter = 0;
    while node.wait(CYCLE_TIME).is_ok() {
        request_counter += 1;
        let mut request = client.loan_flatbuffer()?;
        let builder = request.flatbuffer_builder();

        // BEGIN: standard flatbuffer API
        let title = builder.create_string("Hello World!");

        let mut entries = vec![];
        for i in 0..(request_counter % 15) {
            entries.push(Entry::create(
                builder,
                &EntryArgs {
                    data_1: (6 * i + 5) as i32,
                    data_2: 6 * i + 7,
                },
            ));
        }

        let entries = builder.create_vector(&entries);

        let unbounded_data = UnboundedData::create(
            builder,
            &UnboundedDataArgs {
                title: Some(title),
                entries: Some(entries),
            },
        );
        // END: standard flatbuffer API

        // calls builder.finish(unbounded_data, None) and sets the payload offset
        let mut request = request.assume_init(unbounded_data);
        *request.user_header_mut() = request_counter;

        coutln!("send request {request_counter} ...");
        let pending_response = request.send()?;
    }

    coutln!("exit");

    Ok(())
}
