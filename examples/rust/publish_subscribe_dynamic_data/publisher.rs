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
        .service_builder(&"Service With Dynamic Data".try_into()?)
        .publish_subscribe::<[u8]>()
        .open_or_create()?;

    let maximum_elements = 1024;
    let publisher = service
        .publisher_builder()
        .max_slice_len(maximum_elements)
        .create()?;

    let mut counter = 0;

    while node.wait(CYCLE_TIME).is_ok() {
        let required_memory_size = (counter % 16) + 1;
        let sample = publisher.loan_slice_uninit(required_memory_size)?;
        let sample = sample.write_from_fn(|byte_idx| ((byte_idx + counter) % 255) as u8);

        sample.send()?;

        println!(
            "Send sample {} with {} bytes...",
            counter, required_memory_size
        );

        counter += 1;
    }

    println!("exit");

    Ok(())
}
