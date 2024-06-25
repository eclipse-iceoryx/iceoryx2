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
use examples_common::CustomHeader;
use iceoryx2::prelude::*;

const CYCLE_TIME: Duration = Duration::from_secs(1);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let node = NodeBuilder::new().create::<zero_copy::Service>()?;

    let service = node
        .service_builder("My/Funk/ServiceName".try_into()?)
        .publish_subscribe::<u64>()
        // define the CustomHeader as metadata which is stored in the
        // beginning of every Sample
        .metadata::<CustomHeader>()
        .open_or_create()?;

    let publisher = service.publisher_builder().create()?;

    let mut counter: u64 = 0;

    while let Iox2Event::Tick = Iox2::wait(CYCLE_TIME) {
        counter += 1;
        let mut sample = publisher.loan_uninit()?;

        // set some metadata details
        sample.metadata_mut().version = 123;
        sample.metadata_mut().timestamp = 80337 + counter;

        let sample = sample.write_payload(counter);

        sample.send()?;

        println!("Send sample {} ...", counter);
    }

    println!("exit");

    Ok(())
}
