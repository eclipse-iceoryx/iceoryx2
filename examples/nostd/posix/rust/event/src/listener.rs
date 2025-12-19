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

#![no_std]
#![no_main]

extern crate posix_nostd_common;

use core::panic::PanicInfo;
use core::time::Duration;

extern crate alloc;

use iceoryx2::prelude::*;
use iceoryx2_bb_posix::signal::SignalHandler;

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
        .service_builder(&"MyEventName".try_into().unwrap())
        .event()
        .open_or_create()
    {
        Ok(service) => service,
        Err(e) => {
            cout!("Failed to open or create service: {:?}", e);
            return 1;
        }
    };

    let listener = match service.listener_builder().create() {
        Ok(listener) => listener,
        Err(e) => {
            cout!("Failed to create listener: {:?}", e);
            return 1;
        }
    };

    cout!("Listener ready to receive events!");

    while node.wait(Duration::ZERO).is_ok() {
        if let Ok(Some(event_id)) = listener.timed_wait_one(CYCLE_TIME) {
            cout!("event was triggered with id: {event_id:?}");
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
