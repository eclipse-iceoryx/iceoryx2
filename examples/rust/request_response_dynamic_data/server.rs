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

const CYCLE_TIME: Duration = Duration::from_millis(100);

fn main() -> Result<(), Box<dyn core::error::Error>> {
    set_log_level_from_env_or(LogLevel::Info);
    let node = NodeBuilder::new().create::<ipc::Service>()?;

    let service = node
        .service_builder(&"example//dynamic_request_response".try_into()?)
        .request_response::<[u8], [u8]>()
        .open_or_create()?;

    let server = service
        .server_builder()
        // We guess that the samples are at most 16 bytes in size.
        // This is just a hint to the underlying allocator and is purely optional
        // The better the guess is the less reallocations will be performed
        .initial_max_slice_len(16)
        // The underlying sample size will be increased with a power of two strategy
        // when [`ActiveRequest::loan_slice()`] or [`ActiveRequest::loan_slice_uninit()`]
        // requires more memory than available.
        .allocation_strategy(AllocationStrategy::PowerOfTwo)
        .create()?;

    println!("Server ready to receive requests!");

    let mut counter = 1;
    while node.wait(CYCLE_TIME).is_ok() {
        while let Some(active_request) = server.receive()? {
            println!(
                "received request with {} bytes ...",
                active_request.payload().len()
            );

            let required_memory_size = 1_000_000.min(counter * counter);
            let response = active_request.loan_slice_uninit(required_memory_size)?;
            let response = response.write_from_fn(|byte_idx| ((byte_idx + counter) % 255) as u8);
            println!("  send response with {} bytes", response.payload().len());
            response.send()?;
        }

        counter += 1;
    }

    println!("exit");

    Ok(())
}
