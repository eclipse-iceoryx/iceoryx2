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
pub extern "C" fn run_publisher(seconds: u32) -> i32 {
    set_log_level(iceoryx2_bb_log::LogLevel::Info);

    let service_name = ServiceName::new("Hello/from/C");
    let node = NodeBuilder::new().create::<zero_copy::Service>();

    if service_name.is_err() || node.is_err() {
        return -1;
    }

    let service_name = service_name.unwrap();
    let node = node.unwrap();

    let service = node
        .service_builder(&service_name)
        .publish_subscribe::<u64>()
        .open_or_create();

    if service.is_err() {
        return -1;
    }

    let service = service.unwrap();

    let publisher = service.publisher_builder().create();

    if publisher.is_err() {
        return -1;
    }

    let publisher = publisher.unwrap();

    let mut counter: u64 = 0;

    let mut remaining_seconds = seconds;

    while let NodeEvent::Tick = node.wait(CYCLE_TIME) {
        counter += 1;
        let sample = publisher.loan_uninit();

        if sample.is_err() {
            return -1;
        }

        let sample = sample.unwrap();

        let sample = sample.write_payload(counter);

        if sample.send().is_err() {
            return -1;
        }

        println!("Send sample {} ...", counter);

        remaining_seconds = remaining_seconds.saturating_sub(1);
        if remaining_seconds == 0 {
            break;
        }
    }

    println!("exit");

    0
}
