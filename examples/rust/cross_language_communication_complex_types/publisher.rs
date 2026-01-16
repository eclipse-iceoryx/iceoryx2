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

extern crate alloc;
use alloc::boxed::Box;
use alloc::str::FromStr;

use examples_common::ComplexType;
use examples_common::FullName;
use iceoryx2::prelude::*;
use iceoryx2_bb_container::string::*;
use iceoryx2_bb_container::vector::*;

const CYCLE_TIME: Duration = Duration::from_secs(1);

fn main() -> Result<(), Box<dyn core::error::Error>> {
    set_log_level_from_env_or(LogLevel::Info);

    let node = NodeBuilder::new().create::<ipc::Service>()?;

    let service = node
        .service_builder(&"CrossLanguageComplexTypes".try_into()?)
        .publish_subscribe::<ComplexType>()
        .open_or_create()?;

    let publisher = service.publisher_builder().create()?;

    let mut counter: u64 = 0;

    while node.wait(CYCLE_TIME).is_ok() {
        counter += 1;
        let mut sample = publisher.loan_uninit()?;
        unsafe { ComplexType::placement_default(sample.payload_mut().as_mut_ptr()) };
        let mut sample = unsafe { sample.assume_init() };

        sample.address_book.push(FullName {
            first_name: StaticString::from_str("Kermit")?,
            last_name: StaticString::from_str("The Frog")?,
        })?;
        sample.some_matrix.resize(8, StaticVec::new())?;
        for row in sample.some_matrix.iter_mut() {
            row.resize(8, 0.0)?;
        }
        sample.some_matrix[2][5] = counter as f64 * 0.8912;
        sample.some_value = 5;

        sample.send()?;

        cout!("Send sample {counter} ...");
    }

    cout!("exit");

    Ok(())
}
