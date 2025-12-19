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

mod startup;

use bare_metal_nostd_common::global_allocator;
#[cfg(feature = "semihosting")]
use bare_metal_nostd_common::semihosting;
use bare_metal_nostd_common::writer;

use core::panic::PanicInfo;

use iceoryx2::prelude::*;

#[derive(Debug, Clone, Copy, ZeroCopySend)]
#[type_name("TransmissionData")]
#[repr(C)]
pub struct TransmissionData {
    pub x: i32,
    pub y: i32,
    pub funky: f64,
}

macro_rules! debug {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        use writer::OUTPUT;
        write!(OUTPUT.blocking_lock(), $($arg)*).unwrap();
    }};
}

#[no_mangle]
pub extern "C" fn kmain() -> ! {
    debug!("\n");
    debug!("╔═════════════════════════════════╗\n");
    debug!("║  Bare-Metal Rust on Cortex-R5   ║\n");
    debug!("║   iceoryx2 publish-subscribe    ║\n");
    debug!("╚═════════════════════════════════╝\n");
    debug!("\n");

    global_allocator::initialize();
    debug!("[✓] allocator initialized\n");

    let node = NodeBuilder::new()
        .config(&Config::default())
        .create::<local::Service>()
        .unwrap();
    debug!("[✓] node created\n");

    let service = node
        .service_builder(&"My/Funk/ServiceName".try_into().unwrap())
        .publish_subscribe::<TransmissionData>()
        .open_or_create()
        .unwrap();
    debug!("[✓] service created\n");

    let publisher = service.publisher_builder().create().unwrap();
    debug!("[✓] publisher created\n");

    let subscriber = service.subscriber_builder().create().unwrap();
    debug!("[✓] subscriber created\n");

    debug!("[✓] system initialized\n");

    debug!("\n─── application start ───\n\n");

    let mut counter: u64 = 0;
    loop {
        counter += 1;
        let sample = publisher.loan_uninit().unwrap();

        let x = counter as i32;
        let y = counter as i32 * 3;
        let funky = counter as f64 * 812.12;

        debug!("[TX #{:3}] x={}, y={}, funky={:.2}\n", counter, x, y, funky);
        let sample = sample.write_payload(TransmissionData { x, y, funky });
        sample.send().unwrap();

        while let Some(sample) = subscriber.receive().unwrap() {
            debug!(
                "[RX #{:3}] x={}, y={}, funky={:.2}\n",
                counter, sample.x, sample.y, sample.funky
            );
        }

        if counter == 100 {
            break;
        }
    }

    debug!("\n─── application end ───\n");

    exit(0);
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    debug!("\n╔═══════════════════════════════════════╗\n");
    debug!("║           PANIC OCCURRED!             ║\n");
    debug!("╚═══════════════════════════════════════╝\n");

    if let Some(location) = info.location() {
        debug!("Location: {}:{}\n", location.file(), location.line());
    }

    debug!("Message: {}\n", info);

    exit(1);
}

fn exit(code: usize) -> ! {
    #[cfg(feature = "semihosting")]
    semihosting::exit(code);

    #[cfg(not(feature = "semihosting"))]
    loop {
        // When semihosting is disabled, just loop forever
        let _ = code;
    }
}
