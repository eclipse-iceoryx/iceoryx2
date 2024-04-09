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

use std::time::Duration;

use iceoryx2::prelude::*;
use iceoryx2_bb_container::{
    byte_string::FixedSizeByteString, queue::FixedSizeQueue, vec::FixedSizeVec,
};

#[derive(Debug, Default)]
#[repr(C)]
pub struct ComplexData {
    name: FixedSizeByteString<4>,
    data: FixedSizeVec<u64, 4>,
}

#[derive(Debug, Default)]
#[repr(C)]
pub struct ComplexDataType {
    plain_old_data: u64,
    text: FixedSizeByteString<8>,
    vec_of_data: FixedSizeVec<u64, 4>,
    vec_of_complex_data: FixedSizeVec<ComplexData, 4>,
    a_queue_of_things: FixedSizeQueue<FixedSizeByteString<4>, 2>,
}

const CYCLE_TIME: Duration = Duration::from_secs(1);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let service_name = ServiceName::new("Complex Data Type Example")?;

    let service = zero_copy::Service::new(&service_name)
        .publish_subscribe()
        .open_or_create::<ComplexDataType>()?;

    let publisher = service.publisher().create()?;
    let subscriber = service.subscriber().create()?;
    let mut counter = 0;

    while let Iox2Event::Tick = Iox2::wait(CYCLE_TIME) {
        // acquire and send out sample
        let mut sample = publisher.loan()?;
        sample.payload_mut().plain_old_data = counter;
        sample.payload_mut().text = FixedSizeByteString::from_bytes(b"hello")?;
        sample.payload_mut().vec_of_data.push(counter);
        sample.payload_mut().vec_of_complex_data.push(ComplexData {
            name: FixedSizeByteString::from_bytes(b"bla")?,
            data: {
                let mut v = FixedSizeVec::new();
                v.fill(counter);
                v
            },
        });
        sample
            .payload_mut()
            .a_queue_of_things
            .push(FixedSizeByteString::from_bytes(b"buh")?);

        sample.send()?;
        println!("{} :: send", counter);

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
