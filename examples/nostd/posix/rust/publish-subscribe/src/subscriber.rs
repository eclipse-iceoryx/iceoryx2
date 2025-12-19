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

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::time::Duration;

extern crate alloc;

use iceoryx2::prelude::*;
use iceoryx2_bb_posix::signal::SignalHandler;

use posix_nostd_common::transmission_data::TransmissionData;

const CYCLE_TIME: Duration = Duration::from_secs(1);

#[no_mangle]
extern "C" fn main() -> i32 {
    let node = match NodeBuilder::new().create::<ipc::Service>() {
        Ok(node) => node,
        Err(e) => {
            cout!("Failed to create node: {:?}", e);
            return 1;
        }
    };

    let service = match node
        .service_builder(&"My/Funk/ServiceName".try_into().unwrap())
        .publish_subscribe::<TransmissionData>()
        .open_or_create()
    {
        Ok(service) => service,
        Err(e) => {
            cout!("Failed to open or create service: {:?}", e);
            return 1;
        }
    };

    let subscriber = match service.subscriber_builder().create() {
        Ok(subscriber) => subscriber,
        Err(e) => {
            cout!("Failed to create subscriber: {:?}", e);
            return 1;
        }
    };

    cout!("Subscriber ready to receive data!");

    while node.wait(CYCLE_TIME).is_ok() {
        loop {
            match subscriber.receive() {
                Ok(Some(sample)) => {
                    cout!("received: {:?}", *sample);
                }
                Ok(None) => {
                    // No more samples available
                    break;
                }
                Err(e) => {
                    cout!("Failed to receive sample: {:?}", e);
                    break;
                }
            }
        }
    }

    cout!("exit");

    0
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    cout!("\n╔═══════════════════════════════════════╗\n");
    cout!("║           PANIC OCCURRED!             ║\n");
    cout!("╚═══════════════════════════════════════╝\n");

    if let Some(location) = info.location() {
        cout!("Location: {}:{}\n", location.file(), location.line());
    }

    cout!("Message: {}\n", info);

    SignalHandler::abort();

    loop {
        core::hint::spin_loop();
    }
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
