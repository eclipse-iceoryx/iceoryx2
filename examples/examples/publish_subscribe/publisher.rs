// Copyright (c) 2023 Contributors to the Eclipse Foundation
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
use transmission_data::TransmissionData;

const CYCLE_TIME: Duration = Duration::from_secs(1);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let service_name = ServiceName::new("My/Funk/ServiceName")?;

    let service = zero_copy::Service::new(&service_name)
        .publish_subscribe()
        .open_or_create::<TransmissionData>()?;

    let publisher = service.publisher().create()?;

    let mut counter: u64 = 0;

    while let Iox2Event::Tick = Iox2::wait(CYCLE_TIME) {
        counter += 1;
        let sample = publisher.loan_uninit()?;

        let sample = sample.write_payload(TransmissionData {
            x: counter as i32,
            y: counter as i32 * 3,
            funky: counter as f64 * 812.12,
        });

        publisher.send(sample)?;

        println!("Send sample {} ...", counter);
    }

    println!("exit ...");

    Ok(())
}
