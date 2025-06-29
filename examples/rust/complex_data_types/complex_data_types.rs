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
use iceoryx2_bb_container::{
    byte_string::FixedSizeByteString, queue::FixedSizeQueue, vec::FixedSizeVec,
};

// For both data types we derive from PlacementDefault to allow in memory initialization
// without any copy. Avoids stack overflows when data type is larger than the available stack.
#[derive(Debug, Default, PlacementDefault, ZeroCopySend)]
#[repr(C)]
pub struct ComplexData {
    name: FixedSizeByteString<4>,
    data: FixedSizeVec<u64, 4>,
}

// For both data types we derive from PlacementDefault to allow in memory initialization
// without any copy. Avoids stack overflows when data type is larger than the available stack.
#[derive(Debug, Default, PlacementDefault, ZeroCopySend)]
#[repr(C)]
pub struct ComplexDataType {
    plain_old_data: u64,
    text: FixedSizeByteString<8>,
    vec_of_data: FixedSizeVec<u64, 4>,
    vec_of_complex_data: FixedSizeVec<ComplexData, 404857>,
    a_queue_of_things: FixedSizeQueue<FixedSizeByteString<4>, 2>,
}

const CYCLE_TIME: Duration = Duration::from_secs(1);

fn main() -> Result<(), Box<dyn core::error::Error>> {
    set_log_level_from_env_or(LogLevel::Info);
    let node = NodeBuilder::new().create::<ipc::Service>()?;

    let service = node
        .service_builder(&"Complex Data Type Example".try_into()?)
        .publish_subscribe::<ComplexDataType>()
        .max_publishers(16)
        .max_subscribers(16)
        .open_or_create()?;

    let publisher = service.publisher_builder().create()?;
    let subscriber = service.subscriber_builder().create()?;
    let mut counter = 0;

    while node.wait(CYCLE_TIME).is_ok() {
        // ComplexDataType as a size of over 30MB, we need to perform a placement new
        // otherwise we will encounter a stack overflow in debug builds.
        // Therefore, we acquire an uninitialized sample, use the PlacementDefault
        // trait to initialize ComplexDataType in place and then populate it with data.
        let mut sample = publisher.loan_uninit()?;
        unsafe { ComplexDataType::placement_default(sample.payload_mut().as_mut_ptr()) };
        let mut sample = unsafe { sample.assume_init() };

        let payload = sample.payload_mut();
        payload.plain_old_data = counter;
        payload.text = FixedSizeByteString::from_bytes(b"hello")?;
        payload.vec_of_data.push(counter);
        payload.vec_of_complex_data.push(ComplexData {
            name: FixedSizeByteString::from_bytes(b"bla")?,
            data: {
                let mut v = FixedSizeVec::new();
                v.fill(counter);
                v
            },
        });
        payload
            .a_queue_of_things
            .push(FixedSizeByteString::from_bytes(b"buh")?);

        sample.send()?;
        println!("{counter} :: send");

        // receive sample and print it
        while let Some(sample) = subscriber.receive()? {
            println!(
                "{} :: received: {:?}",
                counter,
                sample.payload().plain_old_data
            );
        }

        counter += 1;
    }

    Ok(())
}
