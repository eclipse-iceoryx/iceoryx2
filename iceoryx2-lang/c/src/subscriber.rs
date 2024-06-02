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
use iceoryx2_bb_log::set_log_level;

const CYCLE_TIME: Duration = Duration::from_secs(1);

#[no_mangle]
pub extern "C" fn run_subscriber(seconds: u32) -> i32 {
    set_log_level(iceoryx2_bb_log::LogLevel::Info);

    let service_name = ServiceName::new("Hello/from/C");

    if service_name.is_err() {
        return -1;
    }

    let service_name = service_name.unwrap();

    let service = zero_copy::Service::new(&service_name)
        .publish_subscribe::<u64>()
        .open_or_create();

    if service.is_err() {
        return -1;
    }

    let service = service.unwrap();

    let subscriber = service.subscriber().create();

    if subscriber.is_err() {
        return -1;
    }

    let subscriber = subscriber.unwrap();

    let mut remaining_seconds = seconds;

    while let Iox2Event::Tick = Iox2::wait(CYCLE_TIME) {
        loop {
            match subscriber.receive() {
                Ok(Some(sample)) => println!("received: {:?}", *sample),
                Ok(None) => break,
                Err(_) => return -1,
            }
        }

        remaining_seconds = remaining_seconds.saturating_sub(1);
        if remaining_seconds == 0 {
            break;
        }
    }

    println!("exit");

    0
}
