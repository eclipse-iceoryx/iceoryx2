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
        .service_builder(&"Service With Dynamic Data".try_into()?)
        .publish_subscribe::<[u8]>()
        .open_or_create()?;

    let publisher = service
        .publisher_builder()
        // We guess that the samples are at most 16 bytes in size.
        // This is just a hint to the underlying allocator and is purely optional
        // The better the guess is the less reallocations will be performed
        .initial_max_slice_len(16)
        // The underlying sample size will be increased with a power of two strategy
        // when [`Publisher::loan_slice()`] or [`Publisher::loan_slice_uninit()`] require more
        // memory than available.
        .allocation_strategy(AllocationStrategy::PowerOfTwo)
        .create()?;

    let mut counter = 1;

    while node.wait(CYCLE_TIME).is_ok() {
        let required_memory_size = 1_000_000.min(counter * counter);
        let sample = publisher.loan_slice_uninit(required_memory_size)?;
        let sample = sample.write_from_fn(|byte_idx| ((byte_idx + counter) % 255) as u8);

        sample.send()?;

        println!("Send sample {counter} with {required_memory_size} bytes...");

        counter += 1;
    }

    println!("exit");

    Ok(())
}
