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
    let max_event_id = service.static_config().event_id_max_value();

    let notifier = match service.notifier_builder().create() {
        Ok(notifier) => notifier,
        Err(e) => {
            cout!("Failed to create notifier: {:?}", e);
            return 1;
        }
    };

    let mut counter: usize = 0;
    while node.wait(CYCLE_TIME).is_ok() {
        counter += 1;
        if let Err(e) = notifier.notify_with_custom_event_id(EventId::new(counter % max_event_id)) {
            cout!("Failed to send event: {:?}", e);
            continue;
        }

        cout!("Trigger event with id {counter} ...");
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
